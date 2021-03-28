use std::fs::{File, read_to_string, write};
use std::{io, fs};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use tar::Archive;
use sha2::{Sha256, Digest};
use json::JsonValue;
use flate2::{Compression, GzBuilder};
use flate2::read::GzDecoder;
use walkdir::{WalkDir, DirEntry};

use crate::errors::{FileCheckError, InternalError};


// 解压缩tar.gz文件
pub fn extract_tar_gz<P>(gz_path: P, extract_path: P)
                         -> Result<(), FileCheckError>
    where
        P: AsRef<Path> {
    let file = File::open(&gz_path).unwrap();
    let dec = GzDecoder::new(file);
    let hash_u8 = dec
        .header()
        .ok_or_else(|| FileCheckError::SplitFileError)?
        .comment()
        .ok_or_else(|| FileCheckError::SplitFileError)?;
    let hash = String::from_utf8(Vec::from(hash_u8))?;
    let mut archive = Archive::new(dec);
    let mut file_vec: Vec<PathBuf> = Vec::new();
    for entry in archive.entries().unwrap() {
        let entry = entry.unwrap();
        let file_pathbuf = entry.path().unwrap().into_owned();
        file_vec.push(file_pathbuf);
    }
    if file_vec.len() != 1 {
        return Err(FileCheckError::SplitFileError);
    }

    let file_upk = File::open(&gz_path).unwrap();
    let dec_upk = GzDecoder::new(file_upk);
    let mut archive_upk = Archive::new(dec_upk);

    archive_upk.unpack(&extract_path).unwrap();

    let mut tar_path = extract_path.as_ref().to_path_buf();
    let filename = file_vec
        .get(0)
        .ok_or_else(|| InternalError::KeyError { key: format!("0") })
        .unwrap();
    tar_path.push(filename);
    let real_hash = fetch_file_sha256(tar_path);
    if hash != real_hash {
        FileCheckError::HashCheckError { right: hash, real: real_hash };
    }
    Ok(())
}

// 解压缩tar文件
pub fn extract_tar<P>(tar_path: P, extract_path: P)
    where
        P: AsRef<Path> {
    let file = File::open(tar_path).unwrap();
    let mut archive = Archive::new(&file);
    archive.set_preserve_permissions(false);
    archive.unpack(extract_path).unwrap();
}

// 压缩tar.gz文件
pub fn compress_tar_gz<P>(gz_path: P, file_path: P, compress_level: u8)
    where
        P: AsRef<Path> {
    let gz_file = File::create(gz_path).unwrap();
    let enc = GzBuilder::new()
        .comment(fetch_file_sha256(&file_path))
        .write(gz_file, Compression::new(compress_level.into()));

    let mut tar = tar::Builder::new(enc);
    tar.mode(tar::HeaderMode::Deterministic);
    let mut file = File::open(&file_path).unwrap();
    let filename = file_path
        .as_ref()
        .file_name()
        .ok_or_else(|| InternalError::FilePathError { path: file_path.as_ref().to_path_buf() })
        .unwrap();
    tar.append_file(filename, &mut file).unwrap();
}

fn iter_child_path(entry: &DirEntry) -> PathBuf {
    // mask name of split
    let mut depth = entry.depth() - 1;
    let mut mid_list: Vec<&OsStr> = vec![entry.file_name()];
    let mut mid_path = entry.path();
    while depth > 0 {
        mid_path = mid_path.parent().unwrap();
        let mid_name = mid_path.file_name().unwrap();
        mid_list.push(mid_name);
        depth -= 1;
    }
    let mut child_name: PathBuf = mid_list.pop().unwrap().into();
    while mid_list.len() > 0 {
        let mid_name = mid_list.pop().unwrap();
        child_name.push(mid_name);
    }
    child_name
}

// 压缩tar文件
// std::io::Error
pub fn compress_tar<P>(tar_path: P, extract_path: P) -> Result<(), FileCheckError>
    where
        P: AsRef<Path> {
    let file = File::create(tar_path).unwrap();
    let mut tar = tar::Builder::new(file);
    tar.mode(tar::HeaderMode::Deterministic);
    let all_extracted_paths = WalkDir::new(extract_path)
        .sort_by_key(|item: &DirEntry| item.clone().into_path());
    for entry in all_extracted_paths {
        let entry = entry.unwrap();
        let item_path = entry.path();
        let item_name: PathBuf;
        if entry.depth() == 0 {
            continue;
        } else if entry.depth() <= 2 {
            item_name = iter_child_path(&entry);
        } else {
            let path = entry.path().to_str().unwrap_or_default().to_string();
            return Err(FileCheckError::TooManyDepthError { path });
        }
        tar.append_path_with_name(item_path, item_name).unwrap();
    }
    Ok(())
}

// 初始化工作路径
pub fn init_path(handle_path: &Path, out_path: &Path) {
    if handle_path.exists() {
        if handle_path.is_dir()
        {
            fs::remove_dir_all(handle_path).unwrap();
        } else if handle_path.is_file() {
            fs::remove_file(handle_path).unwrap();
        }
    }
    if out_path.exists() {
        if out_path.is_dir()
        {
            fs::remove_dir_all(out_path).unwrap();
        } else if out_path.is_file() {
            fs::remove_file(out_path).unwrap();
        }
    }
    let mut ret = fs::create_dir(handle_path);
    while ret.is_err() {
        ret = fs::create_dir(handle_path);
    }
    let mut ret = fs::create_dir(out_path);
    while ret.is_err() {
        ret = fs::create_dir(out_path);
    }

    let mut split_path = handle_path.clone().to_path_buf();
    split_path.push("split");

    let mut merge_path = handle_path.clone().to_path_buf();
    merge_path.push("merge");

    fs::create_dir(split_path).unwrap();
    fs::create_dir(merge_path).unwrap();
}

// 读取configP
// JsonValueError
pub fn load_config<P>(config_path: P) -> Result<JsonValue, json::JsonError>
    where
        P: AsRef<Path> {
    let contents = read_to_string(config_path).unwrap();
    Ok(json::parse(contents.as_str())?)
}

// 写入config
pub fn dump_config<P>(config: JsonValue, config_path: P)
    where
        P: AsRef<Path> {
    write(config_path, config.dump()).unwrap();
}

// 验证文件sha256
pub fn fetch_file_sha256<P>(path: P) -> String
    where
        P: AsRef<Path> {
    let mut file = File::open(path).unwrap();
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256).unwrap();
    let hash_result = sha256.finalize();
    let real_hash = format!("{:x}", hash_result);
    real_hash
}

// 验证字符串sha256
pub fn fetch_string_sha256(target_str: &String) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(target_str);
    let hash_result = sha256.finalize();
    let real_hash = format!("{:x}", hash_result);
    real_hash
}

// 计算层叠id
pub fn get_stack_id(last_stack_id: &String, parent_id: &String) -> String {
    let mut raw_id = last_stack_id.clone();
    raw_id.push_str("\n");
    raw_id.push_str(&parent_id);
    fetch_string_sha256(&raw_id)
}
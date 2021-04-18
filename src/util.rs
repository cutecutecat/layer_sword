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

use crate::errors::{FileCheckError, InternalError, raise, report, report_err, GENERATE_PATH};
use crate::errors::InternalError::{TooLargeConfigSizeError, VecEmptyError, FilePathError};


// 解压缩tar.gz文件
pub fn extract_tar_gz<P>(gz_path: P, extract_path: P)
                         -> Result<(), FileCheckError>
    where
        P: AsRef<Path> {
    let file = raise(File::open(&gz_path));
    let dec = GzDecoder::new(file);
    let hash_u8 = dec
        .header()
        .ok_or_else(|| FileCheckError::SplitFileError)?
        .comment()
        .ok_or_else(|| FileCheckError::SplitFileError)?;
    let hash = raise(String::from_utf8(Vec::from(hash_u8)));
    let mut archive = Archive::new(dec);
    let mut file_vec: Vec<PathBuf> = Vec::new();
    for entry in archive
        .entries()
        .map_err(|e| { report_err(e, FileCheckError::SplitFileError) })? {
        let entry = entry
            .map_err(|e| report_err(e, FileCheckError::SplitFileError))?;
        let file_pathbuf = raise(entry.path()).into_owned();
        file_vec.push(file_pathbuf);
    }
    if file_vec.len() != 1 {
        return Err(FileCheckError::SplitFileError);
    }

    let file_upk = raise(File::open(&gz_path));
    let dec_upk = GzDecoder::new(file_upk);
    let mut archive_upk = Archive::new(dec_upk);

    raise(archive_upk.unpack(&extract_path));

    let mut tar_path = extract_path.as_ref().to_path_buf();
    let filename = raise(file_vec
        .get(0)
        .ok_or_else(|| InternalError::KeyError { key: format!("0") }));
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
    let file = raise(File::open(tar_path));
    let mut archive = Archive::new(&file);
    archive.set_preserve_permissions(false);
    raise(archive.unpack(extract_path));
}

// 压缩tar.gz文件
pub fn compress_tar_gz<P>(gz_path: P, file_path: P, compress_level: u8)
    where
        P: AsRef<Path> {
    let gz_file = raise(File::create(gz_path));
    let enc = GzBuilder::new()
        .comment(fetch_file_sha256(&file_path))
        .write(gz_file, Compression::new(compress_level.into()));

    let mut tar = tar::Builder::new(enc);
    tar.mode(tar::HeaderMode::Deterministic);
    let mut file = raise(File::open(&file_path));
    let filename = raise(file_path
        .as_ref()
        .file_name()
        .ok_or_else(|| InternalError::FilePathError { path: file_path.as_ref().to_path_buf() }));
    raise(tar.append_file(filename, &mut file));
}

fn iter_child_path(entry: &DirEntry) -> PathBuf {
    // mask name of split
    let mut depth = entry.depth() - 1;
    let mut mid_list: Vec<&OsStr> = vec![entry.file_name()];
    let mut mid_path = entry.path();
    while depth > 0 {
        mid_path = raise(mid_path
            .parent()
            .ok_or_else(|| FilePathError { path: mid_path.into() }));
        let mid_name = raise(mid_path
            .file_name()
            .ok_or_else(|| FilePathError { path: mid_path.into() }));
        mid_list.push(mid_name);
        depth -= 1;
    }
    let mut child_name: PathBuf = raise(mid_list.pop().ok_or_else(|| VecEmptyError)).into();
    while mid_list.len() > 0 {
        let mid_name = raise(mid_list.pop().ok_or_else(|| VecEmptyError));
        child_name.push(mid_name);
    }
    child_name
}

// 压缩tar文件
// std::io::Error
pub fn compress_tar<P>(tar_path: P, extract_path: P) -> Result<(), FileCheckError>
    where
        P: AsRef<Path> {
    let file = raise(File::create(tar_path));
    let mut tar = tar::Builder::new(file);
    tar.mode(tar::HeaderMode::Deterministic);
    let all_extracted_paths = WalkDir::new(extract_path)
        .sort_by_key(|item: &DirEntry| item.clone().into_path());
    for entry in all_extracted_paths {
        let entry = raise(entry);
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
        raise(tar.append_path_with_name(item_path, item_name));
    }
    Ok(())
}

// 初始化工作路径
pub fn init_path(handle_path: &Path, out_path: &Path) {
    // set path for error clean work
    let mut path_writer = raise(GENERATE_PATH.write());
    path_writer.push(handle_path.to_path_buf());
    path_writer.push(out_path.to_path_buf());

    for path in vec![handle_path, out_path] {
        if path.exists() {
            if path.is_dir()
            {
                raise(fs::remove_dir_all(path));
            } else if path.is_file() {
                raise(fs::remove_file(path));
            }
        }
        let mut ret = fs::create_dir(path);
        // try several times waiting for remove finished by os
        let mut try_times: u8 = 0;
        while ret.is_err() {
            try_times += 1;
            ret = fs::create_dir(path);
            if try_times > 10 {
                raise(ret);
                return;
            }
        }
    }
    let mut split_path = handle_path.clone().to_path_buf();
    split_path.push("split");

    let mut merge_path = handle_path.clone().to_path_buf();
    merge_path.push("merge");

    raise(fs::create_dir(split_path));
    raise(fs::create_dir(merge_path));
}

// 读取configP
// JsonValueError
pub fn load_config<P>(config_path: P) -> Result<JsonValue, FileCheckError>
    where
        P: AsRef<Path> + Copy {
    let file = report(
        File::open(config_path),
        FileCheckError::ConfigFileError)?;
    let metadata = report(file.metadata(), FileCheckError::ConfigFileError)?;
    let size = metadata.len();
    // raise error if file size > 1MB
    if size > 1048576 {
        return Err(report_err(TooLargeConfigSizeError {
            path: config_path.as_ref().to_path_buf(),
            size: size as usize,
        }, FileCheckError::ConfigFileError));
    }
    let contents = report(
        read_to_string(config_path),
        FileCheckError::ConfigFileError)?;
    let ret = report(
        json::parse(contents.as_str()),
        FileCheckError::ConfigFileError)?;
    Ok(ret)
}

// 写入config
pub fn dump_config<P>(config: JsonValue, config_path: P)
    where
        P: AsRef<Path> {
    raise(write(config_path, config.dump()));
}

// 验证文件sha256
pub fn fetch_file_sha256<P>(path: P) -> String
    where
        P: AsRef<Path> {
    let mut file = raise(File::open(path));
    let mut sha256 = Sha256::new();
    raise(io::copy(&mut file, &mut sha256));
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

#[macro_export]
macro_rules! path_to_string {
    ($p:expr) => {
        raise($p.into_os_string().into_string())
    };
}

#[macro_export]
macro_rules! os_str_to_string {
    ($p:expr) => {
        raise($p.to_os_string().into_string())
    };
}
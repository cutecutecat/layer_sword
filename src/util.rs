use std::fs::{File, read_to_string, write, read_dir};
use std::{io, fs};
use std::path::{Path, PathBuf};

use tar::Archive;
use sha2::{Sha256, Digest};
use json::JsonValue;

use flate2::{Compression, GzBuilder};
use flate2::read::GzDecoder;


use crate::errors::{Error, Result};


// 解压缩tar.gz文件
pub fn extract_tar_gz<P>(gz_path: P, extract_path: P)
                         -> Result<()>
    where
        P: AsRef<Path> {
    if gz_path.as_ref().extension().unwrap_or_default() != "gz" {
        Error::ConvertError();
    }
    let file = File::open(&gz_path)?;
    let dec = GzDecoder::new(file);
    let hash_u8 = dec
        .header()
        .ok_or_else(|| Error::SplitFileError())?
        .comment()
        .ok_or_else(|| Error::SplitFileError())?;
    let hash = String::from_utf8(Vec::from(hash_u8))?;
    let mut archive = Archive::new(dec);
    let mut file_vec: Vec<PathBuf> = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let file_pathbuf = entry.path()?.into_owned();
        file_vec.push(file_pathbuf);
    }
    if file_vec.len() != 1 {
        Error::SplitFileError();
    }

    let file_upk = File::open(&gz_path)?;
    let dec_upk = GzDecoder::new(file_upk);
    let mut archive_upk = Archive::new(dec_upk);

    archive_upk.unpack(&extract_path)?;

    let mut tar_path = extract_path.as_ref().to_path_buf();
    let filename = file_vec
        .get(0)
        .ok_or_else(|| Error::KeyError { key: format!("0") })?;
    tar_path.push(filename);
    let real_hash = fetch_file_sha256(tar_path)?;
    if hash != real_hash {
        Error::HashCheckError { right: hash, real: real_hash };
    }
    Ok(())
}

// 解压缩tar文件
pub fn extract_tar<P>(tar_path: P, extract_path: P) -> Result<()>
    where
        P: AsRef<Path> {
    if tar_path.as_ref().extension().unwrap_or_default() != "tar" {
        Error::ConvertError();
    }
    let file = File::open(tar_path)?;
    let mut archive = Archive::new(&file);
    archive.set_preserve_permissions(false);
    archive.unpack(extract_path)?;
    Ok(())
}

// 压缩tar.gz文件
pub fn compress_tar_gz<P>(gz_path: P, file_path: P, compress_level: u8)
                          -> Result<()>
    where
        P: AsRef<Path> {
    if gz_path.as_ref().extension().unwrap_or_default() != "gz" {
        Error::ConvertError();
    }
    let gz_file = File::create(gz_path)?;
    let enc = GzBuilder::new()
        .comment(fetch_file_sha256(&file_path)?)
        .write(gz_file, Compression::new(compress_level as u32));

    let mut tar = tar::Builder::new(enc);
    tar.mode(tar::HeaderMode::Deterministic);
    let mut file = File::open(&file_path)?;
    let filename = file_path
        .as_ref()
        .file_name()
        .ok_or_else(|| Error::FilePathError { path: file_path.as_ref().to_path_buf() })?;
    tar.append_file(filename, &mut file)?;
    Ok(())
}

// 压缩tar文件
pub fn compress_tar<P>(tar_path: P, extract_path: P) -> Result<()>
    where
        P: AsRef<Path> {
    if tar_path.as_ref().extension().unwrap_or_default() != "tar" {
        Error::ConvertError();
    }
    let file = File::create(tar_path)?;
    let mut tar = tar::Builder::new(file);
    tar.mode(tar::HeaderMode::Deterministic);
    let all_extracted_paths = read_dir(extract_path)?;
    for entry in all_extracted_paths {
        let entry = entry?;
        let item_pathbuf = entry.path();
        let item_name = entry.file_name();
        if item_pathbuf.is_dir() {
            tar.append_dir_all(item_name, item_pathbuf)?;
        } else {
            tar.append_path_with_name(item_pathbuf, item_name)?;
        }
    }
    Ok(())
}

// 初始化工作路径
pub fn init_path(handle_path: &Path) -> Result<()> {
    if handle_path.exists() {
        if handle_path.is_dir()
        {
            fs::remove_dir_all(handle_path)?;
        } else if handle_path.is_file() {
            fs::remove_file(handle_path)?;
        }
    }
    let mut ret = fs::create_dir(handle_path);
    while ret.is_err() {
        ret = fs::create_dir(handle_path);
    }

    let mut split_path = handle_path.clone().to_path_buf();
    split_path.push("split");

    let mut merge_path = handle_path.clone().to_path_buf();
    merge_path.push("merge");

    fs::create_dir(split_path)?;
    fs::create_dir(merge_path)?;
    Ok(())
}

// 读取configP
pub fn load_config<P>(config_path: P) -> Result<JsonValue>
    where
        P: AsRef<Path> {
    let contents = read_to_string(config_path)?;
    Ok(json::parse(contents.as_str())?)
}

// 写入config
pub fn dump_config<P>(config: JsonValue, config_path: P) -> Result<()>
    where
        P: AsRef<Path> {
    write(config_path, config.dump())?;
    Ok(())
}

// 验证文件sha256
pub fn fetch_file_sha256<P>(path: P) -> Result<String>
    where
        P: AsRef<Path> {
    let mut file = File::open(path)?;
    let mut sha256 = Sha256::new();
    io::copy(&mut file, &mut sha256)?;
    let hash_result = sha256.finalize();
    let real_hash = format!("{:x}", hash_result);
    Ok(real_hash)
}

// 验证字符串sha256
pub fn fetch_string_sha256(target_str: &String) -> Result<String> {
    let mut sha256 = Sha256::new();
    sha256.update(target_str);
    let hash_result = sha256.finalize();
    let real_hash = format!("{:x}", hash_result);
    Ok(real_hash)
}

// 计算层叠id
pub fn get_stack_id(last_stack_id: &String, parent_id: &String) -> Result<String> {
    let mut raw_id = last_stack_id.clone();
    raw_id.push_str("\n");
    raw_id.push_str(&parent_id);
    Ok(fetch_string_sha256(&raw_id)?)
}
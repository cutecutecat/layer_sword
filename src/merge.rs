use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

use fs_extra::{dir, file};

use crate::util::{extract_tar, load_config, fetch_file_sha256, get_stack_id, compress_tar, extract_tar_gz};
use crate::inspector::inspect;
use crate::errors::{FileCheckError, InternalError};

#[derive(Debug)]
struct SplitConfig {
    parent_id: String,
    stack_id: String,
    dir_path: String,
    tar_path: String,
    config_path: String,
}

// 获得路径下全部文件并筛选出tar
fn extract_to_tar(target_path: &Path, work_path: &Path) -> Result<Vec<PathBuf>, FileCheckError> {
    let all_extracted_paths = fs::read_dir(target_path).unwrap();

    for entry in all_extracted_paths {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() { continue; }
        if path.extension().unwrap_or_default() != "gz" {
            return Err(FileCheckError::FileExtensionError {
                extension: format!("gz"),
                path: entry.path(),
            });
        }
        extract_tar_gz(path, work_path.to_path_buf())?;
    }

    let mut tar_vec: Vec<PathBuf> = Vec::new();
    let all_extracted_paths = fs::read_dir(work_path).unwrap();

    for entry in all_extracted_paths {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() { continue; }
        if path.extension().unwrap_or_default() != "tar" { continue; }
        tar_vec.push(entry.path());
    }
    Ok(tar_vec)
}

// 新建文件夹分别解压缩
fn extract_to_directory(tar_vec: Vec<PathBuf>, split_path: &Path)
                        -> Result<HashMap<usize, SplitConfig>, FileCheckError> {
    let mut split_config_map: HashMap<usize, SplitConfig> = HashMap::new();
    for tar_file in tar_vec {
        let split_name = tar_file
            .file_name()
            .ok_or_else(|| InternalError::FilePathError { path: tar_file.clone() })
            .unwrap();
        let mut dir_name = PathBuf::from(split_path);
        dir_name.push(split_name);
        fs::create_dir(&dir_name).unwrap();
        if tar_file.extension().unwrap_or_default() != "tar" {
            return Err(FileCheckError::FileExtensionError {
                extension: format!("tar"),
                path: tar_file.clone(),
            });
        }
        extract_tar(&tar_file, &dir_name);

        let mut config_pathbuf = PathBuf::from(split_path);
        config_pathbuf.push(split_name);
        config_pathbuf.push("split_config.json");
        let split_config = load_config(&config_pathbuf)
            .map_err(|_| FileCheckError::ConfigFileError)?;
        let config_path_string = config_pathbuf
            .into_os_string()
            .into_string()
            .map_err(|_| InternalError::ConvertError)
            .unwrap();
        let mut config_body: SplitConfig = SplitConfig {
            parent_id: format!(""),
            stack_id: format!(""),
            dir_path: format!(""),
            tar_path: format!(""),
            config_path: config_path_string,
        };
        config_body.parent_id = split_config["parent_id"].to_string();
        config_body.stack_id = split_config["stack_id"].to_string();
        config_body.dir_path = dir_name
            .into_os_string()
            .into_string()
            .map_err(|_| InternalError::ConvertError)
            .unwrap();
        config_body.tar_path = tar_file
            .into_os_string()
            .into_string()
            .map_err(|_| InternalError::ConvertError)
            .unwrap();
        let index = split_config["index"]
            .as_usize()
            .ok_or(InternalError::ConvertError)
            .unwrap();
        if split_config_map.contains_key(&index) {
            return Err(FileCheckError::SplitsUnmatchedError { index });
        }
        split_config_map.insert(index, config_body);
    }
    Ok(split_config_map)
}

// 验证分割哈希
fn merge_checked_files(split_config_map: HashMap<usize, SplitConfig>, merge_path: &Path)
                       -> Result<(), FileCheckError> {
    let mut parent_id = String::new();
    let mut stack_id = String::new();

    let mut copy_options_file = file::CopyOptions::new();
    copy_options_file.overwrite = true;

    let mut copy_options_dir = dir::CopyOptions::new();
    copy_options_dir.overwrite = true;
    copy_options_dir.copy_inside = true;

    for i in 0..split_config_map.len() {
        let config_body = split_config_map
            .get(&i)
            .ok_or_else(|| InternalError::KeyError { key: i.to_string() })
            .unwrap();
        let tar_hash = fetch_file_sha256(&config_body.tar_path);
        stack_id = get_stack_id(&stack_id, &parent_id);
        if config_body.parent_id != parent_id {
            return Err(FileCheckError::HashCheckError {
                right: config_body.parent_id.clone(),
                real: parent_id.clone(),
            });
        }
        if config_body.stack_id != stack_id {
            return Err(FileCheckError::HashCheckError {
                right: config_body.stack_id.clone(),
                real: stack_id.clone(),
            });
        }
        parent_id = tar_hash;
        fs::remove_file(&config_body.config_path).unwrap();
        let all_extracted_paths = fs::read_dir(&config_body.dir_path).unwrap();
        for entry in all_extracted_paths {
            let entry = entry.unwrap();
            let item_name = entry.file_name();

            let item_name_str = item_name
                .as_os_str()
                .to_str()
                .ok_or_else(|| InternalError::ConvertError)
                .unwrap();

            let mut item_pathbuf = PathBuf::from(&config_body.dir_path);
            item_pathbuf.push(item_name_str);

            let mut dst_pathbuf = merge_path.clone().to_path_buf();
            dst_pathbuf.push(item_name);
            if item_pathbuf.is_dir() {
                dir::copy(item_pathbuf, dst_pathbuf, &copy_options_dir).unwrap();
            } else {
                file::copy(item_pathbuf, dst_pathbuf, &copy_options_file).unwrap();
            }
        }
    }
    Ok(())
}

pub fn merge_layer(target_path: &Path, work_path: &Path, out_path: &Path)
                   -> Result<(), FileCheckError> {
    let mut merge_pathbuf = work_path.to_path_buf();
    merge_pathbuf.push("merge");
    let mut split_pathbuf = work_path.to_path_buf();
    split_pathbuf.push("split");
    let mut tar_pathbuf = out_path.to_path_buf();
    tar_pathbuf.push("merge.tar");

    log::info!("Extracting tar.gz file to tar from file under '{}'",
               &target_path.to_str().ok_or_else(|| InternalError::ConvertError).unwrap());
    let tar_vec = extract_to_tar(target_path, work_path)?;
    log::info!("Extracting tar file to directories");
    let split_config_map =
        extract_to_directory(tar_vec, &split_pathbuf)?;
    log::info!("Merging split directories and check split hash");
    merge_checked_files(split_config_map, &merge_pathbuf)?;
    log::info!("Checking merged dock image files");
    log::info!("[inspect begin]");
    inspect(&merge_pathbuf)?;
    log::info!("[inspect end]");
    log::info!("Compressing merged dock image files to tar file");
    compress_tar(&tar_pathbuf, &merge_pathbuf)?;
    log::info!("Cleaning items inside work path");
    fs::remove_dir_all(work_path).unwrap();
    Ok(())
}
use std::path::{Path, PathBuf};
use std::fs;

use fs_extra::{dir, file};

use crate::split::Split;
use crate::dominator::Config;
use crate::inspector::Inspect;
use crate::path_to_string;
use crate::util::{extract_tar, load_config, compress_tar, extract_tar_gz};
use crate::errors::{FileCheckError, InternalError, raise};

pub trait Merge: Split {
    // 获得路径下全部文件并筛选出tar
    fn extract_to_tar(&self, target_path: &Path, work_path: &Path) -> Result<Vec<PathBuf>, FileCheckError> {
        let all_extracted_paths = raise(fs::read_dir(target_path));

        for entry in all_extracted_paths {
            let entry = raise(entry);
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
        let all_extracted_paths = raise(fs::read_dir(work_path));

        for entry in all_extracted_paths {
            let entry = raise(entry);
            let path = entry.path();
            if path.is_dir() { continue; }
            if path.extension().unwrap_or_default() != "tar" { continue; }
            tar_vec.push(entry.path());
        }
        Ok(tar_vec)
    }

    // 新建文件夹分别解压缩
    fn extract_to_directory(&self, tar_vec: Vec<PathBuf>, split_path: &Path)
                            -> Result<Vec<Box<dyn Config>>, FileCheckError> {
        let mut split_config_vec: Vec<Box<dyn Config>> = Vec::new();
        for tar_file in tar_vec {
            let split_name = raise(tar_file
                .file_name()
                .ok_or_else(|| InternalError::FilePathError { path: tar_file.clone() }));
            let mut dir_name = PathBuf::from(split_path);
            dir_name.push(split_name);
            raise(fs::create_dir(&dir_name));
            if tar_file.extension().unwrap_or_default() != "tar" {
                return Err(FileCheckError::FileExtensionError {
                    extension: format!("tar"),
                    path: tar_file.clone(),
                });
            }
            extract_tar(&tar_file, &dir_name);
            let mut config_path = PathBuf::from(split_path);
            config_path.push(split_name);
            config_path.push("split_config.json");
            let json_config = load_config(&config_path)
                .map_err(|_| FileCheckError::ConfigFileError)?;
            let mut split_config = self.init_config();
            split_config.load_json(json_config);
            split_config.set_path(
                path_to_string!(dir_name),
                path_to_string!(tar_file),
                path_to_string!(config_path));
            split_config_vec.push(split_config);
        }
        split_config_vec.sort_unstable_by_key(|c| c.key());
        for (i, split) in split_config_vec.iter().enumerate() {
            if i != split.key() {
                return Err(FileCheckError::SplitsUnmatchedError { index: split.key() });
            }
        }
        Ok(split_config_vec)
    }


    // 验证分割哈希
    fn check_all_splits(&self, split_config_vec: Vec<Box<dyn Config>>)
                        -> Result<Vec<String>, FileCheckError> {
        let mut stack_id = String::new();
        let mut parent_id = String::new();

        let mut dir_path_vec: Vec<String> = Vec::new();

        for config_body in split_config_vec.iter() {
            let (now_stack_id, now_id) = self.check_with_config(
                config_body,
                stack_id,
                parent_id)?;
            parent_id = now_id;
            stack_id = now_stack_id;
            raise(fs::remove_file(config_body.get_config()));
            dir_path_vec.push(config_body.get_dir());
        }
        Ok(dir_path_vec)
    }

    fn merge_checked_files(&self,
                           dir_path_vec: Vec<String>,
                           merge_path: &Path) {
        let mut copy_options_file = file::CopyOptions::new();
        copy_options_file.overwrite = true;

        let mut copy_options_dir = dir::CopyOptions::new();
        copy_options_dir.overwrite = true;
        copy_options_dir.copy_inside = true;

        for dir_path in dir_path_vec.iter() {
            let all_extracted_paths = raise(fs::read_dir(dir_path));
            for entry in all_extracted_paths {
                let entry = raise(entry);
                let item_name = entry.file_name();

                let item_name_str = raise(item_name
                    .as_os_str()
                    .to_str()
                    .ok_or_else(|| InternalError::ConvertError));

                let mut item_pathbuf = PathBuf::from(dir_path);
                item_pathbuf.push(item_name_str);

                let mut dst_pathbuf = merge_path.clone().to_path_buf();
                dst_pathbuf.push(item_name);
                if item_pathbuf.is_dir() {
                    raise(dir::copy(
                        item_pathbuf,
                        dst_pathbuf,
                        &copy_options_dir));
                } else {
                    raise(file::copy(
                        item_pathbuf,
                        dst_pathbuf,
                        &copy_options_file));
                }
            }
        }
    }

    fn merge_layer(&self,
                   inspector: Box<dyn Inspect>,
                   target_path: &Path,
                   work_path: &Path,
                   out_path: &Path)
                   -> Result<(), FileCheckError> {
        let mut merge_pathbuf = work_path.to_path_buf();
        merge_pathbuf.push("merge");
        let mut split_pathbuf = work_path.to_path_buf();
        split_pathbuf.push("split");
        let mut tar_pathbuf = out_path.to_path_buf();
        tar_pathbuf.push("merge.tar");

        log::info!("Extracting tar.gz file to tar from file under '{}'",
                   raise(target_path.to_str().ok_or_else(|| InternalError::ConvertError)));
        let tar_vec = self.extract_to_tar(target_path, work_path)?;
        log::info!("Extracting tar file to directories");
        let split_config_vec: Vec<Box<dyn Config>> =
            self.extract_to_directory(tar_vec, &split_pathbuf)?;
        log::info!("Check split hash for all the splits");
        let dir_path_vec = self.check_all_splits(split_config_vec)?;
        log::info!("Merging split directories and check split hash");
        self.merge_checked_files(dir_path_vec, &merge_pathbuf);
        log::info!("Checking merged dock image files");
        log::info!("[inspect begin]");
        inspector.inspect(&merge_pathbuf)?;
        log::info!("[inspect end]");
        log::info!("Compressing merged dock image files to tar file");
        compress_tar(&tar_pathbuf, &merge_pathbuf)?;
        log::info!("Cleaning items inside work path");
        raise(fs::remove_dir_all(work_path));
        Ok(())
    }

    fn check_with_config(&self,
                         config_body: &Box<dyn Config>,
                         stack_id: String,
                         parent_id: String)
                         -> Result<(String, String), FileCheckError>;

    fn init_config(&self) -> Box<dyn Config>;
}
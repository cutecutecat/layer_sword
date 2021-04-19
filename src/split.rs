use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use fs_extra::{dir, file};

use crate::inspector::Inspect;
use crate::os_str_to_string;
use crate::util::{compress_tar, compress_tar_gz, extract_tar};
use crate::errors::{FileCheckError, InternalError, raise, raise_err, raise_debug};

pub trait Split {
    /// deduct item -1 to real value from splits number of layer
    fn deduct_split_map(&self,
                        split_names: &Vec<String>,
                        split_map: HashMap<String, i16>,
                        layer_dir_set: &Vec<PathBuf>)
                        -> Result<HashMap<String, i16>, FileCheckError> {
        let mut deduct_map = split_map;
        let mut layer_num: i16 = 0;
        let mut addup_flag: String = String::new();
        for name in split_names {
            let split_num: &i16 = raise(deduct_map
                .get(name)
                .ok_or_else(|| InternalError::KeyError { key: name.clone() }));
            if *split_num == -1 {
                if addup_flag == "" {
                    addup_flag = name.clone();
                } else {
                    raise_err(InternalError::ImpossibleError {
                        msg: format!("more than 1 split of -1 layers: '{}' and '{}'",
                                     addup_flag, name),
                    });
                }
            } else if *split_num < -1
            {
                raise_err(InternalError::ImpossibleError {
                    msg: format!("split number cannot be positive or -1, actuall '{:?}'",
                                 split_num),
                });
            } else {
                layer_num += *split_num as i16;
            }
        }
        // deal with addup layer of -1 flag
        if addup_flag == "" {
            if (layer_dir_set.len() as i16) != layer_num {
                return Err(FileCheckError::BadDockerFileError {
                    msg: format!("layers per split(without deduct) sum to {}, \
                                not equal to real layers {}",
                                 layer_num, layer_dir_set.len()),
                });
            }
        } else {
            if layer_num > (layer_dir_set.len() - 1) as i16 {
                return Err(FileCheckError::BadDockerFileError {
                    msg: format!("layers per split(with deduct) sum to {}, \
                                larger than real layers {}",
                                 layer_num, layer_dir_set.len()),
                });
            }
            deduct_map.insert(addup_flag, layer_dir_set.len() as i16 - layer_num);
        }
        Ok(deduct_map)
    }

    /// copy directories of layers inside image
    fn copy_split_directories(&self,
                              split_names: &Vec<String>,
                              split_map: &HashMap<String, i16>,
                              layer_dir_set: &Vec<PathBuf>,
                              top_path: &PathBuf) {
        let mut id_from: i16 = 0;

        let mut copy_options_dir = dir::CopyOptions::new();
        copy_options_dir.overwrite = true;
        copy_options_dir.copy_inside = true;

        for name in split_names {
            let mut split_path = top_path.clone();
            split_path.push(name.clone());
            raise(fs::create_dir(&split_path));

            for id in 0..split_map[name] {
                let src_path = raise(layer_dir_set
                    .get((id_from + id) as usize)
                    .ok_or_else(|| InternalError::KeyError { key: (id_from + id).to_string() }));
                let item_name = raise(src_path
                    .file_name()
                    .ok_or_else(|| InternalError::FilePathError { path: src_path.clone() }));
                let mut dst_path = split_path.clone();
                dst_path.push(item_name);
                raise(dir::copy(src_path, dst_path, &copy_options_dir));
            }
            id_from += split_map[name];
        }
    }

    /// copy files inside image
    fn copy_split_files(&self,
                        split_names: &Vec<String>,
                        file_map: HashMap<String, PathBuf>,
                        top_pathbuf: &PathBuf) {
        let mut copy_options_file = file::CopyOptions::new();
        copy_options_file.overwrite = true;
        let top_layer = raise(split_names
            .get(split_names.len() - 1)
            .ok_or_else(|| InternalError::KeyError { key: (split_names.len() - 1).to_string() }));
        for (_, src_path) in file_map {
            let filename = raise(src_path
                .file_name()
                .ok_or_else(|| InternalError::FilePathError { path: src_path.clone() }));
            let filename = os_str_to_string!(filename);
            let mut dst_pathbuf = top_pathbuf.clone();
            dst_pathbuf.push(top_layer);
            dst_pathbuf.push(filename);
            raise(file::copy(&src_path, &dst_pathbuf, &copy_options_file));
        }
    }

    /// compress a directory into a tar file
    fn pack_into_tar(&self,
                     split_path: &PathBuf, name: &String)
                     -> Result<PathBuf, FileCheckError> {
        let mut tar_path = split_path.clone();
        tar_path.pop();
        tar_path.push(name.as_str().to_owned() + ".tar");

        let mut compress_path = split_path.clone();
        compress_path.push(name);
        compress_tar(&tar_path, &compress_path)?;
        raise(fs::remove_dir_all(compress_path));
        Ok(tar_path)
    }

    /// compress all splits into tar file
    fn pack_all_tar(&self,
                    split_names: &Vec<String>,
                    split_path: PathBuf) -> Result<Vec<PathBuf>, FileCheckError> {
        let mut parent_id = String::new();
        let mut stack_id = String::new();
        let mut tar_path_vec: Vec<PathBuf> = Vec::new();
        for (i, name) in split_names.iter().enumerate() {
            let (tar_path, now_stack_id, now_id) =
                self.pack_tar_with_config(i,
                                          name,
                                          &split_path,
                                          stack_id,
                                          parent_id)?;
            parent_id = now_id;
            stack_id = now_stack_id;

            tar_path_vec.push(tar_path);
        }
        Ok(tar_path_vec)
    }

    /// compress all splits into tar.gz file
    fn pack_all_gz(&self, out_path: &PathBuf, tar_path_vec: Vec<PathBuf>, compress_level: u8) {
        for tar_path in tar_path_vec.iter() {
            let mut gz_path = out_path.clone();
            gz_path.push(tar_path.file_name().unwrap_or_default());
            gz_path.set_extension("tar.gz");
            compress_tar_gz(&gz_path, &tar_path, compress_level);
        }
    }

    /// function called for a whole split procedure
    fn split_layer(&self,
                   inspector: Box<dyn Inspect>,
                   tar_path: &Path,
                   split_names: Vec<String>,
                   split_map: HashMap<String, i16>,
                   work_path: &Path,
                   out_path: &Path,
                   compress_level: u8)
                   -> Result<(), FileCheckError> {
        let mut extract_path = work_path.to_path_buf();
        extract_path.push("merge");
        let mut split_path = work_path.to_path_buf();
        split_path.push("split");
        log::info!("Extracting tar file of dock image at {}",
                   raise(tar_path.to_str().ok_or_else(|| InternalError::ConvertError)));
        if tar_path.extension().unwrap_or_default() != "tar" {
            return Err(FileCheckError::FileExtensionError {
                extension: format!("tar"),
                path: tar_path.to_path_buf(),
            })?;
        }
        extract_tar(tar_path, &extract_path);
        log::info!("Checking merged dock image files");
        log::info!("[inspect begin]");
        let (file_map, layer_dir_set) =
            inspector.inspect(&extract_path)?;
        log::info!("[inspect end]");
        log::info!("Validating number of each layer");
        let deduct_map =
            self.deduct_split_map(&split_names, split_map, &layer_dir_set)?;
        log::info!("Copying layer directories inside splits into dock image");
        self.copy_split_directories(&split_names, &deduct_map,
                                    &layer_dir_set, &split_path);
        log::info!("Copying files inside splits into dock image");
        self.copy_split_files(&split_names, file_map, &split_path);
        log::info!("Packing items into tar file under {}",
                   raise(out_path.to_str().ok_or_else(|| InternalError::ConvertError)));
        let tar_path_vec = self.pack_all_tar(&split_names, split_path)?;
        log::info!("Packing items into gz file under {} at compress_level {}",
                   raise(out_path.to_str().ok_or_else(|| InternalError::ConvertError)),
                   compress_level);
        self.pack_all_gz(&out_path.to_path_buf(), tar_path_vec, compress_level);
        log::info!("Clean items inside work path");
        raise(fs::remove_dir_all(work_path));
        Ok(())
    }

    /// compress one split into tar file with config of inspection info
    fn pack_tar_with_config(
        &self,
        split_index: usize,
        split_name: &String,
        split_path: &PathBuf,
        stack_id: String,
        parent_id: String)
        -> Result<(PathBuf, String, String), FileCheckError>;
}
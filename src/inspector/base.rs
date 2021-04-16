use std::path::{Path, PathBuf};
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::fs::read_dir;

use regex::Regex;
use json::JsonValue;

use crate::inspector::Inspect;
use crate::os_str_to_string;
use crate::util::{fetch_file_sha256, load_config};
use crate::errors::{FileCheckError, InternalError, raise};

pub struct BaseInspector {}

impl Inspect for BaseInspector {
    fn inspect_route(&self, extract_path: &Path) -> Result<(HashMap<String, PathBuf, RandomState>, HashSet<String, RandomState>), FileCheckError> {
        let mut file_map: HashMap<String, PathBuf> = HashMap::new();
        let mut layer_hash_set: HashSet<String> = HashSet::new();

        let mut config_num: u8 = 0;
        let mut config_path: PathBuf = PathBuf::new();
        let mut manifest_path: PathBuf = PathBuf::new();
        let mut repositories_path: PathBuf = PathBuf::new();

        let expr_config = raise(Regex::new(r#"^[a-z0-9]{64}.json$"#));
        let expr_layer_and_config = raise(Regex::new(r#"^[a-z0-9]{64}"#));

        let all_extracted_paths = raise(read_dir(extract_path));
        let mut now_path: String;
        for entry in all_extracted_paths {
            let entry = raise(entry);
            now_path = raise(entry
                .file_name()
                .into_string()
                .map_err(|_| InternalError::ConvertError));
            // 查找manifest
            if manifest_path.components().next().is_none() && now_path == "manifest.json" {
                manifest_path.push(extract_path);
                manifest_path.push(&now_path);
            }
            // 查找repositories
            if repositories_path.components().next().is_none() && now_path == "repositories" {
                repositories_path.push(extract_path);
                repositories_path.push(&now_path);
            }
            // 查找config和layer
            let judge_sha256 = expr_layer_and_config.is_match(&*now_path);
            if judge_sha256 == true {
                // this is a config
                if expr_config.is_match(&*now_path) == true {
                    if config_num > 1 {
                        return Err(FileCheckError::BadDockerFileError {
                            msg: format!("more than one config.json '{}' and '{:?}'",
                                         now_path.clone(), config_path.clone())
                        });
                    }
                    config_num += 1;
                    config_path.push(extract_path);
                    config_path.push(&now_path);
                } else {
                    // this is a layer
                    layer_hash_set.insert(now_path);
                }
            }
        }
        if config_num == 0 {
            return Err(FileCheckError::BadDockerFileError { msg: format!("no config.json") });
        } else if manifest_path.components().next().is_none() {
            return Err(FileCheckError::BadDockerFileError { msg: format!("no manifest.json") });
        } else if repositories_path.components().next().is_none() {
            return Err(FileCheckError::BadDockerFileError { msg: format!("no repositories") });
        }
        file_map.insert(format!("config_path"), config_path);
        file_map.insert(format!("manifest_path"), manifest_path);
        file_map.insert(format!("repositories_path"), repositories_path);
        Ok((file_map, layer_hash_set))
    }


    fn inspect_config(&self, file_map: &HashMap<String, PathBuf, RandomState>) -> Result<HashSet<String, RandomState>, FileCheckError> {
        let mut layer_tar_hash: HashSet<String> = HashSet::new();

        // 验证config.json自身哈希
        let config_path: &Path = raise(file_map
            .get("config_path")
            .ok_or_else(|| InternalError::KeyError { key: format!("config_path") }))
            .as_path();
        let config_filestem = raise(config_path
            .file_stem()
            .ok_or_else(|| InternalError::FilePathError { path: config_path.to_path_buf() }));
        let config_filestem = os_str_to_string!(config_filestem);
        let hash = fetch_file_sha256(config_path);
        if config_filestem != hash {
            return Err(FileCheckError::BadDockerFileError {
                msg: format!("config.json sha256 check failed \
                    \nreal:'{}'\nright:'{}'", config_filestem.clone(), hash.clone())
            });
        }
        // 验证diff_id存在并返回
        let config = load_config(config_path)
            .map_err(|_| FileCheckError::ConfigFileError)?;
        let diff_ids = &config["rootfs"]["diff_ids"];
        let diff_ids = match diff_ids {
            JsonValue::Array(ids) => { Ok(ids) }
            _ => {
                Err(FileCheckError::BadDockerFileError {
                    msg: format!("config file parse failed")
                })
            }
        }?;

        for diff_id in diff_ids {
            let diff_id = match diff_id {
                JsonValue::String(ids) => { Ok(ids) }
                _ => {
                    Err(FileCheckError::BadDockerFileError { msg: format!("config file parse failed") })
                }
            }?;
            let prefix = &diff_id[0..7];
            let hash = &diff_id[7..];
            if prefix != "sha256:" {
                return Err(FileCheckError::BadDockerFileError {
                    msg: format!("bad hash 'diff_id' prefix inside config.json) \
                \nreal:'{}'\nright:'sha256:'", prefix.clone())
                });
            }
            layer_tar_hash.insert(hash.to_string());
        }
        Ok(layer_tar_hash)
    }

    fn inspect_layer(&self, extract_path: &Path, layer_hash_set: &HashSet<String, RandomState>, config_tar_hash: &HashSet<String, RandomState>) -> Result<(), FileCheckError> {
        if config_tar_hash.len() != layer_hash_set.len() {
            return Err(FileCheckError::BadDockerFileError {
                msg: format!("layer number is different from what inside config.json\
                    \nreal:'{}'\nright:'{}'", layer_hash_set.len(), config_tar_hash.len())
            });
        }

        let mut real_tar_hash: HashSet<String> = HashSet::new();

        for layer in layer_hash_set {
            let mut layer_dir_path = extract_path.clone().to_path_buf();
            layer_dir_path.push(layer.clone());
            let layer_paths = raise(read_dir(&layer_dir_path));
            let mut has_layer = false;
            let mut has_json = false;
            for entry in layer_paths {
                let entry = raise(entry);
                let file_name = entry.file_name();
                let now_path = raise(file_name
                    .to_str()
                    .ok_or_else(|| InternalError::ConvertError));
                if now_path != "json" && now_path != "layer.tar" && now_path != "VERSION" {
                    return Err(FileCheckError::BadDockerFileError {
                        msg: format!("unrecognized file '{}' inside layer '{:?}'",
                                     now_path, layer)
                    });
                }
                if now_path == "layer.tar" {
                    let mut layer_tar_path = layer_dir_path.clone();
                    layer_tar_path.push(now_path);
                    let hash = fetch_file_sha256(layer_tar_path);
                    real_tar_hash.insert(hash);
                    has_layer = true;
                } else if now_path == "json" {
                    let mut layer_json_path = layer_dir_path.clone();
                    layer_json_path.push(now_path);
                    let config = load_config(&layer_json_path)
                        .map_err(|_| FileCheckError::ConfigFileError)?;
                    let parent_layer = match config["parent"].clone() {
                        JsonValue::String(config_item) => { Ok(config_item) }
                        JsonValue::Null => { Ok(format!("")) }
                        _ => {
                            Err(FileCheckError::BadDockerFileError {
                                msg: format!(
                                    "bad json inside layer '{:?}'", layer)
                            })
                        }
                    }?;
                    if !parent_layer.len() == 0 && !layer_hash_set.contains(&*parent_layer) {
                        return Err(FileCheckError::BadDockerFileError {
                            msg: format!("bad json inside layer '{:?}'", layer)
                        });
                    }
                    has_json = true;
                }
            }
            if !has_json {
                return Err(FileCheckError::BadDockerFileError {
                    msg: format!("no json inside layer '{:?}'", layer)
                });
            } else if !has_layer {
                return Err(FileCheckError::BadDockerFileError {
                    msg: format!("no layer.tar inside layer '{:?}'", layer)
                });
            }
        }
        // 对称差
        let error_layer: Vec<_> = config_tar_hash
            .symmetric_difference(&real_tar_hash)
            .collect();
        if error_layer.len() != 0 {
            return Err(FileCheckError::BadDockerFileError {
                msg: format!("some file 'layer.tar' sha256 is not equal to what inside config.json")
            });
        }
        Ok(())
    }

    fn inspect_manifest(&self, extract_path: &Path, file_map: &HashMap<String, PathBuf, RandomState>, layer_hash_set: &HashSet<String, RandomState>) -> Result<Vec<PathBuf>, FileCheckError> {
        // 验证manefist.json内描述文件存在
        let manifest_path = raise(
            file_map
                .get("manifest_path")
                .ok_or_else(|| InternalError::KeyError { key: format!("manifest_path") }));
        let config = load_config(manifest_path)
            .map_err(|_| FileCheckError::BadDockerFileError {
                msg: format!("manifest file parse failed")
            })?;
        let config_array = match &config {
            JsonValue::Array(config_item) => { Ok(config_item) }
            _ => {
                Err(FileCheckError::BadDockerFileError {
                    msg: format!("manifest file parse failed")
                })
            }
        }?;
        if config_array.len() != 1 {
            return Err(FileCheckError::BadDockerFileError {
                msg: format!("manifest file has {} config entries rather than 1",
                             config_array.len())
            });
        }

        let mut layer_dir_vec: Vec<PathBuf> = Vec::new();
        let config_path = match &config[0]["Config"] {
            JsonValue::String(path) => { Ok(path) }
            _ => {
                Err(FileCheckError::BadDockerFileError {
                    msg: format!("manifest file parse failed")
                })
            }
        }?;
        let mut log_config_path = extract_path.to_path_buf();
        log_config_path.push(config_path);
        if file_map["config_path"] != log_config_path {
            return Err(FileCheckError::BadDockerFileError {
                msg: format!("config file path '{:?}' is not equal to '{:?}' inside manefist.json",
                             file_map["config_path"], log_config_path)
            });
        }
        let layers = match &config[0]["Layers"] {
            JsonValue::Array(layers) => { Ok(layers) }
            _ => {
                Err(FileCheckError::BadDockerFileError {
                    msg: format!("manifest file parse failed")
                })
            }
        }?;
        for layer in layers {
            let layer_parent_path = match layer {
                JsonValue::String(layers) => { Ok(Path::new(layers)) }
                _ => {
                    Err(FileCheckError::BadDockerFileError {
                        msg: format!("manifest file parse failed")
                    })
                }
            }?;
            let layer_path = raise(
                layer_parent_path
                    .parent()
                    .ok_or_else(|| InternalError::FilePathError {
                        path: layer_parent_path.to_path_buf()
                    }));
            if !layer_hash_set.contains(
                raise(layer_path
                    .to_str()
                    .ok_or_else(|| InternalError::ConvertError))) {
                return Err(FileCheckError::BadDockerFileError {
                    msg: format!("layer inside manifest doesn't exist")
                });
            }
            let mut layer_full_path = extract_path.to_path_buf();
            layer_full_path.push(layer_path);
            layer_dir_vec.push(layer_full_path);
        }
        Ok(layer_dir_vec)
    }
}
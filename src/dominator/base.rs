use std::path::PathBuf;
use std::collections::HashMap;

use json::{JsonValue, object};

use crate::split::Split;
use crate::merge::Merge;
use crate::dominator::Config;
use crate::util::{fetch_file_sha256, dump_config, get_stack_id};
use crate::errors::{FileCheckError, InternalError, raise};

#[derive(Debug)]
pub struct BaseConfig {
    hash_vec: HashMap<String, String>,
    dir_path: String,
    tar_path: String,
    config_path: String,
    index: usize,
}

impl Config for BaseConfig {
    fn new() -> Self where Self: Sized {
        Self {
            hash_vec: Default::default(),
            dir_path: "".to_string(),
            tar_path: "".to_string(),
            config_path: "".to_string(),
            index: 0,
        }
    }

    fn to_json(&self) -> JsonValue {
        let parent_id = raise(self.hash_vec
            .get("parent_id")
            .ok_or_else(|| InternalError::KeyError { key: "parent_id".into() }));
        let stack_id = raise(self.hash_vec
            .get("stack_id")
            .ok_or_else(|| InternalError::KeyError { key: "stack_id".into() }));
        let split_data: JsonValue = object! {
                parent_id: parent_id.clone(),
                stack_id: stack_id.clone(),
                index: self.index
            };
        split_data
    }

    fn load_json(&mut self, j: JsonValue) {
        self.hash_vec.insert("parent_id".into(), j["parent_id"].to_string());
        self.hash_vec.insert("stack_id".into(), j["stack_id"].to_string());
        self.index = raise(j["index"]
            .as_usize()
            .ok_or_else(|| InternalError::ConvertError));
    }

    fn set_path(&mut self,
                dir_path: String,
                tar_path: String,
                config_path: String) {
        self.dir_path = dir_path;
        self.tar_path = tar_path;
        self.config_path = config_path;
    }

    fn get_dir(&self) -> String {
        self.dir_path.clone()
    }

    fn get_tar(&self) -> String {
        self.tar_path.clone()
    }

    fn get_config(&self) -> String {
        self.config_path.clone()
    }

    fn key(&self) -> usize { self.index }

    fn check_hash(&self, check_map: HashMap<String, String>)
                  -> Result<(), FileCheckError> {
        for (name, hash) in check_map.iter() {
            if !self.hash_vec.contains_key(name) {
                return Err(FileCheckError::HashCheckError {
                    right: hash.clone(),
                    real: "".into(),
                });
            } else if self.hash_vec[name] != hash.clone() {
                return Err(FileCheckError::HashCheckError {
                    right: hash.clone(),
                    real: self.hash_vec[name].clone(),
                });
            }
        }
        Ok(())
    }
}


pub struct BaseDominator {}

impl Split for BaseDominator {
    fn pack_tar_with_config(
        &self,
        split_index: usize,
        split_name: &String,
        split_path: &PathBuf,
        stack_id: String,
        parent_id: String)
        -> Result<(PathBuf, String, String), FileCheckError> {
        let mut cfg = BaseConfig::new();
        cfg.hash_vec.insert("parent_id".into(), parent_id.clone().into());
        cfg.hash_vec.insert(
            "stack_id".into(),
            get_stack_id(&stack_id, &parent_id));
        cfg.index = split_index.into();
        let split_data = cfg.to_json();

        let mut config_pathbuf = split_path.clone();
        config_pathbuf.push(split_name.to_owned());
        config_pathbuf.push("split_config.json");
        dump_config(split_data, &config_pathbuf);

        let tar_path = self.pack_into_tar(split_path, split_name)?;

        let now_id = fetch_file_sha256(&tar_path);
        let now_stack_id = cfg.hash_vec["stack_id"].clone();
        Ok((tar_path, now_stack_id, now_id))
    }
}

impl Merge for BaseDominator {
    fn check_with_config(&self,
                         config_body: &Box<dyn Config>,
                         stack_id: String,
                         parent_id: String)
                         -> Result<(String, String), FileCheckError> {
        let now_id = fetch_file_sha256(&config_body.get_tar());
        let now_stack_id = get_stack_id(&stack_id, &parent_id);

        let mut check_map: HashMap<String, String> = HashMap::new();
        check_map.insert("parent_id".into(), parent_id);
        check_map.insert("stack_id".into(), now_stack_id.clone());
        config_body.check_hash(check_map)?;
        Ok((now_stack_id, now_id))
    }

    fn init_config(&self) -> Box<dyn Config> {
        Box::new(BaseConfig::new())
    }
}
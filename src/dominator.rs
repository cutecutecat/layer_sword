pub mod base;

use std::collections::HashMap;

use json::JsonValue;

use crate::errors::FileCheckError;

pub trait Config {
    fn new() -> Self where Self: Sized;
    fn to_json(&self) -> JsonValue;
    fn load_json(&mut self, j: JsonValue);
    fn set_path(&mut self, dir_path: String, tar_path: String, config_path: String);
    fn get_dir(&self) -> String;
    fn get_tar(&self) -> String;
    fn get_config(&self) -> String;
    fn key(&self) -> usize;
    fn check_hash(&self, check_map: HashMap<String, String>) -> Result<(), FileCheckError>;
}

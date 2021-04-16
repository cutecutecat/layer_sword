pub mod base;

use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use crate::errors::FileCheckError;

pub trait Inspect {
    fn inspect_route(&self, extract_path: &Path)
                     -> Result<(HashMap<String, PathBuf>, HashSet<String>),
                         FileCheckError>;
    fn inspect_config(&self, file_map: &HashMap<String, PathBuf>)
                      -> Result<HashSet<String>, FileCheckError>;
    fn inspect_layer(&self,
                     extract_path: &Path,
                     layer_hash_set: &HashSet<String>,
                     config_tar_hash: &HashSet<String>)
                     -> Result<(), FileCheckError>;
    fn inspect_manifest(&self,
                        extract_path: &Path,
                        file_map: &HashMap<String, PathBuf>,
                        layer_hash_set: &HashSet<String>)
                        -> Result<Vec<PathBuf>, FileCheckError>;
    fn inspect(&self, extract_path: &Path)
               -> Result<(HashMap<String, PathBuf>, Vec<PathBuf>),
                   FileCheckError> {
        log::info!("Inspecting route of required files");
        let (file_map, layer_hash_set) =
            self.inspect_route(extract_path)?;
        log::info!("Inspecting items inside config file");
        let config_tar_hash = self.inspect_config(&file_map)?;
        log::info!("Inspecting file inside each layer");
        self.inspect_layer(&extract_path, &layer_hash_set, &config_tar_hash)?;
        log::info!("Inspecting items inside manifest file");
        let layer_dir_vec =
            self.inspect_manifest(&extract_path, &file_map, &layer_hash_set)?;
        Ok((file_map, layer_dir_vec))
    }
}
use std::path::PathBuf;
use std::fmt::Debug;
use std::sync::RwLock;
use std::fs;

use lazy_static::lazy_static;
use thiserror::Error as ThisError;
use log::error;

lazy_static! {
    pub static ref GENERATE_PATH: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());
}

#[derive(ThisError, Debug)]
pub enum LayerSwordError {
    #[error("[Something happened unexpectedly]")]
    InternalError(#[from] InternalError),
    #[error("[Error from terminal inspection]")]
    TerminalError(#[from] TerminalError),
    #[error("[Error from file inspection]")]
    FileCheckError(#[from] FileCheckError),
}

#[derive(ThisError, Debug)]
pub enum InternalError {
    #[error("Failed to convert types")]
    ConvertError,
    #[error("Key {key} is not found in array type data")]
    KeyError { key: String },
    #[error("Error occurred when parsing path:\n'{path}'")]
    FilePathError { path: PathBuf },
    #[error("Config file is too large(> 1MB):\n'{path}'\nsize:{size}")]
    TooLargeConfigSizeError { path: PathBuf, size: usize },
    #[error("The vector is empty unexpectedly")]
    VecEmptyError,
    #[error("An impossible error occurred:\n{msg:?}")]
    ImpossibleError { msg: String },
}

#[derive(ThisError, Debug)]
pub enum TerminalError {
    #[error("Config file of arguments is invalid")]
    InputConfigError,
    #[error("No arg '{arg}' when running client\n{msg}")]
    WithoutArgError { arg: String, msg: String },
    #[error("Arg '{arg}' error when running client\n{msg}")]
    BadArgError { arg: String, msg: String },
    #[error("Path `{path}` is a file rather than directory")]
    NotDirectoryError { path: String },
    #[error("Path `{path}` is a directory rather than file")]
    NotFileError { path: String },
    #[error("Path `{path}` not exist")]
    NotExistError { path: String },
    #[error("Clap arguments check failed")]
    ClapError,
}

#[derive(ThisError, Debug)]
pub enum FileCheckError {
    #[error("Config file is invalid")]
    ConfigFileError,
    #[error("Split file is invalid")]
    SplitFileError,
    #[error("Docker file check failed:\n{msg:?}")]
    BadDockerFileError { msg: String },
    #[error("Checksum is not valid\nright:'{right}'\nreal:'{real}'")]
    HashCheckError { right: String, real: String },
    #[error("File should have extension '{extension}' at path:\n'{path}'")]
    FileExtensionError { extension: String, path: PathBuf },
    #[error("File have item '{path}' more than 2")]
    TooManyDepthError { path: String },
    #[error("Splits unmatched with more than 1 index '{index}'")]
    SplitsUnmatchedError { index: usize },
}

fn clean_workspace(){
    let path_reader = GENERATE_PATH.read();
    if path_reader.is_ok() {
        let real_path_vec = path_reader
            .expect("An impossible error occurred");
        for path in &*real_path_vec {
            if path.is_dir() {
                if let Err(e) = fs::remove_dir_all(path) {
                    error!("{:#?}", e);
                }
            }
        }
    }
}

pub fn report<V: Debug, E: Debug, O: Debug>(ret: Result<V, E>, map: O) -> Result<V, O> {
    if ret.is_err() {
        return Err(report_err(ret.expect_err("An impossible error occurred"), map));
    } else {
        Ok(ret.expect("An impossible error occurred"))
    }
}

pub fn report_err<E: Debug, O: Debug>(err: E, map: O) -> O {
    error!("{:#?}", err);
    map
}

pub fn raise<V: Debug, E: Debug>(ret: Result<V, E>) -> V {
    if ret.is_err() {
        raise_err(ret.as_ref().expect_err("An impossible error occurred"));
    }
    ret.expect("An impossible error occurred")
}

pub fn raise_err<E: Debug>(err: E) {
    error!("{:#?}", err);
    // clean temporary files if an error is raised
    clean_workspace();
    std::process::exit(-1);
}
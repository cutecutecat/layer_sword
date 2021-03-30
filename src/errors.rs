use std::string::FromUtf8Error;
use std::path::PathBuf;

use thiserror::Error as ThisError;

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

    #[error(transparent)]
    ClapError(#[from] clap::Error),
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
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Splits unmatched with more than 1 index '{index}'")]
    SplitsUnmatchedError { index: usize },
}
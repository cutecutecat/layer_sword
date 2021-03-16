use std::string::FromUtf8Error;
use std::num::ParseIntError;
use std::path::PathBuf;

use thiserror::Error;
use json::JsonError;
use log::SetLoggerError;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to convert types")]
    ConvertError(),
    #[error("Key {key} is not found in array type data")]
    KeyError { key: String },
    #[error("Error occurred when parsing path:\n'{path}'")]
    FilePathError { path: PathBuf },
    #[error("Docker file check failed:\n{msg:?}")]
    BadDockerFileError { msg: String },
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
    #[error("Config file is invalid")]
    ConfigFileError(),
    #[error("Split file is invalid")]
    SplitFileError(),
    #[error("Checksum is not valid\nright:'{right}'\nreal:'{real}'")]
    HashCheckError { right: String, real: String },
    #[error("Internal check failed\n{msg}")]
    InternalError { msg: String },
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error(transparent)]
    JsonError(#[from] JsonError),
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    FsExtraError(#[from] fs_extra::error::Error),
    #[error(transparent)]
    ClapError(#[from] clap::Error),
    #[error(transparent)]
    SetLoggerError(#[from] SetLoggerError),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
}
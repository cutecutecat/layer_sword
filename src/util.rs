use std::fs::{File, read_to_string, write};
use std::{io, fs};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use tar::Archive;
use sha2::{Sha256, Digest};
use json::JsonValue;
use flate2::{Compression, GzBuilder};
use flate2::read::GzDecoder;
use walkdir::{WalkDir, DirEntry};

use crate::errors::{FileCheckError, InternalError, raise, report, report_err, GENERATE_PATH};
use crate::errors::InternalError::{TooLargeConfigSizeError, VecEmptyError, FilePathError};

/// decompress files with tar.gz suffix
///
/// # Examples
///
/// ```no_run
/// use layer_sword::util::extract_tar_gz;
/// fn main() -> std::io::Result<()> {
///     let mut f = extract_tar_gz("base.tar.gz", "tmp");
///     Ok(())
/// }
/// ```
pub fn extract_tar_gz<P>(gz_path: P, extract_path: P)
                         -> Result<(), FileCheckError>
    where
        P: AsRef<Path> {
    let file = raise(File::open(&gz_path));
    let dec = GzDecoder::new(file);
    let hash_u8 = dec
        .header()
        .ok_or_else(|| FileCheckError::SplitFileError)?
        .comment()
        .ok_or_else(|| FileCheckError::SplitFileError)?;
    let hash = raise(String::from_utf8(Vec::from(hash_u8)));
    let mut archive = Archive::new(dec);
    let mut file_vec: Vec<PathBuf> = Vec::new();
    for entry in archive
        .entries()
        .map_err(|e| { report_err(e, FileCheckError::SplitFileError) })? {
        let entry = entry
            .map_err(|e| report_err(e, FileCheckError::SplitFileError))?;
        let file_pathbuf = raise(entry.path()).into_owned();
        file_vec.push(file_pathbuf);
    }
    if file_vec.len() != 1 {
        return Err(FileCheckError::SplitFileError);
    }

    let file_upk = raise(File::open(&gz_path));
    let dec_upk = GzDecoder::new(file_upk);
    let mut archive_upk = Archive::new(dec_upk);

    raise(archive_upk.unpack(&extract_path));

    let mut tar_path = extract_path.as_ref().to_path_buf();
    let filename = raise(file_vec
        .get(0)
        .ok_or_else(|| InternalError::KeyError { key: format!("0") }));
    tar_path.push(filename);
    let real_hash = fetch_file_sha256(tar_path);
    if hash != real_hash {
        FileCheckError::HashCheckError { right: hash, real: real_hash };
    }
    Ok(())
}

/// decompress files with tar suffix
///
/// # Examples
///
/// ```no_run
/// use layer_sword::util::extract_tar;
/// fn main() -> std::io::Result<()> {
///     let mut f = extract_tar("base.tar", "tmp");
///     Ok(())
/// }
/// ```
pub fn extract_tar<P>(tar_path: P, extract_path: P)
    where
        P: AsRef<Path> {
    let file = raise(File::open(tar_path));
    let mut archive = Archive::new(&file);
    archive.set_preserve_permissions(false);
    raise(archive.unpack(extract_path));
}

/// compress tar.gz file from a file with tar suffix
///
/// # Examples
///
/// ```no_run
/// use layer_sword::util::compress_tar_gz;
/// fn main() -> std::io::Result<()> {
///     let mut f = compress_tar_gz("base.tar.gz", "base.tar", 6);
///     Ok(())
/// }
/// ```
pub fn compress_tar_gz<P>(gz_path: P, file_path: P, compress_level: u8)
    where
        P: AsRef<Path> {
    let gz_file = raise(File::create(gz_path));
    let enc = GzBuilder::new()
        .comment(fetch_file_sha256(&file_path))
        .write(gz_file, Compression::new(compress_level.into()));

    let mut tar = tar::Builder::new(enc);
    tar.mode(tar::HeaderMode::Deterministic);
    let mut file = raise(File::open(&file_path));
    let filename = raise(file_path
        .as_ref()
        .file_name()
        .ok_or_else(|| InternalError::FilePathError { path: file_path.as_ref().to_path_buf() }));
    raise(tar.append_file(filename, &mut file));
}

/// fetch the file path on the tail of an entry(1 or 2 if exists)
fn iter_child_path(entry: &DirEntry) -> PathBuf {
    let mut depth = entry.depth() - 1;
    let mut mid_list: Vec<&OsStr> = vec![entry.file_name()];
    let mut mid_path = entry.path();
    while depth > 0 {
        mid_path = raise(mid_path
            .parent()
            .ok_or_else(|| FilePathError { path: mid_path.into() }));
        let mid_name = raise(mid_path
            .file_name()
            .ok_or_else(|| FilePathError { path: mid_path.into() }));
        mid_list.push(mid_name);
        depth -= 1;
    }
    let mut child_name: PathBuf = raise(mid_list.pop().ok_or_else(|| VecEmptyError)).into();
    while mid_list.len() > 0 {
        let mid_name = raise(mid_list.pop().ok_or_else(|| VecEmptyError));
        child_name.push(mid_name);
    }
    child_name
}

/// compress tar file from a directory
///
/// # Examples
///
/// ```no_run
/// use layer_sword::util::compress_tar;
/// fn main() -> std::io::Result<()> {
///     let mut f = compress_tar("base.tar", "base");
///     Ok(())
/// }
/// ```
pub fn compress_tar<P>(tar_path: P, extract_path: P) -> Result<(), FileCheckError>
    where
        P: AsRef<Path> {
    let file = raise(File::create(tar_path));
    let mut tar = tar::Builder::new(file);
    tar.mode(tar::HeaderMode::Deterministic);
    let all_extracted_paths = WalkDir::new(extract_path)
        .sort_by_key(|item: &DirEntry| item.clone().into_path());
    for entry in all_extracted_paths {
        let entry = raise(entry);
        let item_path = entry.path();
        let item_name: PathBuf;
        if entry.depth() == 0 {
            continue;
        } else if entry.depth() <= 2 {
            item_name = iter_child_path(&entry);
        } else {
            let path = entry.path().to_str().unwrap_or_default().to_string();
            return Err(FileCheckError::TooManyDepthError { path });
        }
        raise(tar.append_path_with_name(item_path, item_name));
    }
    Ok(())
}

/// init structure of working directory
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use layer_sword::util::init_path;
/// fn main() -> std::io::Result<()> {
///     let handle_path = Path::new("tmp");
///     let out_path = Path::new("out");
///     let mut f = init_path(handle_path, out_path);
///     Ok(())
/// }
/// ```
pub fn init_path(handle_path: &Path, out_path: &Path) {
    // set path for error clean work
    let mut path_writer = raise(GENERATE_PATH.write());
    path_writer.push(handle_path.to_path_buf());
    path_writer.push(out_path.to_path_buf());

    for path in vec![handle_path, out_path] {
        if path.exists() {
            if path.is_dir()
            {
                raise(fs::remove_dir_all(path));
            } else if path.is_file() {
                raise(fs::remove_file(path));
            }
        }
        let mut ret = fs::create_dir(path);
        // try several times waiting for remove finished by os
        let mut try_times: u8 = 0;
        while ret.is_err() {
            try_times += 1;
            ret = fs::create_dir(path);
            if try_times > 10 {
                raise(ret);
                return;
            }
        }
    }
    let mut split_path = handle_path.clone().to_path_buf();
    split_path.push("split");

    let mut merge_path = handle_path.clone().to_path_buf();
    merge_path.push("merge");

    raise(fs::create_dir(split_path));
    raise(fs::create_dir(merge_path));
}

/// load json config from text file
///
/// # Examples
///
/// ```no_run
/// use layer_sword::util::load_config;
/// fn main() -> std::io::Result<()> {
///     let mut f = load_config("config.json");
///     Ok(())
/// }
/// ```
pub fn load_config<P>(config_path: P) -> Result<JsonValue, FileCheckError>
    where
        P: AsRef<Path> + Copy {
    let file = report(
        File::open(config_path),
        FileCheckError::ConfigFileError)?;
    let metadata = report(file.metadata(), FileCheckError::ConfigFileError)?;
    let size = metadata.len();
    // raise error if file size > 1MB
    if size > 1048576 {
        return Err(report_err(TooLargeConfigSizeError {
            path: config_path.as_ref().to_path_buf(),
            size: size as usize,
        }, FileCheckError::ConfigFileError));
    }
    let contents = report(
        read_to_string(config_path),
        FileCheckError::ConfigFileError)?;
    let ret = report(
        json::parse(contents.as_str()),
        FileCheckError::ConfigFileError)?;
    Ok(ret)
}

/// dump json config into text file
///
/// # Examples
///
/// ```no_run
/// use json::{JsonValue, object};
/// use layer_sword::util::dump_config;
/// fn main() -> std::io::Result<()> {
///     let config: JsonValue = object! {
///                 parent_id: "123456",
///                 stack_id: "stack",
///                 index: -1
///             };
///     let mut f = dump_config(config, "config.json");
///     Ok(())
/// }
/// ```
pub fn dump_config<P>(config: JsonValue, config_path: P)
    where
        P: AsRef<Path> {
    raise(write(config_path, config.dump()));
}

/// fetch sha256 hash of a file
///
/// # Examples
///
/// ```no_run
/// use layer_sword::util::fetch_file_sha256;
/// fn main() -> std::io::Result<()> {
///     let hash = fetch_file_sha256("base.tar");
///     Ok(())
/// }
/// ```
pub fn fetch_file_sha256<P>(path: P) -> String
    where
        P: AsRef<Path> {
    let mut file = raise(File::open(path));
    let mut sha256 = Sha256::new();
    raise(io::copy(&mut file, &mut sha256));
    let hash_result = sha256.finalize();
    let real_hash = format!("{:x}", hash_result);
    real_hash
}

/// fetch sha256 hash of a string
///
/// # Examples
///
/// ```rust
/// use layer_sword::util::fetch_string_sha256;
/// fn main() -> std::io::Result<()> {
///     let target_string = format!("layer_sword");
///     let hash = fetch_string_sha256(&target_string);
///     let right = format!("4deaf80f304870a2bc7a9a1f3a952d86d3db19e01094f28cad8a06e1ad6fb2c1");
///     assert_eq!(hash, right);
///     Ok(())
/// }
/// ```
pub fn fetch_string_sha256(target_str: &String) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(target_str);
    let hash_result = sha256.finalize();
    let real_hash = format!("{:x}", hash_result);
    real_hash
}

/// calculate stack_id from last stack_id and parent_id
///
/// # Examples
///
/// ```rust
/// use layer_sword::util::get_stack_id;
/// fn main() -> std::io::Result<()> {
///     let first_id = format!("a6e99f9b50e1bb8366d55fee15116a4da796c6bc37ebec09e7e77ec4cfa629fb");
///     let second_id = format!("8de3e6511bb095f7d7d4133e877391f6ee1ec2bfda022bc24e2443277d3966b6");
///     let hash = get_stack_id(&first_id, &second_id);
///     let right = format!("a5a8033bc04ce56c3f0982deaabad8125581856fd702262772511efd69b18de9");
///     assert_eq!(hash, right);
///     Ok(())
/// }
/// ```
pub fn get_stack_id(last_stack_id: &String, parent_id: &String) -> String {
    let mut raw_id = last_stack_id.clone();
    raw_id.push_str("\n");
    raw_id.push_str(&parent_id);
    fetch_string_sha256(&raw_id)
}

/// macro convert PathBuf to string
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use layer_sword::path_to_string;
/// use layer_sword::errors::raise;
/// fn main() -> std::io::Result<()> {
///     let path = PathBuf::from("/");
///     let path = path_to_string!(path);
///     assert_eq!(path, "/".to_string());
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! path_to_string {
    ($p:expr) => {
        raise($p.into_os_string().into_string())
    };
}

/// macro convert &osStr to string
/// # Examples
///
/// ```rust
/// use std::ffi::OsStr;
/// use layer_sword::os_str_to_string;
/// use layer_sword::errors::raise;
/// fn main() -> std::io::Result<()> {
///     let path = OsStr::new("/");
///     let path = os_str_to_string!(path);
///     assert_eq!(path, "/".to_string());
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! os_str_to_string {
    ($p:expr) => {
        raise($p.to_os_string().into_string())
    };
}
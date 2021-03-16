use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use fs_extra::{dir, file};
use json::object;

use crate::errors::{Error, Result};
use crate::inspector::inspect;
use crate::util::{compress_tar, compress_tar_gz, dump_config, fetch_file_sha256, get_stack_id, extract_tar};


fn deduct_split_map(split_names: &Vec<String>,
                    split_map: &HashMap<String, i16>,
                    layer_dir_set: &Vec<PathBuf>)
                    -> Result<HashMap<String, i16>> {
    let mut deduct_map = split_map.clone();
    let mut layer_num: i16 = 0;
    let mut addup_flag: String = String::new();
    if split_names.len() != deduct_map.len() {
        Error::BadArgError {
            arg: format!("layers"),
            msg: format!("size of arg names not equal to size of arg layers"),
        };
    }
    for name in split_names {
        if name.len() == 0 {
            Error::BadArgError {
                arg: format!("names"),
                msg: format!("one of arg names NULL string"),
            };
        }
        if !deduct_map.contains_key(name) {
            Error::InternalError { msg: format!("key {} of split_names isn't inside split map", name) };
        }
        let split_num: &i16 = deduct_map
            .get(name)
            .ok_or_else(|| Error::KeyError { key: name.clone() })?;
        if *split_num == -1 {
            if addup_flag == "" {
                addup_flag = name.clone();
            } else {
                Error::BadArgError {
                    arg: format!("layers"),
                    msg: format!("more than 1 split of -1 layers: '{:?}' and '{:?}'",
                                 addup_flag, name),
                };
            }
        } else if *split_num < -1
        {
            Error::BadArgError {
                arg: format!("layers"),
                msg: format!("split number cannot be positive or -1, actuall '{:?}'", split_num),
            };
        } else {
            layer_num += *split_num as i16;
        }
    }
    // deal with addup layer of -1 flag
    if addup_flag == "" {
        if (layer_dir_set.len() as i16) != layer_num {
            Error::BadArgError {
                arg: format!("layers"),
                msg: format!("layers per split(without deduct) doesn't sum to '{:?}', not '{:?}'",
                             layer_num, layer_dir_set.len()),
            };
        }
    } else {
        if layer_num > (layer_dir_set.len() - 1) as i16 {
            Error::BadArgError {
                arg: format!("layers"),
                msg: format!("layers per split(with deduct) doesn't sum to '{:?}', larger than '{:?}'",
                             layer_num, layer_dir_set.len()),
            };
        }
        deduct_map.insert(addup_flag, layer_dir_set.len() as i16 - layer_num);
    }
    Ok(deduct_map)
}

fn copy_split_directories(split_names: &Vec<String>,
                          split_map: &HashMap<String, i16>,
                          layer_dir_set: &Vec<PathBuf>,
                          top_pathbuf: &PathBuf) -> Result<()> {
    let mut id_from: i16 = 0;

    let mut copy_options_dir = dir::CopyOptions::new();
    copy_options_dir.overwrite = true;
    copy_options_dir.copy_inside = true;

    for name in split_names {
        let mut split_pathbuf = top_pathbuf.clone();
        split_pathbuf.push(name.clone());
        fs::create_dir(&split_pathbuf)?;

        for id in 0..split_map[name] {
            let src_path = layer_dir_set
                .get((id_from + id) as usize)
                .ok_or_else(|| Error::KeyError { key: (id_from + id).to_string() })?;
            let item_name = src_path
                .file_name()
                .ok_or_else(|| Error::FilePathError { path: src_path.clone() })?;
            let mut dst_path = split_pathbuf.clone();
            dst_path.push(item_name);
            dir::copy(src_path, dst_path, &copy_options_dir)?;
        }
        id_from += split_map[name];
    }
    Ok(())
}

fn copy_split_files(split_names: &Vec<String>,
                    file_map: HashMap<String, PathBuf>,
                    top_pathbuf: &PathBuf) -> Result<()> {
    let mut copy_options_file = file::CopyOptions::new();
    copy_options_file.overwrite = true;
    let top_layer = split_names
        .get(split_names.len() - 1)
        .ok_or_else(|| Error::KeyError { key: (split_names.len() - 1).to_string() })?;
    for (_, src_path) in file_map {
        let filename = src_path
            .file_name()
            .ok_or_else(|| Error::FilePathError { path: src_path.clone() })?
            .to_os_string()
            .into_string()
            .map_err(|_| Error::ConvertError())?;
        let mut dst_pathbuf = top_pathbuf.clone();
        dst_pathbuf.push(top_layer);
        dst_pathbuf.push(filename);
        file::copy(&src_path, &dst_pathbuf, &copy_options_file)?;
    }
    Ok(())
}

fn pack_into_gz(split_names: &Vec<String>,
                split_pathbuf: &PathBuf,
                out_pathbuf: &PathBuf,
                compress_level: u8) -> Result<()> {
    // 配置文件 parent_id, diff_id, is_top, is_bottom
    // parrent_id = sha256(zip parrent)
    // stack_id = (parrent stack id+'\n'parrent id or '\n')
    let mut parent_id = String::new();
    let mut stack_id = String::new();
    for (i, name) in split_names.iter().enumerate() {
        let mut split_data = object! {
            parent_id: String::new(),
            stack_id: String::new(),
            index: 0
        };
        split_data["index"] = i.into();
        split_data["parent_id"] = parent_id.clone().into();
        stack_id = get_stack_id(&stack_id, &parent_id)?;
        split_data["stack_id"] = stack_id.to_string().into();
        let mut config_pathbuf = split_pathbuf.clone();
        config_pathbuf.push(name.to_owned());
        config_pathbuf.push("split_config.json");
        dump_config(split_data, &config_pathbuf)?;

        // 压缩
        let mut tar_pathbuf = split_pathbuf.clone();
        tar_pathbuf.pop();
        tar_pathbuf.push(name.to_owned() + ".tar");

        let mut compress_path = split_pathbuf.clone();
        compress_path.push(name);
        compress_tar(&tar_pathbuf, &compress_path)?;
        parent_id = fetch_file_sha256(&tar_pathbuf)?;
        fs::remove_dir_all(compress_path)?;

        // 压缩gzip
        let mut gz_pathbuf = out_pathbuf.clone();
        gz_pathbuf.push(tar_pathbuf.file_name().unwrap_or_default());
        gz_pathbuf.set_extension("tar.gz");
        compress_tar_gz(&gz_pathbuf, &tar_pathbuf, compress_level)?;
    }
    Ok(())
}

pub fn split_layer(tar_path: &Path,
                   split_names: Vec<String>,
                   split_map: HashMap<String, i16>,
                   work_path: &Path,
                   out_path: &Path,
                   compress_level: u8)
                   -> Result<()> {
    let mut extract_pathbuf = work_path.to_path_buf();
    extract_pathbuf.push("merge");
    let mut split_pathbuf = work_path.to_path_buf();
    split_pathbuf.push("split");
    log::info!("Extracting tar file of dock image at {}",
               tar_path.to_str().ok_or_else(|| Error::ConvertError())?);
    extract_tar(tar_path, &extract_pathbuf)?;
    log::info!("Checking merged dock image files");
    log::info!("[inspect begin]");
    let (file_map, layer_dir_set) =
        inspect(&extract_pathbuf)?;
    log::info!("[inspect end]");
    log::info!("Validating number of each layer");
    let deduct_map = deduct_split_map(&split_names, &split_map, &layer_dir_set)?;
    log::info!("Copying layer directories inside splits into dock image");
    copy_split_directories(&split_names, &deduct_map,
                           &layer_dir_set, &split_pathbuf)?;
    log::info!("Copying files inside splits into dock image");
    copy_split_files(&split_names, file_map, &split_pathbuf)?;
    log::info!("Packing items into tar.gz file under {}",
               &out_path.to_str().ok_or_else(|| Error::ConvertError())?);
    pack_into_gz(&split_names, &split_pathbuf,
                 &out_path.to_path_buf(), compress_level)?;
    log::info!("Clean items inside work path");
    fs::remove_dir_all(work_path)?;
    Ok(())
}
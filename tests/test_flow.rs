#[cfg(test)]
use std::path::Path;
use std::fs;
use std::collections::HashMap;

use ctor::ctor;
use lazy_static::lazy_static;

use layer_sword::inspector::inspect;
use layer_sword::util::{init_path, extract_tar, fetch_file_sha256};
use layer_sword::split::split_layer;
use layer_sword::merge::merge_layer;
use layer_sword::errors::LayerSwordError;

type Result<T> = core::result::Result<T, LayerSwordError>;

lazy_static! {
    static ref DIR_VEC: Vec<String> = vec![
        "tests/test_init_path",
        "tests/test_inspect",
        "tests/out_split_layer", "tests/test_split_layer",
        "tests/out_deduction", "tests/test_deduction",
        "tests/out_split_four_layer", "tests/test_split_four_layer",
        "tests/out_split_two_layer", "tests/test_split_two_layer",
        "tests/out_merge", "tests/test_merge",
        "tests/out_compress_best","tests/test_compress_best"
    ].iter().map(|s| s.to_string()).collect();
}

#[ctor]
fn before() {
    env_logger::builder().is_test(true).try_init().unwrap_or_else(|_| {});
    for dir in DIR_VEC.clone() {
        let dir_path = Path::new(&dir);
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).unwrap();
        }
        fs::create_dir(dir_path).unwrap();
    }
}

#[test]
fn test_init_path() -> Result<()> {
    log::info!("Test for 'init_path' function.");
    let work_path = Path::new("tests/test_init_path");
    init_path(work_path);
    let split_path = Path::new("tests/test_init_path/split");
    let merge_path = Path::new("tests/test_init_path/merge");
    assert_eq!(split_path.exists(), true);
    assert_eq!(merge_path.exists(), true);

    Ok(())
}

#[test]
fn test_inspect() -> Result<()> {
    log::info!("Test for 'extract_tar' and 'inspect' functions.");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_inspect");
    init_path(work_path);
    let merge_path = Path::new("tests/test_inspect/merge");
    extract_tar(tar_path, merge_path);
    inspect(merge_path)?;

    Ok(())
}

#[test]
fn test_split_layer() -> Result<()> {
    log::info!("Test for basic split procedure.");
    let out_path = Path::new("tests/out_split_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_layer");
    init_path(work_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    split_names.push(format!("app"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), 3);
    split_map.insert(format!("app"), 1);
    let compress_level: u8 = 6;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_split_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("1444a680a0d364083479cf40a35d0a458230ace1587e5180f7c451ce8288c071");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("03b898decb769660a3933aa043ed9e09c486496c3d2afac8b54a1984b61884db");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_layer/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("8e56648c9688ec56fc63b350029a73f09ce2d98830dcf53d656eb3135cea48a3");
    assert_eq!(app_hash, app_right);

    Ok(())
}


#[test]
fn test_deduction() -> Result<()> {
    log::info!("Test for auto-deduction inside split procedure.");
    let out_path = Path::new("tests/out_deduction");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_deduction");
    init_path(work_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    split_names.push(format!("app"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), -1);
    split_map.insert(format!("app"), 1);
    let compress_level: u8 = 6;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_deduction/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("1444a680a0d364083479cf40a35d0a458230ace1587e5180f7c451ce8288c071");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_deduction/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("03b898decb769660a3933aa043ed9e09c486496c3d2afac8b54a1984b61884db");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_deduction/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("8e56648c9688ec56fc63b350029a73f09ce2d98830dcf53d656eb3135cea48a3");
    assert_eq!(app_hash, app_right);

    Ok(())
}

#[test]
fn test_split_four_layer() -> Result<()> {
    log::info!("Test for four split dividing procedure.");
    let out_path = Path::new("tests/out_split_four_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_four_layer");
    init_path(work_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("sys"));
    split_names.push(format!("lib"));
    split_names.push(format!("app"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("sys"), 1);
    split_map.insert(format!("lib"), 2);
    split_map.insert(format!("app"), 1);
    let compress_level: u8 = 6;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_split_four_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("1444a680a0d364083479cf40a35d0a458230ace1587e5180f7c451ce8288c071");
    assert_eq!(os_hash, os_right);

    let sys_path = Path::new("tests/out_split_four_layer/sys.tar.gz");
    let sys_hash = fetch_file_sha256(sys_path);
    let sys_right =
        format!("6ab1d0f0d3e5ebe0f2da1fc1bffd2a645fee6407d14ec45a3ce8f9bc77b4305f");
    assert_eq!(sys_hash, sys_right);

    let lib_path = Path::new("tests/out_split_four_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("ab16d417100ec3689f393f185597fcfd9f2313ca98cc524df2961309710d42a0");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_four_layer/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("e2db3aa24616d2d807711c128eb7392dd4297483b4f3f402a9a29caeff8305f0");
    assert_eq!(app_hash, app_right);

    Ok(())
}

#[test]
fn test_split_two_layer() -> Result<()> {
    log::info!("Test for two split dividing procedure.");
    let out_path = Path::new("tests/out_split_two_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_two_layer");
    init_path(work_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), -1);
    let compress_level: u8 = 6;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_split_two_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("1444a680a0d364083479cf40a35d0a458230ace1587e5180f7c451ce8288c071");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_two_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("5e047d5fedcfb1b703cf4a18251305fa8c11ffe9441d2d10fd0daea1f0eb55fc");
    assert_eq!(lib_hash, lib_right);

    Ok(())
}

#[test]
fn test_merge() -> Result<()> {
    log::info!("Test for basic merge procedure.");
    let target_path = Path::new("tests/data/splits_base");
    let work_path = Path::new("tests/test_merge");
    let out_path = Path::new("tests/out_merge");
    init_path(work_path);
    merge_layer(target_path, work_path, out_path)?;

    let tar_path = Path::new("tests/out_merge/merge.tar");
    let tar_hash = fetch_file_sha256(tar_path);
    let tar_right =
        format!("1203eb785534ec43619880aef42e08bbe7a0ba1f1e10315863bcfe8f6542cea2");
    assert_eq!(tar_hash, tar_right);
    Ok(())
}

#[test]
fn test_compress_best() -> Result<()> {
    log::info!("Test for compress at best level.");
    let out_path = Path::new("tests/out_compress_best");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_compress_best");
    init_path(work_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), -1);
    let compress_level: u8 = 9;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_compress_best/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("f01ee5d4b1a78fe483762761afa40a5d8e324503a6bfdd8c92a0e357b2dbea8d");
    assert_eq!(os_hash, os_right);

    Ok(())
}
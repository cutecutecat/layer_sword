#[cfg(test)]
use std::path::Path;
use std::fs;
use std::collections::HashMap;

use ctor::{ctor, dtor};
use lazy_static::lazy_static;
use simple_logger::SimpleLogger;

use layer_sword::inspector::inspect;
use layer_sword::util::{init_path, extract_tar, fetch_file_sha256};
use layer_sword::split::split_layer;
use layer_sword::merge::merge_layer;
use layer_sword::errors::Result;

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
    SimpleLogger::new().init().unwrap();
    for dir in DIR_VEC.clone() {
        let dir_path = Path::new(&dir);
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).unwrap();
        }
        fs::create_dir(dir_path).unwrap();
    }
}

#[dtor]
fn after() {
    for dir in DIR_VEC.clone() {
        let dir_path = Path::new(&dir);
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).unwrap();
        }
    }
}

#[test]
fn test_init_path() -> Result<()> {
    log::info!("Test for 'init_path' function.");
    let work_path = Path::new("tests/test_init_path");
    init_path(work_path)?;
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
    init_path(work_path)?;
    let merge_path = Path::new("tests/test_inspect/merge");
    extract_tar(tar_path, merge_path)?;
    inspect(merge_path)?;

    Ok(())
}

#[test]
fn test_split_layer() -> Result<()> {
    log::info!("Test for basic split procedure.");
    let out_path = Path::new("tests/out_split_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_layer");
    init_path(work_path)?;
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
    let os_hash = fetch_file_sha256(os_path)?;
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path)?;
    let lib_right =
        format!("022bac800ffd3c66e7ea84b3dff763624f15c4d587c6050ad46c17a63dbd154d");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_layer/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path)?;
    let app_right =
        format!("e825bb228b29bff204c83ea68b78c652e28b38b6449f93e8d7ea3b26411eae1b");
    assert_eq!(app_hash, app_right);

    Ok(())
}



#[test]
fn test_deduction() -> Result<()> {
    log::info!("Test for auto-deduction inside split procedure.");
    let out_path = Path::new("tests/out_deduction");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_deduction");
    init_path(work_path)?;
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
    let os_hash = fetch_file_sha256(os_path)?;
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_deduction/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path)?;
    let lib_right =
        format!("022bac800ffd3c66e7ea84b3dff763624f15c4d587c6050ad46c17a63dbd154d");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_deduction/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path)?;
    let app_right =
        format!("e825bb228b29bff204c83ea68b78c652e28b38b6449f93e8d7ea3b26411eae1b");
    assert_eq!(app_hash, app_right);

    Ok(())
}

#[test]
fn test_split_four_layer() -> Result<()> {
    log::info!("Test for four split dividing procedure.");
    let out_path = Path::new("tests/out_split_four_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_four_layer");
    init_path(work_path)?;
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
    let os_hash = fetch_file_sha256(os_path)?;
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let sys_path = Path::new("tests/out_split_four_layer/sys.tar.gz");
    let sys_hash = fetch_file_sha256(sys_path)?;
    let sys_right =
        format!("c042c827eda36612df51f78a921e18c93c3cea5fe5ff558e08b83487b204db20");
    assert_eq!(sys_hash, sys_right);

    let lib_path = Path::new("tests/out_split_four_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path)?;
    let lib_right =
        format!("caf9f00ddc18b0a533cc75d77f7ac9e0b014f3c5044d1506ea4e1fd7d3ba60bc");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_four_layer/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path)?;
    let app_right =
        format!("103c7f2e0ffef676b9ffd33c6750aa26cd1846c7039738eb2c3b263e8fc2a41b");
    assert_eq!(app_hash, app_right);

    Ok(())
}

#[test]
fn test_split_two_layer() -> Result<()> {
    log::info!("Test for two split dividing procedure.");
    let out_path = Path::new("tests/out_split_two_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_two_layer");
    init_path(work_path)?;
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), -1);
    let compress_level: u8 = 6;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_split_two_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path)?;
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_two_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path)?;
    let lib_right =
        format!("a8eb409aa155198df103ac77b5e7928b33cb5e3022f78eb209935aecca63ba92");
    assert_eq!(lib_hash, lib_right);

    Ok(())
}

#[test]
fn test_merge() -> Result<()> {
    log::info!("Test for basic merge procedure.");
    let target_path = Path::new("tests/data/splits_base");
    let work_path = Path::new("tests/test_merge");
    let out_path = Path::new("tests/out_merge");
    init_path(work_path)?;
    merge_layer(target_path, work_path, out_path)?;

    let tar_path = Path::new("tests/out_merge/merge.tar");
    let tar_hash = fetch_file_sha256(tar_path)?;
    let tar_right =
        format!("dee5fa9709718753d732542ec0bf5c035a1146dd8792d125f1d6c0589b4e23d6");
    assert_eq!(tar_hash, tar_right);
    Ok(())
}

#[test]
fn test_compress_best() -> Result<()> {
    log::info!("Test for compress at best level.");
    let out_path = Path::new("tests/out_compress_best");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_compress_best");
    init_path(work_path)?;
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), -1);
    let compress_level: u8 = 9;
    split_layer(tar_path, split_names, split_map, work_path, out_path, compress_level)?;

    let os_path = Path::new("tests/out_compress_best/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path)?;
    let os_right =
        format!("93caf9860cd0974f5a0365874bfa6c38730cacdc6ddf4a7a12961bde296b5fda");
    assert_eq!(os_hash, os_right);

    Ok(())
}
#[cfg(test)]
use std::path::Path;
use std::fs;

use ctor::{ctor, dtor};
use lazy_static::lazy_static;

use layer_sword::client::cli_main;
use layer_sword::util::fetch_file_sha256;
use layer_sword::errors::LayerSwordError;

type Result<T> = core::result::Result<T, LayerSwordError>;

lazy_static! {
    static ref DIR_VEC: Vec<String> = vec![
        "tests/out_basic", "tests/test_basic",
        "tests/out_split_negatives", "tests/test_split_negatives",
        "tests/out_split_config", "tests/test_split_config",
        "tests/out_merge_basic", "tests/test_merge_basic"
    ].iter().map(|s| s.to_string()).collect();
}

#[ctor]
fn before() {
    let _ = env_logger::builder().is_test(true).try_init();
    for dir_str in DIR_VEC.clone() {
        let dir_path = Path::new(&dir_str);
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).unwrap();
        }
        fs::create_dir(dir_path).unwrap();
    }
}

#[dtor]
fn after() {
    for dir_path in DIR_VEC.clone() {
        let dir_path = Path::new(&dir_path);
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).unwrap();
        }
    }
}

#[test]
fn test_split_basic() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,3,1",
        "-w", "tests/test_basic",
        "-o", "tests/out_basic",
        "-t", "tests/data/base.tar"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let os_path = Path::new("tests/out_basic/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_basic/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("022bac800ffd3c66e7ea84b3dff763624f15c4d587c6050ad46c17a63dbd154d");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_basic/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("e825bb228b29bff204c83ea68b78c652e28b38b6449f93e8d7ea3b26411eae1b");
    assert_eq!(app_hash, app_right);
    Ok(())
}

#[test]
fn test_split_negatives() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-t", "tests/data/base.tar",
        "-l", "1,3,-1",
        "-o", "tests/out_split_negatives",
        "-n", "os,lib,app",
        "-w", "tests/test_split_negatives"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let os_path = Path::new("tests/out_split_negatives/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_negatives/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("022bac800ffd3c66e7ea84b3dff763624f15c4d587c6050ad46c17a63dbd154d");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_negatives/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("e825bb228b29bff204c83ea68b78c652e28b38b6449f93e8d7ea3b26411eae1b");
    assert_eq!(app_hash, app_right);
    Ok(())
}

#[test]
fn test_split_config() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-t", "tests/data/base.tar",
        "-c", "tests/data/config.json",
        "-o", "tests/out_split_config",
        "-w", "tests/test_split_config"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let os_path = Path::new("tests/out_split_config/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("544ceec3428dbf1f6be213376b6346e1ee674ead3f83c132435e7e0d3ec02ae0");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_config/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("022bac800ffd3c66e7ea84b3dff763624f15c4d587c6050ad46c17a63dbd154d");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_config/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("e825bb228b29bff204c83ea68b78c652e28b38b6449f93e8d7ea3b26411eae1b");
    assert_eq!(app_hash, app_right);
    Ok(())
}

#[test]
fn test_merge_basic() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "merge",
        "-t", "tests/data/splits_base",
        "-o", "tests/out_merge_basic",
        "-w", "tests/test_merge_basic"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let tar_path = Path::new("tests/out_merge_basic/merge.tar");
    let tar_hash = fetch_file_sha256(tar_path);
    let tar_right =
        format!("dee5fa9709718753d732542ec0bf5c035a1146dd8792d125f1d6c0589b4e23d6");
    assert_eq!(tar_hash, tar_right);
    Ok(())
}
#[cfg(test)]
mod common;

use std::path::Path;

use layer_sword::client::cli_main;
use layer_sword::util::fetch_file_sha256;
use layer_sword::errors::LayerSwordError;

use common::{testcase_initial, testcase_destroy};

type Result<T> = core::result::Result<T, LayerSwordError>;

#[test]
fn test_split_basic() -> Result<()> {
    testcase_initial(vec!["tests/work_split_basic", "tests/out_split_basic"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,3,1",
        "-w", "tests/work_split_basic",
        "-o", "tests/out_split_basic",
        "-t", "tests/data/base.tar"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let os_path = Path::new("tests/out_split_basic/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_basic/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("87134c9d4507bfe21be863ecf4ff90a0392bd08ee6a4ad803f8b9c81c1e0318f");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_basic/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("6f254b36aca46cd037ca455f0843efba982e7ed338d88c04106096a2f3afd6cc");
    assert_eq!(app_hash, app_right);

    testcase_destroy(vec!["tests/work_split_basic", "tests/out_split_basic"]);
    Ok(())
}

#[test]
fn test_split_negatives() -> Result<()> {
    testcase_initial(vec!["tests/work_split_negatives", "tests/out_split_negatives"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-t", "tests/data/base.tar",
        "-l", "1,3,-1",
        "-w", "tests/work_split_negatives",
        "-o", "tests/out_split_negatives",
        "-n", "os,lib,app"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let os_path = Path::new("tests/out_split_negatives/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_negatives/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("87134c9d4507bfe21be863ecf4ff90a0392bd08ee6a4ad803f8b9c81c1e0318f");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_negatives/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("6f254b36aca46cd037ca455f0843efba982e7ed338d88c04106096a2f3afd6cc");
    assert_eq!(app_hash, app_right);

    testcase_destroy(vec!["tests/work_split_negatives", "tests/out_split_negatives"]);
    Ok(())
}

#[test]
fn test_split_config() -> Result<()> {
    testcase_initial(vec!["tests/work_split_config", "tests/out_split_config"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-t", "tests/data/base.tar",
        "-c", "tests/data/config.json",
        "-w", "tests/work_split_config",
        "-o", "tests/out_split_config"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let os_path = Path::new("tests/out_split_config/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_config/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("87134c9d4507bfe21be863ecf4ff90a0392bd08ee6a4ad803f8b9c81c1e0318f");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_config/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("6f254b36aca46cd037ca455f0843efba982e7ed338d88c04106096a2f3afd6cc");
    assert_eq!(app_hash, app_right);

    testcase_destroy(vec!["tests/work_split_config", "tests/out_split_config"]);
    Ok(())
}

#[test]
fn test_merge_basic() -> Result<()> {
    testcase_initial(vec!["tests/work_merge_basic", "tests/out_merge_basic"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "merge",
        "-t", "tests/data/splits_base",
        "-w", "tests/work_merge_basic",
        "-o", "tests/out_merge_basic"].iter().map(|s| s.to_string()).collect();
    cli_main(args)?;

    let tar_path = Path::new("tests/out_merge_basic/merge.tar");
    let tar_hash = fetch_file_sha256(tar_path);
    let tar_right =
        format!("a82e3d4bcf3194ec7841f6f1f2b4ce34d1107c23ef4e42d4e5073224858cc56b");
    assert_eq!(tar_hash, tar_right);

    testcase_destroy(vec!["tests/work_merge_basic", "tests/out_merge_basic"]);
    Ok(())
}
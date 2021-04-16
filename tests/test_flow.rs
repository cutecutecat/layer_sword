#[cfg(test)]
use std::path::Path;
use std::fs;
use std::collections::HashMap;

use ctor::ctor;
use lazy_static::lazy_static;

use layer_sword::util::{init_path, extract_tar, fetch_file_sha256};
use layer_sword::dominator::base::BaseDominator;
use layer_sword::inspector::base::BaseInspector;
use layer_sword::errors::{LayerSwordError, raise};
use layer_sword::inspector::Inspect;
use layer_sword::split::Split;
use layer_sword::merge::Merge;

type Result<T> = core::result::Result<T, LayerSwordError>;

lazy_static! {
    static ref DIR_VEC: Vec<String> = vec![
        "tests/out_init_path", "tests/test_init_path",
        "tests/out_inspect", "tests/test_inspect",
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
            fs::remove_dir_all(dir_path).unwrap_or_else(|_| {});
        }
        raise(fs::create_dir(dir_path));
    }
}

#[test]
fn test_init_path() -> Result<()> {
    log::info!("Test for 'init_path' function.");
    let work_path = Path::new("tests/test_init_path");
    let out_path = Path::new("tests/out_init_path");
    init_path(work_path, out_path);
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
    let out_path = Path::new("tests/out_inspect");
    init_path(work_path, out_path);
    let merge_path = Path::new("tests/test_inspect/merge");
    extract_tar(tar_path, merge_path);

    let inspector = BaseInspector {};
    inspector.inspect(merge_path)?;

    Ok(())
}

#[test]
fn test_split_layer() -> Result<()> {
    log::info!("Test for basic split procedure.");
    let out_path = Path::new("tests/out_split_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_layer");
    init_path(work_path, out_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    split_names.push(format!("app"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), 3);
    split_map.insert(format!("app"), 1);
    let compress_level: u8 = 6;

    let inspector = BaseInspector {};
    let dominator = BaseDominator {};
    dominator.split_layer(
        Box::new(inspector),
        tar_path,
        split_names,
        split_map,
        work_path,
        out_path,
        compress_level)?;

    let os_path = Path::new("tests/out_split_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("87134c9d4507bfe21be863ecf4ff90a0392bd08ee6a4ad803f8b9c81c1e0318f");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_layer/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("6f254b36aca46cd037ca455f0843efba982e7ed338d88c04106096a2f3afd6cc");
    assert_eq!(app_hash, app_right);

    Ok(())
}


#[test]
fn test_deduction() -> Result<()> {
    log::info!("Test for auto-deduction inside split procedure.");
    let out_path = Path::new("tests/out_deduction");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_deduction");
    init_path(work_path, out_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    split_names.push(format!("app"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), -1);
    split_map.insert(format!("app"), 1);
    let compress_level: u8 = 6;

    let inspector = BaseInspector {};
    let dominator = BaseDominator {};
    dominator.split_layer(
        Box::new(inspector),
        tar_path,
        split_names,
        split_map,
        work_path,
        out_path,
        compress_level)?;

    let os_path = Path::new("tests/out_deduction/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_deduction/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("87134c9d4507bfe21be863ecf4ff90a0392bd08ee6a4ad803f8b9c81c1e0318f");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_deduction/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("6f254b36aca46cd037ca455f0843efba982e7ed338d88c04106096a2f3afd6cc");
    assert_eq!(app_hash, app_right);

    Ok(())
}

#[test]
fn test_split_four_layer() -> Result<()> {
    log::info!("Test for four split dividing procedure.");
    let out_path = Path::new("tests/out_split_four_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_four_layer");
    init_path(work_path, out_path);
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

    let inspector = BaseInspector {};
    let dominator = BaseDominator {};
    dominator.split_layer(
        Box::new(inspector),
        tar_path,
        split_names,
        split_map,
        work_path,
        out_path,
        compress_level)?;

    let os_path = Path::new("tests/out_split_four_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let sys_path = Path::new("tests/out_split_four_layer/sys.tar.gz");
    let sys_hash = fetch_file_sha256(sys_path);
    let sys_right =
        format!("062f2165910f5699aa7509fabed901c494d503c413491241b703ad51c2f00dd4");
    assert_eq!(sys_hash, sys_right);

    let lib_path = Path::new("tests/out_split_four_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("2b19936c3c03947fa115b0c74321e8629c6f5b1dcd189e445384ae0ad72903e3");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_split_four_layer/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("8e654b72a036bbe9c4557f35c45f605830bff5249c9e36fbef5d61542de9605d");
    assert_eq!(app_hash, app_right);

    Ok(())
}

#[test]
fn test_split_two_layer() -> Result<()> {
    log::info!("Test for two split dividing procedure.");
    let out_path = Path::new("tests/out_split_two_layer");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_split_two_layer");
    init_path(work_path, out_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), -1);
    let compress_level: u8 = 6;

    let inspector = BaseInspector {};
    let dominator = BaseDominator {};
    dominator.split_layer(
        Box::new(inspector),
        tar_path,
        split_names,
        split_map,
        work_path,
        out_path,
        compress_level)?;

    let os_path = Path::new("tests/out_split_two_layer/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("78847ae9c6eef8cd1e84fd76d244bcc96ce45f60b6166a0a0a16ff8e858c8da4");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_split_two_layer/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("ec151b9edfdd803fe50365e3415248ef6648b89aed94933a26e8f51fdd9e1569");
    assert_eq!(lib_hash, lib_right);

    Ok(())
}

#[test]
fn test_merge() -> Result<()> {
    log::info!("Test for basic merge procedure.");
    let target_path = Path::new("tests/data/splits_base");
    let work_path = Path::new("tests/test_merge");
    let out_path = Path::new("tests/out_merge");
    init_path(work_path, out_path);

    let inspector = BaseInspector {};
    let dominator = BaseDominator {};
    dominator.merge_layer(Box::new(inspector), target_path, work_path, out_path)?;

    let tar_path = Path::new("tests/out_merge/merge.tar");
    let tar_hash = fetch_file_sha256(tar_path);
    let tar_right =
        format!("a82e3d4bcf3194ec7841f6f1f2b4ce34d1107c23ef4e42d4e5073224858cc56b");
    assert_eq!(tar_hash, tar_right);
    Ok(())
}

#[test]
fn test_compress_best() -> Result<()> {
    log::info!("Test for compress at best level.");
    let out_path = Path::new("tests/out_compress_best");
    let tar_path = Path::new("tests/data/base.tar");
    let work_path = Path::new("tests/test_compress_best");
    init_path(work_path, out_path);
    let mut split_names: Vec<String> = Vec::new();
    split_names.push(format!("os"));
    split_names.push(format!("lib"));
    split_names.push(format!("app"));
    let mut split_map: HashMap<String, i16> = HashMap::new();
    split_map.insert(format!("os"), 1);
    split_map.insert(format!("lib"), -1);
    split_map.insert(format!("app"), 1);
    let compress_level: u8 = 9;

    let inspector = BaseInspector {};
    let dominator = BaseDominator {};
    dominator.split_layer(
        Box::new(inspector),
        tar_path,
        split_names,
        split_map,
        work_path,
        out_path,
        compress_level)?;

    let os_path = Path::new("tests/out_compress_best/os.tar.gz");
    let os_hash = fetch_file_sha256(os_path);
    let os_right =
        format!("4d8e2ffdb06608da91278e5bfa3aff87bd69c7e88f562bfa62f1e2274ebfa43a");
    assert_eq!(os_hash, os_right);

    let lib_path = Path::new("tests/out_compress_best/lib.tar.gz");
    let lib_hash = fetch_file_sha256(lib_path);
    let lib_right =
        format!("37ebf36672232ee91a03e1ec0f33bd5f3063f380cf9e50d048d1df3e582e6057");
    assert_eq!(lib_hash, lib_right);

    let app_path = Path::new("tests/out_compress_best/app.tar.gz");
    let app_hash = fetch_file_sha256(app_path);
    let app_right =
        format!("25b75bc8a75d4a2be082d9e5a2c6dd1e86641fc22e39158aedcf911d62c3296e");
    assert_eq!(app_hash, app_right);

    Ok(())
}
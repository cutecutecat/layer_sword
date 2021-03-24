#[cfg(test)]
use std::path::Path;
use std::fs;

use ctor::ctor;
use lazy_static::lazy_static;

use layer_sword::client::cli_main;
use layer_sword::errors::{LayerSwordError};

type Result<T> = core::result::Result<T, LayerSwordError>;

lazy_static! {
    static ref DIR_VEC: Vec<String> = vec![
    "tests/out_split_conflict", "tests/test_split_conflict",
    "tests/out_split_no_info", "tests/out_split_no_info",
    "tests/out_split_no_target", "tests/test_split_no_target",
    "tests/out_merge_no_target", "tests/test_merge_no_target",
    "tests/out_split_bad_extension", "tests/test_split_bad_extension",
    "tests/out_split_bad_info", "tests/test_split_bad_info"
    ].iter().map(|s| s.to_string()).collect();
}

#[ctor]
fn before() {
    env_logger::builder().is_test(true).try_init().unwrap_or_else(|_| {});
    for dir_str in DIR_VEC.clone() {
        let dir_path = Path::new(&dir_str);
        if dir_path.exists() {
            fs::remove_dir_all(dir_path).unwrap();
        }
        fs::create_dir(dir_path).unwrap();
    }
}

#[test]
fn test_blank() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::TerminalError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_split_conflict() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,3,1",
        "-w", "tests/test_split_conflict",
        "-c", "tests/data/config.json",
        "-o", "tests/out_split_conflict",
        "-t", "tests/data/base.tar"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::TerminalError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_split_no_info() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-w", "tests/test_split_no_info",
        "-o", "tests/out_split_no_info",
        "-t", "tests/data/base.tar"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::TerminalError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_split_no_target() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-w", "tests/test_split_no_target",
        "-c", "tests/data/config.json",
        "-o", "tests/out_split_no_target"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::TerminalError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_merge_no_target() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "merge",
        "-o", "tests/out_merge_no_target",
        "-w", "tests/test_merge_no_target"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::TerminalError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_split_bad_extension() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-w", "tests/test_split_bad_extension",
        "-c", "tests/data/config.json",
        "-o", "tests/out_split_bad_extension",
        "-t", "tests/data/config.json"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::FileCheckError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_split_bad_info() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,4,1",
        "-w", "tests/test_split_bad_info",
        "-o", "tests/out_split_bad_info",
        "-t", "tests/data/base.tar"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        LayerSwordError::FileCheckError { .. } => {
            println!("{}", e);
            Err(e)
        }
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}
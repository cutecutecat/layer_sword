#[cfg(test)]
use std::path::Path;
use std::fs;

use ctor::{ctor, dtor};
use lazy_static::lazy_static;
use simple_logger::SimpleLogger;

use layer_sword::client::cli_main;
use layer_sword::errors::{Result, Error};

lazy_static! {
    static ref DIR_VEC: Vec<String> = vec![
    "tests/out_conflicts", "tests/test_conflicts"
    ].iter().map(|s| s.to_string()).collect();
}

#[ctor]
fn before() {
    SimpleLogger::new().init().unwrap();
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
fn test_blank() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        Error::ClapError { .. } => Err(e),
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}

#[test]
fn test_conflicts() -> Result<()> {
    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,3,1",
        "-w", "tests/test_conflicts",
        "-c", "tests/data/config.json",
        "-o", "tests/out_conflicts",
        "-t", "tests/data/base.tar"].iter().map(|s| s.to_string()).collect();
    let result = cli_main(args);
    assert!(result.is_err());
    let error_chk = result.or_else(|e| match e {
        Error::ClapError { .. } => Err(e),
        _ => Ok(())
    });
    assert!(error_chk.is_err());
    Ok(())
}
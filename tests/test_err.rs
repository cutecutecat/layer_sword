#[cfg(test)]
mod common;

use layer_sword::client::cli_main;
use layer_sword::errors::LayerSwordError;

use common::{testcase_initial, testcase_destroy};

type Result<T> = core::result::Result<T, LayerSwordError>;

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
    testcase_initial(vec!["tests/work_split_conflict", "tests/out_split_conflict"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,3,1",
        "-w", "tests/work_split_conflict",
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

    testcase_destroy(vec!["tests/work_split_conflict", "tests/out_split_conflict"]);
    Ok(())
}

#[test]
fn test_split_no_info() -> Result<()> {
    testcase_initial(vec!["tests/work_split_no_info", "tests/out_split_no_info"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-w", "tests/work_split_no_info",
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

    testcase_destroy(vec!["tests/work_split_no_info", "tests/out_split_no_info"]);
    Ok(())
}

#[test]
fn test_split_no_target() -> Result<()> {
    testcase_initial(vec!["tests/work_split_no_target", "tests/out_split_no_target"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-w", "tests/work_split_no_target",
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

    testcase_destroy(vec!["tests/work_split_no_target", "tests/out_split_no_target"]);
    Ok(())
}

#[test]
fn test_merge_no_target() -> Result<()> {
    testcase_initial(vec!["tests/work_merge_no_target", "tests/out_merge_no_target"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "merge",
        "-w", "tests/work_merge_no_target",
        "-o", "tests/out_merge_no_target"].iter().map(|s| s.to_string()).collect();
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

    testcase_destroy(vec!["tests/work_merge_no_target", "tests/out_merge_no_target"]);
    Ok(())
}

#[test]
fn test_split_bad_extension() -> Result<()> {
    testcase_initial(vec!["tests/work_split_bad_extension", "tests/out_split_bad_extension"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-w", "tests/work_split_bad_extension",
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

    testcase_destroy(vec!["tests/work_split_bad_extension", "tests/out_split_bad_extension"]);
    Ok(())
}

#[test]
fn test_split_bad_info() -> Result<()> {
    testcase_initial(vec!["tests/work_split_bad_info", "tests/out_split_bad_info"]);

    let args: Vec<String> = vec![
        "target/release/layer_sword.exe",
        "split",
        "-n", "os,lib,app",
        "-l", "1,4,1",
        "-w", "tests/work_split_bad_info",
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

    testcase_destroy(vec!["tests/work_split_bad_info", "tests/out_split_bad_info"]);
    Ok(())
}
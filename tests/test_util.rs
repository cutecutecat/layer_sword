#[cfg(test)]
use std::path::Path;

use layer_sword::util::{fetch_string_sha256, fetch_file_sha256, get_stack_id};
use layer_sword::errors::LayerSwordError;

type Result<T> = core::result::Result<T, LayerSwordError>;

#[test]
fn test_string_sha256() -> Result<()> {
    log::info!("Test for 'fetch_string_sha256' function");
    let target_string = format!("layer_sword");
    let hash = fetch_string_sha256(&target_string);
    let right = format!("4deaf80f304870a2bc7a9a1f3a952d86d3db19e01094f28cad8a06e1ad6fb2c1");
    assert_eq!(hash, right);
    Ok(())
}

#[test]
fn test_file_sha256() -> Result<()> {
    log::info!("Test for 'fetch_file_sha256' function");
    let tar_path = Path::new("tests/data/base.tar");
    let hash = fetch_file_sha256(tar_path);
    let right = format!("8de3e6511bb095f7d7d4133e877391f6ee1ec2bfda022bc24e2443277d3966b6");
    assert_eq!(hash, right);
    Ok(())
}

#[test]
fn test_stack_id() -> Result<()> {
    log::info!("Test for 'get_stack_id' function");
    let first_id = format!("a6e99f9b50e1bb8366d55fee15116a4da796c6bc37ebec09e7e77ec4cfa629fb");
    let second_id = format!("8de3e6511bb095f7d7d4133e877391f6ee1ec2bfda022bc24e2443277d3966b6");
    let hash = get_stack_id(&first_id, &second_id);
    let right = format!("a5a8033bc04ce56c3f0982deaabad8125581856fd702262772511efd69b18de9");
    assert_eq!(hash, right);
    Ok(())
}
#[cfg(test)]
use std::path::Path;

use layer_sword::util::fetch_file_sha256;
use layer_sword::errors::LayerSwordError;

type Result<T> = core::result::Result<T, LayerSwordError>;

#[test]
fn test_file_sha256() -> Result<()> {
    log::info!("Test for 'fetch_file_sha256' function");
    let tar_path = Path::new("tests/data/base.tar");
    let hash = fetch_file_sha256(tar_path);
    let right = format!("8de3e6511bb095f7d7d4133e877391f6ee1ec2bfda022bc24e2443277d3966b6");
    assert_eq!(hash, right);
    Ok(())
}
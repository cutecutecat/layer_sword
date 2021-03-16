mod util;
mod errors;
mod inspector;
mod split;
mod merge;
mod client;

use std::env;

use errors::Result;

// TODO: 優化clone
// TODO: 测试用例
// TODO: 文档
use crate::client::cli_main;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    cli_main(args)?;
    Ok(())
}
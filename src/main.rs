mod dominator;
mod inspector;
mod split;
mod merge;
mod client;
mod validator;
mod util;
mod errors;

use std::env;

use crate::client::cli_main;
use crate::errors::raise_err;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(e) = cli_main(args) {
        env_logger::builder().is_test(false).try_init().unwrap_or_else(|_| {});
        raise_err(e);
    }
}
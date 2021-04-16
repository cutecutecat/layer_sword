#![feature(associated_type_defaults)]
#![feature(destructuring_assignment)]

mod dominator;
mod inspector;
mod split;
mod merge;
mod client;
mod util;
mod errors;

use std::env;

use log::error;

use crate::client::cli_main;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Err(e) = cli_main(args) {
        env_logger::builder().is_test(false).try_init().unwrap_or_else(|_| {});
        error!("{}", e);
    }
}
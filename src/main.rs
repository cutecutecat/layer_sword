#![feature(associated_type_defaults)]
#![feature(destructuring_assignment)]

mod util;
mod errors;
mod dominator;
mod inspector;
mod split;
mod merge;
mod client;

use std::env;

use log::error;

use crate::client::cli_main;

fn main(){
    let args: Vec<String> = env::args().collect();
    if let Err(e) = cli_main(args){
        error!("{}", e);
    }
}
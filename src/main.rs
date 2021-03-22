mod util;
mod errors;
mod inspector;
mod split;
mod merge;
mod client;

use std::env;
use std::error::Error;

use log::error;

use crate::client::cli_main;


fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().is_test(false).try_init().unwrap();

    let args: Vec<String> = env::args().collect();
    if let Err(e) = cli_main(args){
        error!("{}", e);
        return Err(Box::new(e));
    }
    Ok(())
}
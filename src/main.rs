mod util;
mod errors;
mod inspector;
mod split;
mod merge;
mod client;

use std::env;

use log::error;

use crate::client::cli_main;
use crate::errors::LayerSwordError;


fn main() -> Result<(), LayerSwordError>{
    let args: Vec<String> = env::args().collect();
    if let Err(e) = cli_main(args){
        error!("{}", e);
        return Err(e);
    }
    Ok(())
}
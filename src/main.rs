use std::error::Error;

use parse::{get_config_path, read_config};
use ssh::connect_to_cse;

mod parse;
mod ssh;

fn main() -> Result<(), Box<dyn Error>> {
    get_config_path();
    read_config();
    connect_to_cse()
}

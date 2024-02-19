use std::error::Error;

use parse::construct_ssh_config;
use ssh::connect_to_cse;

mod parse;
mod ssh;

fn main() -> Result<(), Box<dyn Error>> {
    // get_config_path();
    construct_ssh_config();
    connect_to_cse()
}

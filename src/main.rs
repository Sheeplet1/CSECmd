use std::{env::args, error::Error};

use parse::construct_ssh_config;
use ssh::{connect_and_exec, Config};

mod parse;
mod ssh;

fn main() -> Result<(), Box<dyn Error>> {
    let mut config: Config = construct_ssh_config();

    // Parse cmd line arguments for the command

    // csecmd <command>
    let command = args().nth(1).expect("A command is required.");
    config.command = command;

    connect_and_exec(config)?;

    todo!()
}

use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use home::home_dir;

pub enum Auth {
    Password,
    // Key
}

struct Config {
    server_addr: String,
    port: u16,
    user: String,
    auth_type: Auth,
    password: String,
}

fn default_config_contents() -> &'static str {
    include_str!("../example.toml")
}

pub fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    // check if config.toml exists at $HOME/.config/csecmd/
    let home_dir = home_dir().unwrap();
    let mut conf_path = PathBuf::new();
    conf_path.push(home_dir);
    conf_path.push(".config/csecmd");

    if !conf_path.is_dir() {
        // Create directory at path
        println!("Initialised csecmd directory at ~/.config");
        fs::create_dir_all(&conf_path).unwrap_or_else(|e| eprintln!("Error: {e}"));
    }

    conf_path.push("config.toml");

    Ok(conf_path)
}

pub fn read_config() -> Result<(), Box<dyn Error>> {
    let mut conf_path = get_config_path()?;

    // If config file does not exist, then create using the scaffold
    // from example.toml
    if !conf_path.exists() {
        let mut file = File::create(&conf_path)?;
        let default_contents = default_config_contents();

        file.write_all(default_contents.as_bytes())?;
    }

    Ok(())
}

use crate::ssh::{Auth, Config};
use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use home::home_dir;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AuthType {
    Password,
    // Key
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    server: ServerConfig,
    auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    addr: String,
    port: u16,
    user: String,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    auth_type: AuthType,
    password: Option<String>, // Key stuff here
}

fn default_config_contents() -> &'static str {
    include_str!("../example.toml")
}

pub fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    // check if config.toml exists at $HOME/.config/csecmd/
    let home_dir = home_dir().expect("All systems have a home directory");
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

fn read_config() -> Result<TomlConfig, Box<dyn Error>> {
    let conf_path = get_config_path()?;

    // If config file does not exist, then create using the scaffold
    // from example.toml
    if !conf_path.exists() {
        let mut file = File::create(&conf_path)?;
        let default_contents = default_config_contents();

        file.write_all(default_contents.as_bytes())?;
        eprintln!(
            "Please fill out recommended fields for config, located at {:?}",
            conf_path
        );
        std::process::exit(1)
    }

    let contents = fs::read_to_string(&conf_path)?;
    let config: TomlConfig = toml::from_str(&contents)?;

    Ok(config)
}

pub fn construct_ssh_config() -> Config {
    // Get contents from config.toml
    let config: TomlConfig = read_config().unwrap_or_else(|e| {
        println!("Error reading config: {e}");
        std::process::exit(1)
    });

    // If password has not been changed, then prompt user for ssh password
    let auth: Auth = match config.auth.auth_type {
        AuthType::Password => {
            let pw = match config.auth.password {
                Some(p) => p,
                None => rpassword::prompt_password("ssh password: ").unwrap_or_default(),
            };
            Auth::Password(pw)
        } // TODO: Add key here
    };

    // Construct config struct
    Config {
        server_addr: format!("{}:{}", config.server.addr, config.server.port),
        username: config.server.user,
        auth,
        command: String::new(),
    }
}

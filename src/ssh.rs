use ssh2::Session;
use std::{error::Error, net::TcpStream};

pub enum Auth {
    Password(String),
    // AuthKey(AuthKey) - define AuthKey
}

pub struct Config {
    pub server_addr: String,
    pub username: String,
    pub auth: Auth,
    pub command: String,
}

pub fn connect_to_cse() -> Result<(), Box<dyn Error>> {
    let tcp = TcpStream::connect("cse.unsw.edu.au:22")?;
    println!("Connecting to CSE UNSW...");

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    println!("Handshake successful!");

    Ok(())
}

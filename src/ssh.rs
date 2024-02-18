use ssh2::Session;
use std::{error::Error, net::TcpStream};

pub fn connect_to_cse() -> Result<(), Box<dyn Error>> {
    let tcp = TcpStream::connect("cse.unsw.edu.au:22")?;
    println!("Connecting to CSE UNSW...");

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    println!("Handshake successful!");

    Ok(())
}

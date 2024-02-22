use ignore::Walk;
use serde::Deserialize;
use ssh2::{Session, Sftp};
use std::{
    error::Error,
    fs::File,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub enum Auth {
    Password(String),
    // AuthKey(AuthKey) - define AuthKey
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_addr: String,
    pub username: String,
    pub auth: Auth,
    pub command: String,
}

/// Connect to CSE UNSW's servers, upload the local directory and execute
/// the given command. Returns the standard output from the command.
pub fn connect_and_exec(config: Config) -> Result<(), Box<dyn Error>> {
    let tcp = TcpStream::connect(config.server_addr)?;
    println!("Connecting to CSE UNSW...");

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    println!("Handshake successful!");

    match config.auth {
        Auth::Password(p) => sess.userauth_password(config.username.as_str(), p.as_str())?,
    };

    println!("Authentication as {}", config.username);

    let sftp = sess.sftp()?;

    // using current time-stamp as file name - realistically, this name doesn't
    // matter as we will be cleaning it up after we've completed our command so
    // as to reduce space consumption and congestion.
    let temp_dir_name = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();
    let remote_dir = format!(".csecmd_dump/temp/{}", temp_dir_name);
    let remote_dir_path = Path::new(&remote_dir);

    sftp_mkdir_recur(&sftp, remote_dir_path)?;

    // Set up sandbox directory which will contain the uploaded files.
    let sandbox_path = remote_dir_path.join("sandbox");
    sftp.mkdir(sandbox_path.as_path(), 0o755)?;

    let local_dir = "./";
    upload_dir(&sftp, Path::new(local_dir), &sandbox_path)?;
    println!("Synced local files to remote...");

    // NOTE: Executing command stuff
    let mut channel = sess.channel_session()?;
    channel.exec(format!("cd ~/{}", remote_dir).as_str())?;

    let mut command_file = sftp.create(&remote_dir_path.join("command.txt"))?;
    command_file.write_all(config.command.as_bytes())?;

    // TODO: Execute given command on the uploaded directory.

    // TODO: Read output into a buffer and print to this local machine's standard
    // output.

    // let mut output = String::new();
    // channel.read_to_string(&mut output)?;
    // println!("===== Output =====");
    // println!("{:#?}", output);
    //
    // let _ = channel.wait_close();

    // TODO: Add a clean-up part which deletes the sandbox files, otherwise
    // this will eventually cap out and/or clutter the storage allocation.

    println!("Disconnected from CSE UNSW...");

    Ok(())
}

/// Recursively make directories on the remote server to copy the path provided.
fn sftp_mkdir_recur(sftp: &Sftp, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut curr_path = PathBuf::new();
    for component in path.components() {
        curr_path.push(component);

        // TODO: Check that path is not to a file, if it is, then exit the
        // function.

        if let Ok(metadata) = sftp.stat(curr_path.as_path()) {
            if metadata.is_dir() {
                continue;
            }

            return Err(format!("{:?} is not a directory", curr_path).into());
        }
        sftp.mkdir(curr_path.as_path(), 0o755)?;
    }

    Ok(())
}

/// Uploads a file located at the `local_path` to the remote server at the `remote_path`.
fn upload_file(sftp: &Sftp, local_path: &Path, remote_path: &Path) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(local_path)?;
    let mut metadata = Vec::new();

    file.read_to_end(&mut metadata)?;

    let mut remote_file = sftp.create(remote_path)?;
    remote_file.write_all(&metadata)?;

    Ok(())
}

/// Upload the current directory to the remote server.
pub fn upload_dir(
    sftp: &Sftp,
    local_path: &Path,
    remote_base_path: &Path,
) -> Result<(), Box<dyn Error>> {
    // TODO: Add styling here - spinner/progress bar for user

    for res in Walk::new("./") {
        match res {
            Ok(entry) => {
                let path = entry.path();
                if let Ok(strip_path) = path.strip_prefix(local_path) {
                    let remote_path = remote_base_path.join(strip_path);

                    // If path is a directory, attempt to create the directory.
                    if path.is_dir() {
                        match sftp.mkdir(&remote_path, 0o755) {
                            Ok(_) => (),
                            Err(err) => eprintln!(
                                // BUG: /sandbox/ creation error
                                //  SFTP(4) : Failure
                                "Directory creation error at {:?}\n{:?}",
                                remote_path, err
                            ),
                        }
                    } else {
                        upload_file(sftp, path, &remote_path)?;
                    }
                }
            }
            Err(err) => eprintln!("Error opening local file: {}", err),
        }
    }

    Ok(())
}

fn clean_up() -> Result<(), Box<dyn Error>> {
    todo!()
}

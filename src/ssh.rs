use console::style;
use ignore::Walk;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use ssh2::{Session, Sftp};
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    time::Duration,
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
    println!("{} Connecting to CSE UNSW", style("[1/7]").bold().dim());

    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    println!("{} Handshake successful!", style("[2/7]").bold().dim());

    match config.auth {
        Auth::Password(p) => sess.userauth_password(config.username.as_str(), p.as_str())?,
    };

    println!(
        "{} Authentication as {}",
        style("[3/7]").bold().dim(),
        style(config.username).bold().yellow()
    );

    let sftp = sess.sftp()?;

    // using current time-stamp as file name - realistically, this name doesn't
    // matter as we will be cleaning it up after we've completed our command so
    // as to reduce space consumption and congestion.
    let temp_dir_name = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S").to_string();

    let remote_dir = format!(".csecmd_dump/temp/{}", temp_dir_name);
    let remote_dir_path = Path::new(&remote_dir);

    sftp_mkdir_recur(&sftp, remote_dir_path)?;

    // Set up sandbox directory which will contain the uploaded files.
    let local_dir = "./";
    let sandbox_path = remote_dir_path.join("sandbox");
    upload_dir(&sftp, Path::new(local_dir), &sandbox_path)?;
    println!(
        "{} Synced local files to remote",
        style("[4/7]").bold().dim()
    );

    let mut command_file = sftp.create(&remote_dir_path.join("command.txt"))?;
    command_file.write_all(config.command.as_bytes())?;

    let mut channel = sess.channel_session()?;

    let mut prefix = String::new();
    prefix.push_str(&format!("cd {}/sandbox && ", remote_dir));
    let command = format!("{}{}", prefix, config.command);
    channel.exec(&command)?;

    println!("{} Executed command", style("[5/7]").bold().dim());
    sess.set_blocking(false);

    println!(
        "{} {} {}",
        style("==========").bold().magenta(),
        style("Output").bold().italic().magenta(),
        style("==========").bold().magenta()
    );

    let mut buffer = [0; 4096];
    loop {
        if channel.eof() {
            break;
        }

        let mut is_data_available = false;

        // trying to read standard output
        match channel.read(&mut buffer) {
            Ok(size) if size > 0 => {
                print!("{}", String::from_utf8_lossy(&buffer[..size]));
                is_data_available = true;
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }

        // trying to read standard errors
        match channel.stderr().read(&mut buffer) {
            Ok(size) if size > 0 => {
                eprint!("{}", String::from_utf8_lossy(&buffer[..size]));
                is_data_available = true;
            }
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }

        if !is_data_available {
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    println!("{}", style("============================").bold().magenta());
    sess.set_blocking(true);
    match clean_up(&sftp, remote_dir_path) {
        Ok(_) => println!(
            "{} Successful clean up on aisle 5",
            style("[6/7]").bold().dim()
        ),
        Err(e) => eprintln!("Error cleaning up: {e}"),
    }

    channel.wait_close()?;

    match channel.exit_status()? {
        0 => println!("{} Successfully left CSE", style("[7/7]").bold().dim()),
        _status => eprintln!("Exist status: Error {}", _status),
    }

    Ok(())
}

/// Recursively make directories on the remote server to copy the path provided.
fn sftp_mkdir_recur(sftp: &Sftp, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut curr_path = PathBuf::new();
    for component in path.components() {
        curr_path.push(component);

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

/// Uploads a file located at the `local_path` to the remote server at the
/// `remote_path`.
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

    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}")?;
    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.set_prefix("Syncing files...");
    pb.enable_steady_tick(Duration::from_millis(100));

    for res in Walk::new("./") {
        match res {
            Ok(entry) => {
                let path = entry.path();
                if let Ok(strip_path) = path.strip_prefix(local_path) {
                    let remote_path = remote_base_path.join(strip_path);

                    // If path is a directory, attempt to create the directory.
                    if path.is_dir() {
                        match sftp.mkdir(&remote_path, 0o755) {
                            Ok(_) => pb.set_message(
                                format!("Created remote directory: {:?}", remote_path).to_string(),
                            ),
                            Err(err) => eprintln!(
                                "Directory creation error at {:?}\n{:?}",
                                remote_path, err
                            ),
                        }
                    } else {
                        upload_file(sftp, path, &remote_path)?;
                        pb.set_message(format!("Uploaded file: {:?}", remote_path).to_string());
                    }
                }
            }
            Err(err) => eprintln!("Error opening local file: {}", err),
        }
    }

    Ok(())
}

/// Recursively clean up the temporary files which was uploaded to the remote
/// server in order to execute the command.
fn clean_up(sftp: &Sftp, remote_path: &Path) -> Result<(), Box<dyn Error>> {
    let files = sftp.readdir(remote_path)?;

    for (file, stats) in files {
        if stats.is_dir() {
            clean_up(sftp, file.as_path())?;
        } else {
            sftp.unlink(file.as_path())?;
        }
    }

    // delete the parent directory if it is empty and not locked by nfs
    if sftp.readdir(remote_path)?.is_empty() {
        sftp.rmdir(remote_path)?;
    } else {
        // BUG: .nfs file blocking deletion of parent directory
        // eprintln!("Error: could not delete directory at {:?}", remote_path);
    }

    Ok(())
}

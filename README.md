# CSECmd

CSECmd is a utility-based tool designed to assist with running UNSW CSE specific
commands such as `autotest` and `give` on a local machine.

It is powered by Rust as I want to learn Rust.

## Installation

To install CSECmd directly from source using Cargo, execute the below command:

```sh
cargo install csecmd --git git@github.com:Sheeplet1/CSECmd.git

```

## Configuration and Usage

After installation, navigate to the directory where you want to execute the
command.

```sh
cd /path/to/your/work
# Example: cd ~/cs6991/weekly_exercises/wk1/exercise01

```

To use CSECmd, run as follows:

```sh
csecmd <your_command>
# Example: csecmd "6991 autotest" or csecmd "6991 give-crate"

```

## Configuration Details

After the initial installation and execution, CSECmd will prompt you to
modify a TOML configuration file. This file contains settings for connecting
to CSE servers, including server details and authentication method.

After completing the configuration setup, re-run CSECmd in your project
directory.

### Server Configuration

```toml
[server]
addr = "cse.unsw.edu.au"    # Do not change.
port = 22                   # Do not change.
user = "z5555555"           # Change this.

```

### Authentication Configuration

At the moment, there is only password authentication. Future plans is to allow
for SSH keys and SSH agents.

#### 1. Password Authentication

```toml
[auth]
auth_type = "password"
password = "changeme"   # Optional. Recommended to fill in for convenience.

```

## Examples

TODO

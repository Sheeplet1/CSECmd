# TODO 

[] Establish a connection to cse servers 
[] Create config files based on `example.toml` 
    [] Server address and port 
    [] zID and password 
    [] This needs to be initialised in `$HOME/.config/csecmd/config.toml` and prompt
       the user to change the necessary fields.
[] Parse command line arguments for `command`
[] SFTP - upload required files for `6991 autotest` and `6991 give-crate` to work.
    [] Add ability to ignore `target` folder and `.git` folders.
[] Change directory to container directory w/ files and execute command.
[] Receive output and display onto local machine.

# Higher Level Architecture

`csecmd "6991 autotest"` -> connect to cse servers (using config files) ->
upload required files for given command -> execute command -> receive output ->
return output to local machine for display

# MVP 

I am able to run `csecmd <command>` successfully and receive the standard output.

Specifically, I want to be able to run `6991 autotest` and `6991 give-crate`.

## Process

- I want to be able to connect into the CSE server using my username and password.
- I want to be able to run `6991 autotest` and `6991 give-crate` on my local
files and directories.
- I want to be able to see the output that occurs from these commands on my
local command-line.

# Nice-haves

- No sync for files; mainly for commands that do not require file sync such as 
`6991 sturec`
- Pass environment files as args
- ssh-key authentication
- ssh agent authentication
- command to display where csecmd is pulling config from (`--config`)
- Styling for output
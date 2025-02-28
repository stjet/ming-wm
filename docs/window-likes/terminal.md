A basic, line-buffered, modal terminal.

## Usage

The terminal displays the current mode in the lower left corner. The possible modes are "INPUT", "RUNNING", "STDIN".

The terminal starts off in INPUT mode, which allows entering commands to run. If `$HOME` is set, the starting directory is that. Otherwise, it is root (`/`).

In INPUT mode, commands can be freely typed. There are a few special control sequences:

- `ctrl+p`: Brings up the last run command to the command input
- `ctrl+n`: Clears the command input

Once a command is entered, hit 'enter' to execute it. The terminal will change into "RUNNING" mode. In this mode, clicking any key except for 'i' will result in the terminal writing the current output of the running command to the window (`ctrl+c` will force the process to exit). It will also check if the command has exited, in which case the INPUT mode is returned to. Clicking the 'i' key will change the terminal to "STDIN" mode.

In STDIN mode, any keys typed followed by the 'enter' key will send those keys to the command's STDIN, if it is still running. To escape STDIN mode, use the `esc` key.

## Copy / Paste

This window-like supports the paste [shortcut](../system/shortcuts.md) (`Alt+P`) if in INPUT or STDIN mode.


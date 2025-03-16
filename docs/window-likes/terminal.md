A basic, modal terminal.

## Usage

The terminal displays the current mode in the lower left corner. The possible modes are "INPUT", "RUNNING", "STDIN".

The terminal starts off in INPUT mode, which allows entering commands to run. If `$HOME` is set, the starting directory is that. Otherwise, it is root (`/`).

In INPUT mode, commands can be freely typed. There are a few special control sequences:

- `ctrl+p`: Equivalent to the up arrow in most terminals. Brings up the previous command in the command history, and so on.
- `ctrl+n`: Equivalent to the down arrow in most terminals. Either clears the current input if not in a previous command, else brings up the next command in the command history.

Tab completion is also supported, though only for file/directory paths.

Once a command is entered, hit 'enter' to execute it. The terminal will change into "RUNNING" mode. In this mode, clicking any key except for 'i' will result in the terminal writing the current output of the running command to the window (`ctrl+c` will force the process to exit). It will also check if the command has exited, in which case the INPUT mode is returned to. Clicking the 'i' key will change the terminal to "STDIN" mode.

In STDIN mode, any keys typed followed by the 'enter' key will send those keys to the command's STDIN, if it is still running. To escape STDIN mode, use the `esc` key.

ANSI escape codes are currently not supported, and are stripped.

### Sudo

To get sudo to read from stdin, the `-S` option will need to be used (eg, `sudo -S ls`). Then switch to STDIN mode, type in the password and hit enter.

## Copy / Paste

This window-like supports the paste [shortcut](../system/shortcuts.md) (`Alt+P`) if in INPUT or STDIN mode.

## Notes

Some commands like `git diff` don't quite work well yet. Also, some command outputs are very long, but the terminal doesn't really support scrolling. Instead, redirect the output of those commands to a file and read it in Malvim (eg `git diff > diff.txt`).

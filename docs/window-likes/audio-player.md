Audio player with playlist and folder support. Requires the dev version of `alsa` lib.

## Commands

Type to write commands, backspace to delete last character, and enter to run command.

- `t`: Toggle pause/play
- `l`: Next/skip
- `j`: Volume down
- `k`: Volume up
- `b <dir>`: Set base directory (`<dir>` is path). Unless paths are absolute, they will be relative to what the base directory currently is (by default, `$HOME` is set, `/` otherwise)
- `p <dir / playlist file>`: Play audio files in `<dir>` or play the songs listed in the `<playlist file>`. Unless paths are absolute, they will be relative to the directory specified by the `b <dir>` command
- `a <dir / playlist file>`: Same as `p` but appends to the end of the queue instead of clearing the current song and the queue

Tab completion is supported for the `<dir>` and `<dir / playlist file>` arguments.

## Playlists

Example playlist file:

```
hanyuu-maigo/オノマトペ
inabakumori/*
iyowa/*
such/Hegira
```

If there is no file extension, the player assumes `.mp3`.


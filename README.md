Ming-wm is a keyboard-based, retro-themed window manager for Linux. It is neither for Wayland or the X Window System - it writes directly to the framebuffer. Inspirations include i3, Haiku, SerenityOS, and Windows98.

![example 1](/docs/images/ws1.png)
![example 2](/docs/images/ws3.png)
https://github.com/user-attachments/assets/2efc0122-80fa-48dd-8d31-f307217c2961

The [Koxinga web browser](https://github.com/stjet/koxinga) can be separately installed.

![koxinga browser example](/docs/images/koxinga.png)

## Building

Create a `password.env` file in the same directory as `build.rs`, otherwise the default password will be "password".

For best performance:

```
cargo build --release --all-features
```

The user may need to be added to the `video` group.

Exclude `--all-features` if the audio player window is not needed. To compile and use the audio player window, ALSA dev packages need to be installed (`alsa-lib-dev` on Alpine, `libasound2-dev` on Debian, `alsa-lib-devl` on Fedora, already included with `alsa-lib` on Arch).

## Installing

After building, to properly install ming-wm, run the following to put the necessary binaries, font data, and bmp files into `/usr/local/bin`:

```bash
chmod +x ./install
sudo ./install
```

Alternatively, to move the binaries to `~/.local/bin` (which probably needs to be added to `PATH`, run the following:

```bash
chmod +x local-install
sudo ./local-install
```

## Running

```
ming
```

Type in the password to unlock. Open the start menu by doing `Alt+s`, and use the `j` and `k` keys to move up and down (like Vim), and press the `Enter` key to select a category / open a window.

Usage for most of the included windows and window-likes are included in `docs/window-likes`, which can also be accessed from the "Help" entry in the start menu.

## Running on Mobile Linux

More or the less the same, but includes with an onscreen keyboard for touchscreens.

Currently, the touchscreen input device is assumed to be at `/dev/input/by-path/first-touchscreen`, but this is easily editable (see `src/bin/wm.rs`). For touchscreen support, the user running `ming` needs to have read permissions for that `/dev/input/` file.

```
ming touch
```

Optionally, in landscape mode:

```
ming touch rotate
```

<image alt="mobile example" src="/docs/images/mobile.png" width="50%">

## Philosophy

See [/docs/philosophy.md](/docs/philosophy.md) for some hopefully interesting ramblings.

## Developing Windows

Windows (may be called apps in other window managers) can be developed in any language, though it is easiest to do so in Rust because the `ming-wm-lib` crate can be used.

The `docs` directory includes a [brief introduction to writing windows](docs/system/writing_windows.md), and (incomplete) documentation on the workings of ming-wm.

See [koxinga](https://github.com/stjet/koxinga) or `src/bin` for examples.

A (very poorly written, and WIP) window is being written in Lisp Scheme: [ming-flashcards](https://github.com/stjet/ming-flashcards).

## Security

Make sure the permissions of `password.env` are so other users cannot read or write to it. If there is no plan to recompile, just delete it.

Understand the implications of adding the user to the `video` group. And if the permissions of a `/dev/input/` file was changed for touchscreen support, understand those implications too.

Obviously, don't run the executable with `sudo` or `doas`, or as the root user!

## License

Licensed under the GPLv3. The font data in the `bmps/shippori-mincho` folder are covered by the OFL. The font was created by FONTDASU. The font data in `bmps/nimbus-roman` are licensed under the AGPL. This is a very slightly modified version of the font was created by URW Studio. The font data in `bmps/nimbus-romono` is also licensed under the AGPL. This is a slightly modified version of the Nimbus Roman font by URW Studio.


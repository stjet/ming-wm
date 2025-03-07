Ming-wm is a keyboard-based, retro-themed window manager for Linux. It is single-threaded, and is neither for Wayland or the X Window System - it writes directly to the framebuffer. Inspirations include i3, Haiku, SerenityOS, and Windows98, and it is a conceptual successor to the previous [mingde](https://github.com/stjet/mingde) and [ming-os](https://github.com/stjet/ming-os).

![example 1](/docs/images/ws1.png)
![example 2](/docs/images/ws3.png)

## Building

Create a `password.env` file in the same directory as `build.rs`, otherwise the default password will be "incorrect mule lightbulb niche".

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

## Running on Mobile Linux

Running with an onscreen keyboard. The framebuffer may not be redrawn to the screen without a (real) key press. The volume down button seems to work.

`evtest` needs to be installed. Currently, the input device is assumed to be at `/dev/first-touchscreen`.

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

## Documentation

### Developing Windows

[section incomplete]

Windows (may be called apps in other window managers) can be developed in any language, though it is easiest to do so in Rust because the `ming-wm-lib` crate can be used.

### Window Usage

Usage for most of the included windows and window-likes are included in `docs/window-likes`, which can also be accessed from the "Help" entry in the start menu.

## Security

Make sure the permissions of `password.env` are so other users cannot read or write to it. If there is no plan to recompile, just delete it.

Obviously, don't run the executable with `sudo` or `doas`, or as the root user!

## License

Licensed under the GPLv3. The font data in the `bmps/shippori-mincho` folder are covered by the OFL. The font was created by FONTDASU. The font data in `bmps/nimbus-roman` are licensed under the AGPL. This is a very slightly modified version of the font was created by URW Studio. The font data in `bmps/nimbus-romono` is also licensed under the AGPL. This is a slightly modified version of the Nimbus Roman font by URW Studio.


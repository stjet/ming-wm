Ming-wm is a keyboard-based, retro-themed window manager for Linux. It is single-threaded, and is neither for Wayland or the X Window System - it writes directly to the framebuffer. Inspirations include i3, Haiku, SerenityOS, and Windows98, and it is a conceptual successor to the previous [mingde](https://github.com/stjet/mingde) and [ming-os](https://github.com/stjet/ming-os).

![example 1](/docs/images/ws1.png)
![example 2](/docs/images/ws3.png)

## Running

Create a `password.txt` file in the same directory as `build.rs`, otherwise the default password will be "incorrect mule lightbulb niche".

For best performance:
```
cargo build --release --all-features
# Either,
./target/release/main
# or
cargo run --release
```

Exclude `--all-features` if the audio player window is not needed. To compile and use the audio player window, ALSA dev packages need to be installed (`alsa-lib-dev` on Alpine, `libasound2-dev` on Debian, `alsa-lib-devl` on Fedora, already included with `alsa-lib` on Arch).

### Running on Mobile Linux

Running with an onscreen keyboard. The framebuffer may not be redrawn to the screen without a (real) key press. The volume down button seems to work.

`evtest` needs to be installed.

```
cargo build --release
./target/release/main touch
```

Optionally, in landscape mode (todo: osk may be broken in landscape mode):

```
cargo build --release
./target/release/main touch rotate
```

![mobile example](/docs/images/mobile.png)

## Philosophy

See [/docs/philosophy.md](/docs/philosophy.md) for some hopefully interesting ramblings.

## License

Licensed under the GPLv3. The font data in the `bmps/shippori-mincho` folder are covered by the OFL. The font was created by FONTDASU. The font data in `bmps/nimbus-roman` are licensed under the AGPL. This is a very slightly modified version of the font was created by URW Studio. The font data in `bmps/nimbus-romono` is also licensed under the AGPL. This is a slightly modified version of the Nimbus Roman font by URW Studio.


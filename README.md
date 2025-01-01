Ming-wm is a keyboard-based, retro-themed window manager for Linux. It is single-threaded, and is neither for Wayland or the X Window System - it writes directly to the framebuffer. Inspirations include i3, Haiku, SerenityOS, and Windows98, and it is a conceptual successor to the previous [mingde](https://github.com/stjet/mingde) and [ming-os](https://github.com/stjet/ming-os).

![example 1](/docs/images/ws1.png)
![example 2](/docs/images/ws2.png)

## Running

Create a `password.txt` file in the same directory as `build.rs`, otherwise the default password will be "incorrect mule lightbulb niche".

For best performance:
```
cargo build --release
./target/release/main
```

Though just `cargo run --release` can be done.


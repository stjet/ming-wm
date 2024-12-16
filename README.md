
## Running

For best performance:
```
cargo build --release
./target/release/main
```

Though just `cargo run --release` can be done.

## Config

Config files should be protected so they can only be written to with root privileges.

### Desktop Backgrounds

In `$XDG_CONFIG_DIR/ming-wm/desktop-background`, you can configure what the desktop background should be for each workspace. The first line decides the background for the first workspace, and so on. If lines are missing, or empty, or the config file is missing, the default green background is used.

If a line starts with "#", and is followed by 6 lowercase hex characters, then it will interpreted as a RGB colour.

If a line starts with "r", then what follows with be interpreted as a path to a BMP image file in BGRA order, and if it starts with any other character, what follows will be interpreted as a path to a BMP image file in RGBA order.

Example:

```
#008080
#003153
r./bmps/castle1440x842.bmp
r./bmps/ming1440x842.bmp
r./bmps/blur1440x842.bmp
```

//


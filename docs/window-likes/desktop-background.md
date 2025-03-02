Displays desktop backgrounds, which can be a BMP image file or a solid colour.

## Config

Config files should have the appropriated protections so they can only be written to root privileges.

In `$XDG_CONFIG_DIR/ming-wm/desktop-background`, you can configure what the desktop background should be for each workspace. The first line decides the background for the first workspace, and so on. If lines are missing, or empty, or the config file is missing, the default green background is used.

If a line starts with "#", and is followed by 6 lowercase hex characters, then it will interpreted as a RGB colour.

If a line starts with "r", then what follows with be interpreted as a path to a BMP image file in BGRA order, and if it starts with any other character, what follows will be interpreted as a path to a BMP image file in RGBA order. The path should be absolute.

Example:

```
#008080
#003153
r/home/username/Pictures/castle1440x842.bmp
r/home/username/Pictures/ming1440x842.bmp
r/home/username/Pictures/blur1440x842.bmp
```

## Unrelated: Themes Config

Not handled by the desktop background, but here anyways. To configure, create `$XDG_CONFIG_DIR/ming-wm/desktop-background`.

Example:

```
Standard
Standard
Standard
Standard
Standard
Forest
Royal
Industrial
Night
```

This would set the first 5 workspaces to the Standard theme, with the 6th being Forest theme, 7th being Royal theme, 8th being Industrial theme, and the 9th being Night theme. Those are currently all the themes available. The Forest and Royal themes only differ from the Standard theme in their window decoration (the window top) colour.

Editing `/src/themes.rs` and re-compiling is the only way to make new themes. Feel free to open a PR if an especially pleasant theme is made.


The build script (`build.rs`) takes the font char BMP (must have an alpha channel and transparent background) files in `bmps` and processes them into `.alpha` files which are read by the window manager to draw characters from a specific font. The `.alpha` files are very simple. They include the vertical offset of the character (the second character in the BMP file name), as well as several lines of CSV, where each value is either blank (0) or a value from 0-255 representing the alpha value of that pixel in the original BMP file.

The default included fonts are `nimbus-roman`, `nimbus-romono` (a version of `nimbus-roman` slightly modified to be better for monospace environments), and `shippori-mincho` for Japanese (and technically Chinese).

License information for those fonts can be font in the `README.md` at the project root.

Custom fonts can be added, though to get the window manager and window binaries to use the font, the source needs to be modified and recompiled.

Fonts must have a `?` char, which will be used when characters that are not defined in the font are encountered.

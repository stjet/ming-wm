ming-wm was originally written for Linux, but has been ported to OpenBSD, because OpenBSD is cool and awesome.

## Requirements / Assumptions

This port has been tested relying on the following configuration:

- UEFI enabled, so that the [efifb driver]() can be used
- The keyboard encoding is `xx.metaesc` (where xx is the country code), so that key combos with the Alt key can be parsed correctly

## Limitations

- No touchscreen support

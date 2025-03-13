A text editor. Specifically, a subset of a vim.

## Usage

It is probably best to read a Vim tutorial for the basics. All supportd keystrokes should *mostly* behave the same as in Vim.

### Supported in Command-line Mode

- `e[dit]`
- `t[abe]`, `[tab]n`, `[tab]p`
- `q[uit]`
- `w[rite]`

### Supported in Normal Mode

- `:`
- `i`
- `A`
- `r`
- `dd`
- `dw`
- `G`
- `gg`
- `<number>gg`
- `f<char>`
- `F<char>`
- `x`
- `h`, `j`, `k`, `l`
- `0`, `^`, `$`

### Malvim Specific

In Command-line Mode, `autoindent` can be done to toggle auto-indenting (when making new line in Insert Mode [ie, by hitting Enter/Return], space indentation of the new line will be the same as the space indentation of the current line).

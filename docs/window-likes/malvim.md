A text/code editor. Specifically, a subset of vim.

Funnily enough, that subset doesn't include the **vi**sual (ie, multi-line) capabilities of vim that the "vi" stands for. Perhaps it should be called "maled"?

## Usage

It is probably best to read a Vim tutorial for the basics. All supportd keystrokes should *mostly* behave the same as in Vim.

### Supported in Command-line Mode

- `e[dit] <file>`
- `t[abe] <file>`, `[tab]n`, `[tab]p`
- `q[uit]`
- `w[rite]`

Tab completion is supported for the `<file>` argument.

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
- `<num>h`, `<num>j`, `<num>k`, `<num>l`
- `0`, `^`, `$`

### Malvim Specific

In Command-line Mode, `autoindent` can be done to toggle auto-indenting (when making new line in Insert Mode [ie, by hitting Enter/Return], space indentation of the new line will be the same as the space indentation of the current line). **Toggling on `autoindent` is highly recommended when editing code.**

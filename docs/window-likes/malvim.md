A text/code editor. Specifically, a subset of vim.

Funnily enough, that subset doesn't include the **vi**sual (ie, multi-line) capabilities of vim that the "vi" stands for. Perhaps it should be called "maled"?

## Usage

It is probably best to read a Vim tutorial for the basics. All supportd keystrokes should *mostly* behave the same as in Vim.

### Supported in Command-line Mode

- `e[dit] <file>`
- `t[abe] <file>`, `[tab]n`, `[tab]p`
- `q[uit]`
- `w[rite]`
- `/<query>`

Tab completion is supported for the `<file>` argument. Down arrow will clear the current command, and up arrow will fill in the last ran command.

### Supported in Normal Mode

- `:`
- `i`
- `o`, `O`
- `A`
- `r`
- `dd`
- `<number>dd`
- `dw` (`dw` is not identical to vim's behaviour), `dW`
- `d$`
- `G`
- `gg`
- `<number>gg`
- `f<char>`, `F<char>`
- `<number>f<char>`, `<number>F<char>`
- `;` (same as `f<char>` but with the char the cursor is on, so not the same as vim)
- `<num>;`
- `,` (same as `F<char>` but with the char the cursor is on, so not the same as vim)
- `<num>,`
- `x`
- `h` (or left arrow), `j` (or down arrow), `k` (or up arrow), `l` (or right arrow)
- `<num>h`, `<num>j` (or down arrow), `<num>k` (or up arrow), `<num>l`
- `0`, `^`, `$`
- `%`

### Malvim Specific

In Command-line Mode, `autoindent` can be done to toggle auto-indenting (when making new line in Insert Mode [ie, by hitting Enter/Return], space indentation of the new line will be the same as the space indentation of the current line). **Toggling on `autoindent` is highly recommended when editing code.**

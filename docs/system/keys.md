Relevant section taken from `src/bin/main.rs`:

```rust
fn key_to_char(key: Key) -> Option<KeyChar> {
  match key {
    Key::Char('\n') => Some(KeyChar::Press('𐘂')),
    Key::Char(c) => Some(KeyChar::Press(c)),
    Key::Alt(c) => Some(KeyChar::Alt(c)),
    Key::Ctrl(c) => Some(KeyChar::Ctrl(c)),
    Key::Backspace => Some(KeyChar::Press('𐘁')),
    Key::Esc => Some(KeyChar::Press('𐘃')),
    Key::Up => Some(KeyChar::Press('𐙘')),
    Key::Down => Some(KeyChar::Press('𐘞')),
    Key::Left => Some(KeyChar::Press('𐙣')),
    Key::Right => Some(KeyChar::Press('𐙥')),
    _ => None,
  }
}
```

The special keys backspace, enter, escape, and the arrows, are represented by a single Linear A char. For ease, there are [methods](https://docs.rs/ming-wm-lib/latest/ming_wm_lib/messages/struct.KeyPress.html) to check whether a key press is a backspace, enter, etc, without pasting the Linear A into the code.

Although the arrow keys are supported, please try and support the Vim `hjkl` if possible!

The `Press` events are sent as `WindowMessage::KeyPress(KeyPress)`, and the `Ctrl` events are sent as `WindowMessage::CtrlKeyPress(KeyPress)`. Any keys pressed along with the Alt key are not passed to the windows.


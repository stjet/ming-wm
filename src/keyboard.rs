use termion::event::Key;

#[derive(Clone, Debug, PartialEq)]
pub enum KeyChar {
  Press(char),
  Alt(char),
}

//use Linear A for escape, backspace, enter
pub fn key_to_char(key: Key) -> Option<KeyChar> {
  match key {
    Key::Char('\n') => Some(KeyChar::Press('𐘂')),
    Key::Char(c) => Some(KeyChar::Press(c)),
    Key::Alt(c) => Some(KeyChar::Alt(c)),
    Key::Backspace => Some(KeyChar::Press('𐘁')),
    _ => None,
  }
}


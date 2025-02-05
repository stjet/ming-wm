use std::path::PathBuf;

use termion::event::Key;

use crate::window_manager::KeyChar;

//use Linear A for escape, backspace, enter
pub fn key_to_char(key: Key) -> Option<KeyChar> {
  match key {
    Key::Char('\n') => Some(KeyChar::Press('𐘂')),
    Key::Char(c) => Some(KeyChar::Press(c)),
    Key::Alt(c) => Some(KeyChar::Alt(c)),
    Key::Ctrl(c) => Some(KeyChar::Ctrl(c)),
    Key::Backspace => Some(KeyChar::Press('𐘁')),
    Key::Esc => Some(KeyChar::Press('𐘃')),
    _ => None,
  }
}

pub fn min(one: usize, two: usize) -> usize {
  if one > two { two } else { one } 
}

pub trait Substring {
  fn substring(&self, start: usize, end: usize) -> &str;
  fn remove(&self, index: usize, len: usize) -> String;
  fn remove_last(&self) -> String;
}

impl Substring for String {
  fn substring(&self, start: usize, end: usize) -> &str {
    let mut byte_start = 0;
    let mut byte_end = 0;
    let mut chars = self.chars();
    for i in 0..end {
      let char_length = chars.next().unwrap().len_utf8();
      if i < start {
        byte_start += char_length;
      }
      if i == end {
        break;
      }
      byte_end += char_length;
      
    }
    &self[byte_start..byte_end]
  }

  fn remove(&self, index: usize, len: usize) -> String {
    self.substring(0, index).to_string() + self.substring(index + len, self.chars().count())
  }

  fn remove_last(&self) -> String {
    self.substring(0, self.chars().count() - 1).to_string()
  }
}

//the tuple is first, line #, actual line
pub fn calc_actual_lines<'a>(lines: impl Iterator<Item = &'a String>, max_chars_per_line: usize, one_extra: bool) -> Vec<(bool, usize, String)> {
  let mut actual_lines = Vec::new();
  let mut line_num = 0;
  for real_line in lines {
    let mut line = real_line.to_string() + if one_extra { " " } else { "" };
    let mut first = true;
    loop {
      if line.chars().count() <= max_chars_per_line {
        actual_lines.push((first, line_num, line));
        break;
      } else {
        let mut line_chars = line.chars();
        let mut push_string = String::new();
        for _i in 0..max_chars_per_line {
          push_string += &line_chars.next().unwrap().to_string();
        }
        actual_lines.push((first, line_num, push_string));
        line = line_chars.collect();
      }
      first = false;
    }
    line_num += 1;
  }
  actual_lines
}

pub fn calc_new_cursor_pos(cursor_pos: usize, new_length: usize) -> usize {
  if cursor_pos >= new_length {
    if new_length == 0 {
      0
    } else {
      new_length - 1
    }
  } else {
    cursor_pos
  }
}

pub fn concat_paths(current_path: &str, add_path: &str) -> Result<PathBuf, ()> {
  let mut new_path = PathBuf::from(current_path);
  if add_path.starts_with("/") {
    //absolute path
    new_path = PathBuf::from(add_path);
  } else {
    //relative path
    for part in add_path.split("/") {
      if part == ".." {
        if let Some(parent) = new_path.parent() {
          new_path = parent.to_path_buf();
        } else {
          return Err(());
        }
      } else {
        new_path.push(part);
      }
    }
  }
  Ok(new_path)
}

//go from seconds to minutes:seconds
pub fn format_seconds(seconds: u64) -> String {
  let mut m = (seconds / 60).to_string(); //automatically rounds down
  if m.len() == 1 {
    m = "0".to_string() + &m;
  }
  let mut s = (seconds % 60).to_string();
  if s.len() == 1 {
    s = "0".to_string() + &s;
  }
  m + ":" + &s
}

pub const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

pub fn u8_to_hex(u: u8) -> String {
  let mut h = String::new();
  h.push(HEX_CHARS[(u / 16) as usize]);
  h.push(HEX_CHARS[(u % 16) as usize]);
  h
}

pub fn hex_to_u8(c1: char, c2: char) -> u8 {
  (HEX_CHARS.iter().position(|c| c == &c1).unwrap() * 16 + HEX_CHARS.iter().position(|c| c == &c2).unwrap()) as u8
}

pub fn is_hex(c: char) -> bool {
  HEX_CHARS.iter().position(|hc| hc == &c).is_some()
}


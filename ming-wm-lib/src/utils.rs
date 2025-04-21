use std::path::PathBuf;
use std::fs::read_dir;

use crate::framebuffer_types::{ Dimensions, Point };

pub fn min(one: usize, two: usize) -> usize {
  if one > two { two } else { one } 
}

pub trait Substring {
  fn substring(&self, start: usize, end: usize) -> &str;
  fn remove(&self, index: usize, len: usize) -> String;
  fn remove_last(&self) -> String;
  fn find_substring(&self, substr: &str) -> Option<usize>;
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
    /*
    let mut result = String::new();
    let mut chars = self.chars().skip(start);
    for _i in 0..(end - start) {
      result += &chars.next().unwrap().to_string();
    }
    result
    */
  }

  fn remove(&self, index: usize, len: usize) -> String {
    self.substring(0, index).to_string() + self.substring(index + len, self.chars().count())
  }

  fn remove_last(&self) -> String {
    self.substring(0, self.chars().count() - 1).to_string()
  }

  fn find_substring(&self, substr: &str) -> Option<usize> {
    //slightly inefficient
    let substr_len = substr.chars().count();
    let self_len = self.chars().count();
    if substr_len <= self_len {
      for start in 0..=(self_len - substr_len) {
        if self.substring(start, start + substr_len) == substr {
          return Some(start);
        }
      }
    }
    None
  }
}

//the tuple is first, line #, actual line
pub fn calc_actual_lines<'a>(lines: impl Iterator<Item = &'a String>, max_chars_per_line: usize, one_extra: bool) -> Vec<(bool, usize, String)> {
  let mut actual_lines = Vec::new();
  for (line_num, real_line) in lines.enumerate() {
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
  //if current_path is a file, automatically uses it's parent (a directory)
  if new_path.is_file() {
    new_path = new_path.parent().unwrap().to_path_buf();
  }
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
  HEX_CHARS.iter().any(|hc| hc == &c)
}

pub fn point_inside(point: Point, top_left: Point, size: Dimensions) -> bool {
  let x = point[0];
  let y = point[1];
  let x2 = top_left[0];
  let y2 = top_left[1];
  let x3 = x2 + size[0];
  let y3 = y2 + size[1];
  x >= x2 && y >= y2 && x <= x3 && y <= y3
}

pub fn get_rest_of_split(split: &mut dyn Iterator<Item = &str>, sep: Option<&str>) -> String {
  let mut rest = String::new();
  let mut n = split.next();
  loop {
    if n.is_none() {
      break;
    }
    rest += &n.unwrap();
    n = split.next();
    if n.is_some() && sep.is_some() {
      rest += sep.unwrap();
    }
  }
  rest
}

pub fn path_autocomplete(current_path: &str, partial_path: &str) -> Option<String> {
  if let Ok(new_path) = concat_paths(current_path, &partial_path) {
    let partial_name;
    let parent;
    if partial_path.ends_with("/") {
      partial_name = "".to_string();
      parent = new_path.as_path();
    } else {
      //this is just silly
      partial_name = new_path.clone().file_name().unwrap().to_os_string().to_string_lossy().to_string();
      parent = new_path.parent().unwrap();
    };
    if let Ok(entries) = read_dir(parent) {
      for entry in entries {
        let entry_path = entry.unwrap().path();
        let name = entry_path.file_name().unwrap().to_os_string().to_string_lossy().to_string();
        if name.starts_with(&partial_name) {
          let add = name[partial_name.len()..].to_string();
          let add_len = add.len();
          return Some(add + if entry_path.is_dir() && add_len > 0 {
            "/"
          } else {
            ""
          });
        }
      }
    }
    None
  } else {
    None
  }
}

pub fn get_all_files(dir: PathBuf) -> Vec<PathBuf> {
  let mut files = Vec::new();
  for entry in read_dir(dir).unwrap() {
    let path = entry.unwrap().path();
    if path.is_dir() {
      files.extend(get_all_files(path));
    } else {
      files.push(path);
    }
  }
  files
}

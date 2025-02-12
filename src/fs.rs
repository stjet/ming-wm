use std::fs::{ read_dir, File };
use std::path::PathBuf;
use std::io::Read;

fn get_font_char(dir: &str, c: char) -> Option<(char, Vec<Vec<u8>>, u8)> {
  let c = if c == '/' { '𐘋' } else if c == '\\' { '𐚆' } else if c == '.' { '𐘅' } else { c };
  if let Ok(mut file) = File::open(dir.to_string() + "/" + &c.to_string() + ".alpha") {
    let mut ch: Vec<Vec<u8>> = Vec::new();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let lines: Vec<&str> = contents.split("\n").collect();
    for l in 1..lines.len() {
      ch.push(lines[l].split(",").map(|n| n.parse().unwrap()).collect());
    }
    return Some((c, ch, lines[0].parse().unwrap()));
  }
  None
}

pub fn get_font_char_from_fonts(fonts: &[String], c: char) -> (char, Vec<Vec<u8>>, u8) {
  for font in fonts {
    if let Some(font_char) = get_font_char(&("./bmps/".to_string() + font), c) {
      return font_char;
    }
  }
  //so a ? char must be in every font
  get_font_char(&("./bmps/".to_string() + &fonts[0]), '?').unwrap()
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


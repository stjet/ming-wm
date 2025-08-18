use std::fs::{ read_dir, File };
use std::io::Read;
use std::collections::HashMap;

use ming_wm_lib::dirs;
use ming_wm_lib::utils::get_rest_of_split;

fn get_font_char(dir: &str, c: char) -> Option<(char, Vec<Vec<u8>>, u8)> {
  let c = if c == '/' { '𐘋' } else if c == '\\' { '𐚆' } else if c == '.' { '𐘅' } else { c };
  if let Ok(mut file) = File::open(dir.to_string() + "/" + &c.to_string() + ".alpha") {
    let mut ch: Vec<Vec<u8>> = Vec::new();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let lines: Vec<&str> = contents.split("\n").collect();
    for ln in 1..lines.len() {
      //.unwrap_or(0) is important because zeroes are just empty
      ch.push(lines[ln].replace(":", ",,,,").replace(";", ",,,").replace(".", ",,").split(",").map(|n| n.parse().unwrap_or(0)).collect());
    }
    return Some((c, ch, lines[0].parse().unwrap()));
  }
  None
}

pub fn get_font_char_from_fonts(fonts: &[String], c: char) -> (char, Vec<Vec<u8>>, u8) {
  for font in fonts {
    let p = dirs::exe_dir(Some(&("ming_bmps/".to_string() + &font))).to_string_lossy().to_string();
    if let Some(font_char) = get_font_char(&p, c) {
      return font_char;
    }
  }
  let p = dirs::exe_dir(Some(&("ming_bmps/".to_string() + &fonts[0]))).to_string_lossy().to_string();
  //so a ? char should be in every font. otherwise will just return blank
  get_font_char(&p, '?').unwrap_or(('?', vec![vec![0]], 0))
}

//Category, Vec<Display name, file name>
pub type ExeWindowInfos = HashMap<String, Vec<(String, String)>>;

//well, doesn't actually look to see if its executable. Just if it contains a _ and has no file extension, and is a file
pub fn get_all_executable_windows() -> ExeWindowInfos {
  let mut exes = HashMap::new();
  for entry in read_dir(dirs::exe_dir(None)).unwrap() {
    let pb = entry.unwrap().path();
    if pb.is_file() && pb.extension().is_none() {
      let parts = pb.file_stem().unwrap().to_string_lossy().to_string();
      let mut parts = parts.split('_');
      let category = parts.next().unwrap();
      let display = get_rest_of_split(&mut parts, Some(" "));
      let file_name = pb.file_name().unwrap().to_string_lossy().to_string();
      if display != String::new() && category.starts_with("ming") {
        let pair = (display, file_name);
        exes.entry(category.to_string()).and_modify(|v: &mut Vec<(String, String)>| (*v).push(pair.clone())).or_insert(vec![pair]);
      }
    }
  }
  exes
}


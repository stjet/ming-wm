use std::fs::read_dir;

use bmp_rust::bmp::BMP;

pub fn get_font_char(dir: &str, c: char) -> Option<(char, Vec<Vec<u8>>, u8)> {
  let c = if c == '/' { '𐘋' } else { c };
  let mut font: Vec<(char, Vec<Vec<u8>>, u8)> = Vec::new();
  for entry in read_dir(dir).unwrap() {
    let path = entry.unwrap().path();
    let path_chars: Vec<char> = path.file_name().unwrap().to_str().unwrap().to_string().chars().collect();
    if path_chars[0] == c {
      let mut ch: Vec<Vec<u8>> = Vec::new();
      if !path.is_dir() {
        let b = BMP::new_from_file(&path.clone().into_os_string().into_string().unwrap());
        let dib_header = b.get_dib_header().unwrap();
        let width = dib_header.width as usize;
        let height = dib_header.height as usize;
        for y in 0..height {
          let mut row = Vec::new();
          for x in 0..width {
            let pixel_color = b.get_color_of_px(x, y).unwrap();
            //if black, true
            row.push(pixel_color[3]); //push alpha channel
          }
          ch.push(row);
        }
        return Some((path_chars[0], ch, path_chars[1].to_digit(10).unwrap() as u8));
      }
    }
  }
  None
}

//the Vec<u8> should be [u8; 3] but thats a job for another day
pub fn get_bmp(path: &str) -> Vec<Vec<Vec<u8>>> {
  let mut bmp: Vec<Vec<Vec<u8>>> = Vec::new();
  let b = BMP::new_from_file(path);
  let dib_header = b.get_dib_header().unwrap();
  let width = dib_header.width as usize;
  let height = dib_header.height as usize;
  for y in 0..height {
    let mut row = Vec::new();
    for x in 0..width {
      let pixel_color = b.get_color_of_px(x, y).unwrap();
      //if black, true
      row.push(vec![pixel_color[0], pixel_color[1], pixel_color[2]]); //push alpha channel
    }
    bmp.push(row);
  }
  bmp
}


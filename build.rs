use std::fs::{ read_dir, File };
use std::io::Write;

use bmp_rust::bmp::BMP;

fn font_chars_to_alphas(dir: &str) {
  for entry in read_dir(dir).unwrap() {
    let path = entry.unwrap().path();
    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
    let file_name: Vec<&str> = file_name.split(".").collect();
    if file_name[1] == "bmp" {
      if !path.is_dir() {
        let mut ch: Vec<Vec<String>> = Vec::new();
        let b = BMP::new_from_file(&path.clone().into_os_string().into_string().unwrap());
        let dib_header = b.get_dib_header().unwrap();
        let width = dib_header.width as usize;
        let height = dib_header.height as usize;
        for y in 0..height {
          let mut row = Vec::new();
          for x in 0..width {
            let pixel_color = b.get_color_of_px(x, y).unwrap();
            //if black, true
            row.push(pixel_color[3].to_string()); //push alpha channel
          }
          ch.push(row);
        }
        let ch: Vec<String> = ch.into_iter().map(|row| {
          row.join(",")
        }).collect();
        let chars: Vec<char> = file_name[0].chars().collect();
        File::create(dir.to_string() + "/" + &chars[0].to_string() + ".alpha").unwrap().write_all(
            (chars[1].to_string() + "\n" + &ch.join("\n")).as_bytes()
        ).unwrap();
      }
    }
  }
}

fn main() {
  for entry in read_dir("./bmps").unwrap() {
    let path = entry.unwrap().path();
    if path.is_dir() {
      font_chars_to_alphas(path.to_str().unwrap());
    }
  }
}

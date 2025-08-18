use std::fs::{ read_dir, read_to_string, write, File };
use std::io::Write;
use std::env;
use std::path::Path;
use std::process::Command;

use bitcoin_hashes::Sha512;
use bmp_rust::bmp::BMP;

fn font_chars_to_alphas(dir: &str) {
  for entry in read_dir(dir).unwrap() {
    let path = entry.unwrap().path();
    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
    let file_name: Vec<&str> = file_name.split(".").collect();
    if file_name.len() < 2 {
      continue;
    }
    if file_name[1] == "bmp" && !path.is_dir() {
      let mut ch: Vec<Vec<String>> = Vec::new();
      let b = BMP::new_from_file(&path.clone().into_os_string().into_string().unwrap()).unwrap();
      let dib_header = b.get_dib_header().unwrap();
      let width = dib_header.width as usize;
      let height = dib_header.height as usize;
      for y in 0..height {
        let mut row = Vec::new();
        for x in 0..width {
          let pixel_color = b.get_color_of_px(x, y).unwrap();
          if pixel_color[3] == 0 {
            //zeroes are just empty. eg 255,0,255 becomes 255,,255
            row.push(String::new());
          } else {
            row.push(pixel_color[3].to_string()); //push alpha channel
          }
        }
        ch.push(row);
      }
      let ch: Vec<String> = ch.into_iter().map(|row| {
        row.join(",").replace(",,,,", ":").replace(",,,", ";").replace(",,", ".")
      }).collect();
      let chars: Vec<char> = file_name[0].chars().collect();
      File::create(dir.to_string() + "/" + &chars[0].to_string() + ".alpha").unwrap().write_all(
          (chars[1].to_string() + "\n" + &ch.join("\n")).as_bytes()
      ).unwrap();
    }
  }
}

fn main() {
  //hash + "salt" password and add to build
  let password = read_to_string("password.env").unwrap_or("password".to_string()).replace("\n", "") + "salt?sorrycryptographers!1!";
  let hash = Sha512::hash(password.as_bytes()).to_byte_array();
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("password.rs");
  write(&dest_path, format!("pub const PASSWORD_HASH: [u8; 64] = {:?};", hash)).unwrap();
  //process bmps
  for entry in read_dir("./bmps").unwrap() {
    let path = entry.unwrap().path();
    if path.is_dir() {
      font_chars_to_alphas(path.to_str().unwrap());
    }
  }
  //copy bmp folders to target
  let profile = env::var_os("PROFILE").unwrap().to_string_lossy().to_string();
  Command::new("rm").arg("-rf").arg(format!("./target/{}/ming_bmps", profile)).output().unwrap(); //delete at target first so cp works
  Command::new("cp").arg("-r").arg("./bmps").arg(format!("./target/{}/ming_bmps", profile)).output().unwrap();
  //also copy the docs folder
  Command::new("rm").arg("-rf").arg(format!("./target/{}/ming_docs", profile)).output().unwrap();
  Command::new("cp").arg("-r").arg("./docs").arg(format!("./target/{}/ming_docs", profile)).output().unwrap();
}

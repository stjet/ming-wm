use std::fs::{ OpenOptions, create_dir };
use std::io::Write;

use crate::dirs::data_dir;

/// Writes to `<XDG data directory>/ming-wm/logs.txt`. Use only for debugging!
pub fn log(message: &str) {
  let data = data_dir().unwrap().into_os_string().into_string().unwrap();
  let _ = create_dir(format!("{}/ming-wm", data));
  let _ = writeln!(OpenOptions::new().append(true).create(true).open(format!("{}/ming-wm/logs.txt", data)).unwrap(), "{}", message);
}


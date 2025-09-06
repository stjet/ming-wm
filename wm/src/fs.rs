use std::fs::read_dir;
use std::collections::HashMap;

use ming_wm_lib::dirs;
use ming_wm_lib::utils::get_rest_of_split;

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


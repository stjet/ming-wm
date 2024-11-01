use std::process::{ Command, Stdio };
use std::io::{ Read, Write };

use ron;

fn main() {
  println!("{}", 'だ');
  println!("a");
  let mut a = Command::new("cargo").arg("run").arg("-q").arg("--bin").arg("start_menu").stdout(Stdio::piped()).stdin(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
  a.stdin.unwrap().write_all("subtype\n".to_string().as_bytes());
  let mut output = String::new();
  a.stdout.as_mut().unwrap().read_to_string(&mut output);
  println!("{}", output);
  //println!("{}", &ron::to_string(&[122, 400]).unwrap());
}

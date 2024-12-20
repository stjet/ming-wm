use std::process::{ Command, Stdio };
use std::io::{ Read, Write };

use ron;

use ming_wm::messages::WindowMessage;

fn main() {
  println!("{}", ron::to_string(&WindowMessage::Init([100,100])).unwrap());
  //println!("{}", &ron::to_string(&[122, 400]).unwrap());
}

use std::vec::Vec;
use std::process::{ Command, Child, Stdio };
use std::io::{ BufReader, BufRead, Read, Write };
use std::cell::RefCell;

use ron;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

pub struct ProxyWindowLike {
  process: RefCell<Child>,
}

impl WindowLike for ProxyWindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    self.process.borrow_mut().stdin.as_mut().unwrap().write_all(("handle_message ".to_string() + &ron::to_string(&message).unwrap() + "\n").as_bytes());
    let output = self.read_line();
    ron::from_str(&output).unwrap()
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    self.process.borrow_mut().stdin.as_mut().unwrap().write_all(("draw ".to_string() + &ron::to_string(&theme_info).unwrap() + "\n").as_bytes());
    let output = self.read_line();
    ron::from_str(&output).unwrap()
  }

  //properties
  fn title(&self) -> String {
    self.process.borrow_mut().stdin.as_mut().unwrap().write_all("title\n".as_bytes());
    self.read_line()
  }

  fn resizable(&self) -> bool {
    self.process.borrow_mut().stdin.as_mut().unwrap().write_all("resizable\n".to_string().as_bytes());
    let output = self.read_line();
    ron::from_str(&output).unwrap()
  }

  fn subtype(&self) -> WindowLikeType {
    self.process.borrow_mut().stdin.as_mut().unwrap().write_all("subtype\n".to_string().as_bytes());
    let output = self.read_line();
    ron::from_str(&output).unwrap()
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    self.process.borrow_mut().stdin.as_mut().unwrap().write_all(("ideal_dimensions".to_string() + &ron::to_string(&dimensions).unwrap() + "\n").as_bytes());
    let output = self.read_line();
    ron::from_str(&output).unwrap()
  }
}

//kill process when this window like dropped
impl Drop for ProxyWindowLike {
  fn drop(&mut self) {
    self.process.borrow_mut().kill();
  }
}

impl ProxyWindowLike {
  pub fn new(file: &str) -> Self {
    ProxyWindowLike {
      //--quiet
      process: RefCell::new(Command::new("cargo").arg("run").arg("--quiet").arg("--release").arg("--bin").arg(file).stdout(Stdio::piped()).stdin(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap()),
    }
  }

  fn read_line(&self) -> String {
    let mut output = String::new();
    let mut buffer = self.process.borrow_mut();
    let buffer = buffer.stdout.as_mut().unwrap();
    let mut reader = BufReader::new(buffer);
    reader.read_line(&mut output).unwrap();
    output
  }
}


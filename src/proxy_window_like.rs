use std::vec::Vec;
use std::process::{ Command, Child, Stdio };
use std::io::{ BufReader, BufRead, Write };
use std::cell::RefCell;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::dirs;
use crate::serialize::{ Serializable, DrawInstructionsVec };

pub struct ProxyWindowLike {
  process: RefCell<Child>,
}

//try to handle panics of child processes so the entire wm doesn't crash
impl WindowLike for ProxyWindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    if let Some(stdin) = self.process.borrow_mut().stdin.as_mut() {
      let _ = stdin.write_all(("handle_message ".to_string() + &message.serialize() + "\n").as_bytes());
    }
    let output = self.read_line();
    WindowMessageResponse::deserialize(&output).unwrap_or(WindowMessageResponse::JustRedraw)
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    if let Some(stdin) = self.process.borrow_mut().stdin.as_mut() {
      let _ = stdin.write_all(("draw ".to_string() + &theme_info.serialize() + "\n").as_bytes());
    }
    let output = self.read_line();
    DrawInstructionsVec::deserialize(&output).unwrap_or(Vec::new())
  }

  //properties
  fn title(&self) -> String {
    if let Some(stdin) = self.process.borrow_mut().stdin.as_mut() {
      let _ = stdin.write_all("title\n".as_bytes());
    }
    self.read_line().chars().filter(|c| *c != '\n').collect()
  }

  fn resizable(&self) -> bool {
    //serialize for bool is just true -> "true", false -> "false"
    if let Some(stdin) = self.process.borrow_mut().stdin.as_mut() {
      let _ = stdin.write_all("resizable\n".to_string().as_bytes());
    }
    let output = self.read_line();
    output == "true\n"
  }

  fn subtype(&self) -> WindowLikeType {
    if let Some(stdin) = self.process.borrow_mut().stdin.as_mut() {
      let _ = stdin.write_all("subtype\n".to_string().as_bytes());
    }
    let output = self.read_line();
    WindowLikeType::deserialize(&output).unwrap_or(WindowLikeType::Window)
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    if let Some(stdin) = self.process.borrow_mut().stdin.as_mut() {
      let _ = stdin.write_all(("ideal_dimensions ".to_string() + &dimensions.serialize() + "\n").as_bytes());
    }
    let output = self.read_line();
    Dimensions::deserialize(&output).unwrap_or([420, 420])
  }
}

//kill process when this window like dropped
impl Drop for ProxyWindowLike {
  fn drop(&mut self) {
    let _ = self.process.borrow_mut().kill();
  }
}

impl ProxyWindowLike {
  pub fn new(name: &str) -> Self {
    let loc = dirs::exe_dir(Some(name)).to_string_lossy().to_string();
    ProxyWindowLike {
      process: RefCell::new(Command::new(loc).stdout(Stdio::piped()).stdin(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap()),
    }
  }

  //return empty string if error, do not propogate Err becuase that's messy
  //or maybe return "panicked"?
  fn read_line(&self) -> String {
    let mut buffer = self.process.borrow_mut();
    if let Some(buffer) = buffer.stdout.as_mut() {
      let mut output = String::new();
      let mut reader = BufReader::new(buffer);
      if let Ok(_) = reader.read_line(&mut output) {
        output
      } else {
        String::new()
      }
    } else {
      String::new()
    }
  }
}


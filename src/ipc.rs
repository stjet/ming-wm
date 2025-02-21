use std::io::{ stdin, BufRead };
use std::panic;

use crate::window_manager::WindowLike;
use crate::serialize::Serializable;
use crate::themes::ThemeInfo;
use crate::framebuffer::Dimensions;
use crate::messages::WindowMessage;
use crate::logging::log;

/*
pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse;

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;

  //properties
  fn title(&self) -> &'static str {
    ""
  }

  fn resizable(&self) -> bool {
    false
  }

  fn subtype(&self) -> WindowLikeType;

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions; //needs &self or its not object safe or some bullcrap
}
*/

const LOG: bool = false;

pub fn listen(mut window_like: impl WindowLike) {
  panic::set_hook(Box::new(|panic_info| {
    let (filename, line) = panic_info.location().map(|l| (l.file(), l.line())).unwrap_or(("<unknown>", 0));

    let cause = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
      format!("{:?}", s)
    } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
      format!("{:?}", s)
    } else {
      "panic occurred".to_string()
    };

    log(&format!("A panic occurred at {}:{}: {}", filename, line, cause));
  }));

  let stdin = stdin();
  for line in stdin.lock().lines() {
    let line = line.unwrap().clone();
    if LOG {
      log(&line);
    }
    let mut parts = line.split(" ");
    let method = parts.next().unwrap();
    let arg = &parts.collect::<Vec<&str>>().join(" ");
    let output = match method {
      "handle_message" => {
        format!("{}", &window_like.handle_message(WindowMessage::deserialize(arg).unwrap()).serialize())
      },
      "draw" => {
        format!("{}", &window_like.draw(&ThemeInfo::deserialize(arg).unwrap()).serialize())
      },
      "title" => {
        format!("{}", window_like.title())
      },
      "resizable" => {
        format!("{}", window_like.resizable())
      },
      "subtype" => {
        format!("{}", &window_like.subtype().serialize())
      },
      "ideal_dimensions" => {
        format!("{}", &window_like.ideal_dimensions(Dimensions::deserialize(arg).unwrap()).serialize())
      },
      _ => String::new(),
    };
    if output != String::new() {
      if LOG {
        log(&output);
      }
      println!("{}", output);
    }
  }
}


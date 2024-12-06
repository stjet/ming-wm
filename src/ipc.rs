use std::io::{ stdin, BufRead };

//use serde::{ Deserialize, Serialize };
use ron;

use crate::window_manager::WindowLike;
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
        format!("{}", ron::to_string(&window_like.handle_message(ron::from_str(arg).unwrap())).unwrap())
      },
      "draw" => {
        format!("{}", ron::to_string(&window_like.draw(&ron::from_str(arg).unwrap())).unwrap())
      },
      "title" => {
        format!("{}", window_like.title())
      },
      "resizable" => {
        format!("{}", window_like.resizable())
      },
      "subtype" => {
        format!("{}", ron::to_string(&window_like.subtype()).unwrap())
      },
      "ideal_dimensions" => {
        format!("{}", ron::to_string(&window_like.ideal_dimensions(ron::from_str(arg).unwrap())).unwrap())
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


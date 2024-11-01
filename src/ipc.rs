use std::io::{ stdin, BufRead };

//use serde::{ Deserialize, Serialize };
use ron;

use crate::window_manager::WindowLike;

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

pub fn listen(mut window_like: impl WindowLike) {
  let stdin = stdin();
  for line in stdin.lock().lines() {
    let line = line.unwrap().clone();
    let mut parts = line.split(" ");
    let method = parts.next().unwrap();
    let arg = &parts.collect::<Vec<&str>>().join(" ");
    if method == "handle_message" {
      println!("{}", ron::to_string(&window_like.handle_message(ron::from_str(arg).unwrap())).unwrap());
    } else if method == "draw" {
      println!("{}", ron::to_string(&window_like.draw(&ron::from_str(arg).unwrap())).unwrap());
    } else if method == "title" {
      println!("{}", window_like.title());
    } else if method == "resizable" {
      println!("{}", window_like.resizable());
    } else if method == "subtype" {
      println!("{}", ron::to_string(&window_like.subtype()).unwrap());
    } else if method == "ideal_dimensions" {
      println!("{}", ron::to_string(&window_like.ideal_dimensions(ron::from_str(arg).unwrap())).unwrap());
    }
  }
}


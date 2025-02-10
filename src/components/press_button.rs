use std::vec;
use std::vec::Vec;

use crate::components::Component;
use crate::framebuffer::{ Dimensions, Point };
use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager::{ DrawInstructions };

const MONO_WIDTH: u8 = 10;

pub struct PressButton<T> {
  pub top_left: Point,
  pub size: Dimensions,
  text: String,
  press_return: T,
}

impl<T: Clone> Component<T> for PressButton<T> {
  //
  fn handle_message(&mut self, message: WindowMessage) -> Option<T> {
    match message {
      WindowMessage::Touch(_, _) => {
        //assume that the parent window-like passed it to us intentionally and checked already
        //we can check again, but why?
        Some(self.press_return.clone())
      },
      _ => None,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let half = self.size[0] / 2 - self.text.len() * MONO_WIDTH as usize / 2;
    vec![
      DrawInstructions::Rect(self.top_left, [self.size[0], 1], theme_info.border_left_top),
      DrawInstructions::Rect(self.top_left, [1, self.size[1]], theme_info.border_left_top),
      DrawInstructions::Rect([self.top_left[0], self.top_left[1] + self.size[1]], [self.size[0], 1], theme_info.border_right_bottom),
      DrawInstructions::Rect([self.top_left[0] + self.size[0], self.top_left[1]], [1, self.size[1]], theme_info.border_right_bottom),
      //assume normal background colour
      DrawInstructions::Text([self.top_left[0] + half, self.top_left[1] + 8], vec!["times-new-romono".to_string()], self.text.clone(), theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)),
    ]
  }

  //properties
  fn focusable(&self) -> bool {
    false
  }

  fn clickable(&self) -> bool {
    false
  }
  
  fn name(&self) -> &String {
    //sorry
    &self.text
  }
}

impl<T> PressButton<T> {
  pub fn new(top_left: Point, size: Dimensions, text: String, press_return: T) -> Self {
    Self {
      top_left,
      size,
      text,
      press_return,
    }
  }
}


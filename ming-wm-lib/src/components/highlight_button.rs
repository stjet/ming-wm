use std::vec;
use std::vec::Vec;

use crate::components::Component;
use crate::framebuffer_types::{ Dimensions, Point };
use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager_types::DrawInstructions;

pub struct HighlightButton<T> {
  name_: String,
  top_left: Point,
  size: Dimensions,
  pub text: String,
  pub highlighted: bool,
  click_return: T,
  toggle_highlight_return: T, //also unhighlight return
}

impl<T: Clone> Component<T> for HighlightButton<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T> {
    match message {
      WindowMessage::Focus | WindowMessage::Unfocus => {
        self.highlighted = !self.highlighted;
        Some(self.toggle_highlight_return.clone())
      },
      WindowMessage::FocusClick => {
        //we know this click was for this button, otherwise window wouldn't have given us this message
        Some(self.click_return.clone())
      },
      _ => None,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let font_height = 15;
    if self.highlighted {
      vec![
        //highlight background
        DrawInstructions::Rect(self.top_left, self.size, theme_info.top),
        DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], vec!["nimbus-roman".to_string()], self.text.clone(), theme_info.top_text, theme_info.top, None, None),
      ]
    } else {
      vec![
        DrawInstructions::Rect(self.top_left, self.size, theme_info.background),
        DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], vec!["nimbus-roman".to_string()], self.text.clone(), theme_info.text, theme_info.background, None, None),
      ]
    }
  }

  //properties
  fn focusable(&self) -> bool {
    true
  }

  fn clickable(&self) -> bool {
    true
  }
  
  fn name(&self) -> &String {
    &self.name_
  }
}

impl<T> HighlightButton<T> {
  pub fn new(name_: String, top_left: Point, size: Dimensions, text: String, click_return: T, toggle_highlight_return: T, highlighted: bool) -> Self {
    Self {
      name_,
      top_left,
      size,
      text,
      click_return,
      toggle_highlight_return,
      highlighted,
    }
  }
}


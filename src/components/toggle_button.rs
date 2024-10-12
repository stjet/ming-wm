use std::vec;
use std::vec::Vec;

use crate::components::Component;
use crate::framebuffer::{ Dimensions, Point };
use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager::DrawInstructions;

//we need a text width and height measure function first
pub enum ToggleButtonAlignment {
  Centre,
  Left,
}

pub struct ToggleButton<T> {
  name_: String,
  top_left: Point,
  size: Dimensions,
  text: &'static str,
  draw_bg: bool,
  pub inverted: bool, //whether is it clicked or not
  alignment: ToggleButtonAlignment,
  click_return: T,
  unclick_return: T,
}

impl<T: Clone> Component<T> for ToggleButton<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T> {
    match message {
      WindowMessage::FocusClick => {
        //we know this click was for this button, otherwise window wouldn't have given us this message
        self.inverted = !self.inverted;
        if self.inverted {
          Some(self.click_return.clone())
        } else {
          Some(self.unclick_return.clone())
        }
      },
      _ => None,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    //to make sure the text gets vertically centred
    let font_height = 15;
    vec![
      //top and left border
      DrawInstructions::Rect(self.top_left, [self.size[0], 2], if self.inverted { theme_info.border_right_bottom } else { theme_info.border_left_top }),
      DrawInstructions::Rect(self.top_left, [2, self.size[1]], if self.inverted { theme_info.border_right_bottom } else { theme_info.border_left_top }),
      //right and bottom border
      DrawInstructions::Rect([self.top_left[0] + self.size[0] - 2, self.top_left[1]], [2, self.size[1]], if self.inverted { theme_info.border_left_top } else { theme_info.border_right_bottom }),
      DrawInstructions::Rect([self.top_left[0], self.top_left[1] + self.size[1] - 2], [self.size[0], 2], if self.inverted { theme_info.border_left_top } else { theme_info.border_right_bottom }),
      //the background if self.draw_bg
      //DrawInstructions::Rect(),
      //the text (for now, hardcoded top left)
      DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], "times-new-roman", self.text.to_string(), theme_info.text, theme_info.background, None),
    ]
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

impl<T> ToggleButton<T> {
  pub fn new(name_: String, top_left: Point, size: Dimensions, text: &'static str, click_return: T, unclick_return: T, draw_bg: bool, alignment: Option<ToggleButtonAlignment>) -> Self {
    Self {
      name_,
      top_left,
      size,
      text,
      click_return,
      unclick_return,
      draw_bg,
      inverted: false,
      alignment: alignment.unwrap_or(ToggleButtonAlignment::Centre),
    }
  }
}


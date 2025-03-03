use std::vec;
use std::vec::Vec;

use crate::components::Component;
use ming_wm_lib::framebuffer_types::{ Dimensions, Point };
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::messages::WindowMessage;
use ming_wm_lib::window_manager_types::DrawInstructions;

pub struct ToggleButton<T> {
  name_: String,
  top_left: Point,
  size: Dimensions,
  text: String,
  pub inverted: bool, //whether is it clicked or not
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
      DrawInstructions::Text([self.top_left[0] + 4, self.top_left[1] + (self.size[1] - font_height) / 2], vec!["nimbus-roman".to_string()], self.text.to_string(), theme_info.text, theme_info.background, None, None),
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
  pub fn new(name_: String, top_left: Point, size: Dimensions, text: String, click_return: T, unclick_return: T) -> Self {
    Self {
      name_,
      top_left,
      size,
      text,
      click_return,
      unclick_return,
      inverted: false,
    }
  }
}


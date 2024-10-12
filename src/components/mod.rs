use std::vec::Vec;

use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager::DrawInstructions;

pub mod toggle_button;
pub mod highlight_button;

pub trait Component<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T>;
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;

  //properties
  //focusing is a way for the *window* to know what component to send input, presses, etc
  //focusing for components is purely to give a visual representation
  fn focusable(&self) -> bool;
  fn clickable(&self) -> bool;
  fn name(&self) -> &String; //should be unique
}


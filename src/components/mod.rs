use std::vec::Vec;

use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::messages::WindowMessage;
use ming_wm_lib::window_manager_types::DrawInstructions;

pub mod toggle_button;
pub mod highlight_button;
pub mod paragraph;
pub mod press_button;

pub trait Component<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T>;
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;

  //properties
  //focusing is a way for the *window* to know what component to send input, presses, etc
  //focusing for components is purely to give a visual representation
  fn focusable(&self) -> bool;
  fn clickable(&self) -> bool;
  //fn pressable(&self) -> bool; //touch
  fn name(&self) -> &String; //should be unique
}


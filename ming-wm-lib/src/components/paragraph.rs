use std::vec;
use std::vec::Vec;

use crate::framebuffer_types::{ Dimensions, Point };
use crate::themes::ThemeInfo;
use crate::messages::WindowMessage;
use crate::window_manager_types::DrawInstructions;
use crate::utils::calc_actual_lines;
use crate::components::Component;

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 18;

pub struct Paragraph<T> {
  name_: String,
  actual_lines: Vec<(bool, usize, String)>, //first, line #, actual line
  top_left: Point,
  size: Dimensions,
  line_pos: usize,
  key_return: T,
}

impl<T: Copy> Component<T> for Paragraph<T> {
  fn handle_message(&mut self, message: WindowMessage) -> Option<T> {
    if let WindowMessage::KeyPress(key_press) = message {
      if key_press.key == 'j' {
        //down
        if self.line_pos != self.actual_lines.len() - 1 {
          self.line_pos += 1;
        }
        Some(self.key_return)
      } else if key_press.key == 'k' {
        //up
        if self.line_pos != 0 {
          self.line_pos -= 1;
        }
        Some(self.key_return)
      } else {
        None
      }
    } else {
      None
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = Vec::new();
    let max_lines = self.size[1] / LINE_HEIGHT;
    let mut start_y = self.top_left[1];
    for line in self.actual_lines.iter().skip(self.line_pos).take(max_lines) {
      instructions.push(DrawInstructions::Text([self.top_left[0], start_y], vec!["nimbus-romono".to_string()], line.2.clone(), theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)));
      start_y += LINE_HEIGHT;
    }
    instructions
  }

  //properties
  fn focusable(&self) -> bool {
    true
  }

  fn clickable(&self) -> bool {
    false
  }
  
  fn name(&self) -> &String {
    &self.name_
  }
}

impl<T> Paragraph<T> {
  pub fn new(name_: String, top_left: Point, size: Dimensions, text: String, key_return: T) -> Self {
    let mut s = Self {
      name_,
      actual_lines: Vec::new(),
      top_left,
      size,
      line_pos: 0,
      key_return,
    };
    s.new_text(text);
    s
  }

  pub fn new_text(&mut self, text: String) {
    let max_chars_per_line = self.size[0] / MONO_WIDTH as usize;
    let lines: Vec<String> = text.split("\n").map(|s| s.to_string()).collect();
    self.actual_lines = calc_actual_lines(lines.iter(), max_chars_per_line, true);
  }
}


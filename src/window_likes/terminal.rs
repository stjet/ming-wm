use std::vec::Vec;
use std::vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, WINDOW_TOP_HEIGHT };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

const MONO_WIDTH: u8 = 8;
const LINE_HEIGHT: usize = 15;
const PADDING: usize = 4;

#[derive(Default)]
pub struct Terminal {
  dimensions: Dimensions,
  lines: Vec<String>,
  actual_lines: Vec<String>, //wrapping
  actual_line_num: usize, //what line # is at the top, for scrolling
  current_input: String,
}

//for some reason key presses, then moving the window leaves the old window still there, behind it. weird

impl WindowLike for Terminal {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.lines = vec!["Mingde Terminal".to_string(), "".to_string()];
        self.calc_actual_lines();
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        self.current_input += &key_press.key.to_string();
        self.calc_actual_lines();
        WindowMessageResponse::JustRerender
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      //
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      DrawInstructions::Rect([0, 0], self.dimensions, theme_info.alt_background),
      //
    ];
    //add the visible lines of text
    let end_line = self.actual_line_num + (self.dimensions[1] - WINDOW_TOP_HEIGHT- PADDING * 2) / LINE_HEIGHT;
    let mut text_y = WINDOW_TOP_HEIGHT + PADDING;
    for line_num in self.actual_line_num..end_line {
      if line_num == self.actual_lines.len() {
        break;
      }
      let line = self.actual_lines[line_num].clone();
      instructions.push(DrawInstructions::Text([PADDING, text_y], "times-new-roman", line, theme_info.alt_text, theme_info.alt_background, Some(MONO_WIDTH)));
      text_y += LINE_HEIGHT;
    }
    instructions
  }

  fn title(&self) -> &'static str {
    "Terminal"
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [410, 410 + WINDOW_TOP_HEIGHT]
  }

  fn resizable(&self) -> bool {
    true
  }
}

impl Terminal {
  pub fn new() -> Self {
    Default::default()
  }

  fn calc_actual_lines(&mut self) {
    self.actual_lines = Vec::new();
    let max_chars_per_line = (self.dimensions[0] - PADDING * 2) / MONO_WIDTH as usize;
    for line_num in 0..=self.lines.len() {
      let mut working_line = if line_num == self.lines.len() {
        "$ ".to_string() + &self.current_input + "█"
      } else {
        self.lines[line_num].clone()
      };
      //cannot index or do .len() because those count bytes not characters
      loop {
        if working_line.chars().count() <= max_chars_per_line {
          
          self.actual_lines.push(working_line);
          break;
        } else {
          let mut working_line_chars = working_line.chars();
          let mut push_string = String::new();
          for i in 0..max_chars_per_line {
            push_string += &working_line_chars.next().unwrap().to_string();
          }
          self.actual_lines.push(push_string);
          working_line = working_line_chars.collect();
        }
      }
    }
  }
}


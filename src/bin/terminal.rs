use std::vec::Vec;
use std::vec;
use std::process::{ Command, Output };
use std::str::from_utf8;
use std::io;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::Dimensions;
use ming_wm::themes::ThemeInfo;
use ming_wm::utils::concat_paths;
use ming_wm::ipc::listen;

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 15;
const PADDING: usize = 4;

enum CommandResponse {
  ActualCommand(io::Result<Output>),
  Custom,
}

#[derive(Default)]
pub struct Terminal {
  dimensions: Dimensions,
  lines: Vec<String>,
  actual_lines: Vec<String>, //wrapping
  actual_line_num: usize, //what line # is at the top, for scrolling
  current_input: String,
  current_path: String,
}

//for some reason key presses, then moving the window leaves the old window still there, behind it. weird

impl WindowLike for Terminal {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.current_path = "/".to_string();
        self.lines = vec!["Mingde Terminal".to_string(), "".to_string()];
        self.calc_actual_lines();
        WindowMessageResponse::JustRerender
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == '𐘁' { //backspace
          if self.current_input.len() > 0 {
            self.current_input = self.current_input[..self.current_input.len() - 1].to_string();
          } else {
            return WindowMessageResponse::DoNothing;
          }
        } else if key_press.key == '𐘂' { //the enter key
          self.lines.push("$ ".to_string() + &self.current_input);
          if let CommandResponse::ActualCommand(maybe_output) = self.process_command() {
            if let Ok(output) = maybe_output {
              let write_output = if output.status.success() {
                output.stdout
              } else {
                output.stderr
              };
              for line in from_utf8(&write_output).unwrap_or("Failed to parse process output as utf-8").split("\n") {
                self.lines.push(line.to_string());
              }
            } else {
              self.lines.push("Failed to execute process".to_string());
            }
          }
          self.current_input = String::new();
        } else {
          self.current_input += &key_press.key.to_string();
        }
        self.calc_actual_lines();
        self.actual_line_num = self.actual_lines.len().checked_sub(self.get_max_lines()).unwrap_or(0);
        WindowMessageResponse::JustRerender
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      DrawInstructions::Rect([0, 0], self.dimensions, theme_info.alt_background),
    ];
    //add the visible lines of text
    let end_line = self.actual_line_num + self.get_max_lines();
    let mut text_y = PADDING;
    for line_num in self.actual_line_num..end_line {
      if line_num == self.actual_lines.len() {
        break;
      }
      let line = self.actual_lines[line_num].clone();
      instructions.push(DrawInstructions::Text([PADDING, text_y], "times-new-romono".to_string(), line, theme_info.alt_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
      text_y += LINE_HEIGHT;
    }
    instructions
  }

  fn title(&self) -> String {
    "Terminal".to_string()
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [410, 410]
  }

  fn resizable(&self) -> bool {
    true
  }
}

impl Terminal {
  pub fn new() -> Self {
    Default::default()
  }

  fn get_max_lines(&self) -> usize {
    (self.dimensions[1] - PADDING * 2) / LINE_HEIGHT
  }

  fn process_command(&mut self) -> CommandResponse {
    if self.current_input.starts_with("clear ") || self.current_input == "clear" {
      self.lines = Vec::new();
      CommandResponse::Custom
    } else if self.current_input.starts_with("cd ") {
      let mut cd_split = self.current_input.split(" ");
      cd_split.next().unwrap();
      let arg = cd_split.next().unwrap();
      if let Ok(new_path) = concat_paths(&self.current_path, arg) {
        if new_path.exists() {
          self.current_path = new_path.to_str().unwrap().to_string();
        }
      }
      CommandResponse::Custom
    } else {
      CommandResponse::ActualCommand(Command::new("sh").arg("-c").arg(&self.current_input).current_dir(&self.current_path).output())
    }
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
          for _i in 0..max_chars_per_line {
            push_string += &working_line_chars.next().unwrap().to_string();
          }
          self.actual_lines.push(push_string);
          working_line = working_line_chars.collect();
        }
      }
    }
  }
}

pub fn main() {
  listen(Terminal::new());
}


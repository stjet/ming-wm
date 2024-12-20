use std::vec::Vec;
use std::vec;
use std::process::{ Command, Child, Stdio };
use std::io::Read;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::Dimensions;
use ming_wm::themes::ThemeInfo;
use ming_wm::utils::concat_paths;
use ming_wm::ipc::listen;

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 15;
const PADDING: usize = 4;

#[derive(Default, PartialEq)]
enum State {
  #[default]
  Input, //typing in to run command
  Running, //running command
}

#[derive(Default)]
pub struct Terminal {
  dimensions: Dimensions,
  state: State,
  lines: Vec<String>,
  actual_lines: Vec<String>, //wrapping
  actual_line_num: usize, //what line # is at the top, for scrolling
  current_input: String,
  current_path: String,
  running_process: Option<Child>,
  last_command: Option<String>,
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
        if self.state == State::Input {
          if key_press.key == '𐘁' { //backspace
            if self.current_input.len() > 0 {
              self.current_input = self.current_input[..self.current_input.len() - 1].to_string();
            } else {
              return WindowMessageResponse::DoNothing;
            }
          } else if key_press.key == '𐘂' { //the enter key
            self.lines.push("$ ".to_string() + &self.current_input);
            self.last_command = Some(self.current_input.clone());
            self.state = self.process_command();
            self.current_input = String::new();
          } else {
            self.current_input += &key_press.key.to_string();
          }
          self.calc_actual_lines();
          self.actual_line_num = self.actual_lines.len().checked_sub(self.get_max_lines()).unwrap_or(0);
          WindowMessageResponse::JustRerender
        } else {
          //update
          let running_process = self.running_process.as_mut().unwrap();
          if let Some(status) = running_process.try_wait().unwrap() {
            //process exited
            let mut output = String::new();
            if status.success() {
              let _ = running_process.stdout.as_mut().unwrap().read_to_string(&mut output);
            } else {
              let _ = running_process.stderr.as_mut().unwrap().read_to_string(&mut output);
            }
            for line in output.split("\n") {
              self.lines.push(line.to_string());
            }
            self.state = State::Input;
            self.calc_actual_lines();
            WindowMessageResponse::JustRerender
          } else {
            //still running
            WindowMessageResponse::DoNothing
          }
        }
      },
      WindowMessage::CtrlKeyPress(key_press) => {
        if self.state == State::Running && key_press.key == 'c' {
          //kills and running_process is now None
          let _ = self.running_process.take().unwrap().kill();
          self.state = State::Input;
          WindowMessageResponse::JustRerender
        } else if self.state == State::Input && (key_press.key == 'p' || key_press.key == 'n') {
          //only the last command is saved unlike other terminals. good enough for me
          if key_press.key == 'p' && self.last_command.is_some() {
            self.current_input = self.last_command.clone().unwrap();
            WindowMessageResponse::JustRerender
          } else if key_press.key == 'n' {
            self.current_input = String::new();
            WindowMessageResponse::JustRerender
          } else {
            WindowMessageResponse::DoNothing
          }
        } else {
          WindowMessageResponse::DoNothing
        }
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
      instructions.push(DrawInstructions::Text([PADDING, text_y], vec!["times-new-romono".to_string()], line, theme_info.alt_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
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

  fn process_command(&mut self) -> State {
    if self.current_input.starts_with("clear ") || self.current_input == "clear" {
      self.lines = Vec::new();
      State::Input
    } else if self.current_input.starts_with("cd ") {
      let mut cd_split = self.current_input.split(" ");
      cd_split.next().unwrap();
      let arg = cd_split.next().unwrap();
      if let Ok(new_path) = concat_paths(&self.current_path, arg) {
        if new_path.exists() {
          self.current_path = new_path.to_str().unwrap().to_string();
        }
      }
      State::Input
    } else {
      self.running_process = Some(Command::new("sh").arg("-c").arg(&self.current_input).current_dir(&self.current_path).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap());
      State::Running
    }
  }

  fn calc_actual_lines(&mut self) {
    self.actual_lines = Vec::new();
    let max_chars_per_line = (self.dimensions[0] - PADDING * 2) / MONO_WIDTH as usize;
    let end = if self.state == State::Input {
      self.lines.len()
    } else {
      self.lines.len() - 1
    };
    for line_num in 0..=end {
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


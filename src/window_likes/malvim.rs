use std::vec::Vec;
use std::vec;
use std::fmt;
use std::path::PathBuf;
use std::fs::{ read_to_string, write };

use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::themes::ThemeInfo;
use crate::framebuffer::Dimensions;
use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::utils::calc_actual_lines;

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 18;
const PADDING: usize = 2;
const BAND_HEIGHT: usize = 18;

struct FileInfo {
  pub name: String,
  pub path: String,
  pub changed: bool,
  pub top_line_pos: usize,
  pub line_pos: usize,
  pub cursor_pos: usize,
  pub content: Vec<String>,
  //
}

#[derive(Default, PartialEq)]
enum State {
  #[default]
  None,
  //
}

#[derive(Default, PartialEq)]
enum Mode {
  #[default]
  Normal,
  Insert,
  Command,
}

impl fmt::Display for Mode {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    let write_str = match self {
      Mode::Normal => "NORMAL",
      Mode::Insert => "INSERT",
      Mode::Command => "COMMAND",
    };
    fmt.write_str(write_str)?;
    Ok(())
  }
}

#[derive(Default)]
struct Current {
  pub actual_lines: Vec<(bool, usize, String)>, //first, line #, actual line
  pub line_num_width: usize, //file line digits * MONO_WIDTH
  pub max_lines: usize, //max actual lines on screen
  pub max_chars_per_line: usize,
}

#[derive(Default)]
pub struct Malvim {
  dimensions: Dimensions,
  state: State,
  mode: Mode,
  command: Option<String>,
  bottom_message: Option<String>,
  maybe_num: Option<usize>,
  files: Vec<FileInfo>,
  current_file_index: usize,
  current: Current,
}

impl WindowLike for Malvim {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == '𐘃' { //escape key
          self.mode = Mode::Normal;
          self.state = State::None;
        } else if key_press.key == ':' && self.mode == Mode::Normal && self.state == State::None {
          self.mode = Mode::Command;
          self.command = Some(String::new());
        } else if key_press.key == 'i' && self.mode == Mode::Normal && self.state == State::None && self.files.len() > 0 {
          self.mode = Mode::Insert;
        } else if self.mode == Mode::Insert {
          let current_file = &self.files[self.current_file_index];
          if key_press.key == '𐘂' { //the enter key
            //
          } else if key_press.key == '𐘁' { //backspace
            //
          } else {
            //
          }
        } else if self.mode == Mode::Normal && self.files.len() > 0 {
          let current_file = &mut self.files[self.current_file_index];
          if key_press.key == 'h' {
            current_file.cursor_pos = current_file.cursor_pos.checked_sub(1).unwrap_or(0);
          } else if key_press.key == 'j' {
            //
          } else if key_press.key == 'k' {
            //
          } else if key_press.key == 'l' {
            current_file.cursor_pos += 1;
            if current_file.cursor_pos == current_file.content[current_file.line_pos].len() {
              current_file.cursor_pos = current_file.content[current_file.line_pos].len() - 1;
            }
          } else if key_press.key == '0' {
            current_file.cursor_pos = 0;
          } else if key_press.key == '$' {
            current_file.cursor_pos = current_file.content[current_file.line_pos].len().checked_sub(1).unwrap_or(0);
          }
          //
        } else if self.mode == Mode::Command {
          self.bottom_message = None;
          let command = self.command.clone().unwrap_or("".to_string());
          if key_press.key == '𐘂' { //the enter key
            self.process_command();
            self.command = None;
            self.mode = Mode::Normal;
          } else if key_press.key == '𐘁' { //backspace
            if command.len() > 0 {
              self.command = Some(command[..command.len() - 1].to_string());
            }
          } else {
            self.command = Some(command.to_string() + &key_press.key.to_string());
          }
        } else {
          return WindowMessageResponse::DoNothing;
        }
        self.calc_current(); //too over zealous but whatever
        WindowMessageResponse::JustRerender
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      //the top white bar
      DrawInstructions::Rect([0, 0], [self.dimensions[0], BAND_HEIGHT], theme_info.alt_text),
      //black background
      DrawInstructions::Rect([0, BAND_HEIGHT], [self.dimensions[0], self.dimensions[1] - BAND_HEIGHT * 3], theme_info.alt_background),
      //slight above bottom blue bar
      DrawInstructions::Rect([0, self.dimensions[1] - BAND_HEIGHT * 2], [self.dimensions[0], BAND_HEIGHT], theme_info.top),
      //black background
      DrawInstructions::Rect([0, self.dimensions[1] - BAND_HEIGHT], [self.dimensions[0], BAND_HEIGHT], theme_info.alt_background),
    ];
    //write file tabs
    let mut used_width = 0;
    for file_index in 0..self.files.len() {
      let file_info = &self.files[file_index];
      let future_used_width = used_width + 4 + file_info.name.len() * MONO_WIDTH as usize;
      //just cut off when too many file tabs open to fit
      if future_used_width > self.dimensions[0] {
        break;
      }
      let background = if file_index == self.current_file_index {
        theme_info.alt_background
      } else {
        theme_info.alt_secondary
      };
      instructions.extend(vec![
        DrawInstructions::Rect([used_width, 2], [file_info.name.len() * MONO_WIDTH as usize + 4, BAND_HEIGHT - 2], background),
        DrawInstructions::Text([used_width + 2, 2], "times-new-romono", file_info.name.clone(), theme_info.alt_text, background, Some(0), Some(MONO_WIDTH)),
      ]);
      used_width = future_used_width;
    }
    //write the actual current file
    let mut sub_line_num = 0; //a line in a file can be split into multiple lines for display
    if self.files.len() > 0 {
      let current_file = &self.files[self.current_file_index];
      let current = &self.current;
      for line_num in current_file.top_line_pos..(current_file.top_line_pos + current.max_lines) {
        if line_num == current.actual_lines.len() {
          break;
        }
        let line = &current.actual_lines[line_num];
        let rel_line_num = line_num - current_file.top_line_pos;
        //write line num text (if start of line)
        let y0 = BAND_HEIGHT + rel_line_num * LINE_HEIGHT + PADDING;
        if line.0 {
          instructions.push(DrawInstructions::Text([PADDING, y0], "times-new-romono", line.1.to_string(), theme_info.alt_secondary, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
          sub_line_num = 0;
        }
        let x1 = current.line_num_width + PADDING * 2;
        //write actual line
        //line.2
        instructions.push(DrawInstructions::Text([x1, y0], "times-new-romono", line.2.clone(), theme_info.alt_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
        sub_line_num += 1;
        let max = sub_line_num * current.max_chars_per_line;
        let min = max - current.max_chars_per_line;
        if line.1 == current_file.line_pos && current_file.cursor_pos >= min && current_file.cursor_pos < max {
          let top_left = [x1 + (current_file.cursor_pos - min) * MONO_WIDTH as usize, y0];
          //the cursor is on this line, draw it
          instructions.push(DrawInstructions::Rect(top_left, [MONO_WIDTH as usize, LINE_HEIGHT], theme_info.top));
          //draw the char over it
          instructions.push(DrawInstructions::Text(top_left, "times-new-romono", line.2.chars().nth(current_file.cursor_pos - min).unwrap().to_string(), theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
        }
      }
    }
    //bottom blue band stuff
    //write mode
    instructions.push(DrawInstructions::Text([0, self.dimensions[1] - BAND_HEIGHT * 2 + 1], "times-new-romono", self.mode.to_string(), theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
    let file_status;
    if self.files.len() > 0 {
      file_status = self.files[self.current_file_index].name.clone();
    } else {
      file_status = "No file open".to_string();
    }
    instructions.push(DrawInstructions::Text([self.dimensions[0] - file_status.len() * (MONO_WIDTH as usize), self.dimensions[1] - BAND_HEIGHT * 2 + 1], "times-new-romono", file_status, theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
    //write command or bottom message
    if self.mode == Mode::Command {
      instructions.push(DrawInstructions::Text([0, self.dimensions[1] - BAND_HEIGHT], "times-new-romono", ":".to_string() + &self.command.clone().unwrap_or("".to_string()), theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
    } else if self.mode == Mode::Normal && self.bottom_message.is_some() {
      instructions.push(DrawInstructions::Text([0, self.dimensions[1] - BAND_HEIGHT], "times-new-romono", self.bottom_message.clone().unwrap(), theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
    }
    instructions
  }

  fn title(&self) -> &'static str {
    "Malvim"
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

impl Malvim {
  pub fn new() -> Self {
    Default::default()
  }
  
  fn calc_current(&mut self) {
    if self.files.len() == 0 {
      return;
    }
    let current_file = &self.files[self.current_file_index];
    let line_num_width = current_file.content.len().to_string().len() * MONO_WIDTH as usize;
    let max_chars_per_line = (self.dimensions[0] - line_num_width - PADDING * 2) / MONO_WIDTH as usize;
    let actual_lines = calc_actual_lines(current_file.content.iter(), max_chars_per_line);
    //now, see if the line_pos is still visible from the top_line_pos,
    //if not, move top_line_pos down until it is
    let max_lines = (self.dimensions[1] - BAND_HEIGHT * 3 - PADDING) / LINE_HEIGHT;
    if current_file.top_line_pos + max_lines < current_file.line_pos {
      self.files[self.current_file_index].top_line_pos = current_file.line_pos.checked_sub(max_lines).unwrap_or(0);
    }
    self.current = Current {
      actual_lines,
      line_num_width,
      max_lines,
      max_chars_per_line,
    };
  }

  fn process_command(&mut self) {
    let mut parts = self.command.as_ref().unwrap().split(" ");
    let first = parts.next().unwrap();
    let arg = parts.next().unwrap_or("");
    if first == "e" || first == "edit" || ((first == "t" || first == "tabe") && self.files.len() > 0) {
      //find the file and open it
      let mut failed = false;
      let mut new_path = if self.files.len() > 0 {
        PathBuf::from(self.files[self.current_file_index].path.clone()).parent().unwrap().to_path_buf()
      } else {
        PathBuf::from("/")
      };
      for part in arg.split("/") {
        if part == ".." {
          if let Some(parent) = new_path.parent() {
            new_path = parent.to_path_buf();
          } else {
            failed = true;
          }
        } else {
          new_path.push(part);
        }
      }
      if !failed && new_path.is_file() {
        let name = new_path.file_name().unwrap().to_string_lossy().into_owned();
        let path = new_path.to_string_lossy().into_owned();
        if let Ok(content) = read_to_string(new_path) {
          let file_info = FileInfo {
            name,
            path,
            changed: false,
            top_line_pos: 0,
            line_pos: 0,
            cursor_pos: 0,
            content: content.split("\n").map(|s| s.to_string()).collect(),
          };
          if first == "e" || first == "edit" {
            if self.files.len() > 0 {
              self.files[self.current_file_index] = file_info;
            } else {
              self.files.push(file_info);
            }
          } else {
            //t(abe)
            self.current_file_index += 1;
            if self.current_file_index == self.files.len() - 1 {
              self.files.push(file_info);
            } else {
              self.files.insert(self.current_file_index, file_info);
            }
          }
        } else {
          self.bottom_message = Some("Failed to open that file".to_string());
        }
      } else {
        self.bottom_message = Some("That is not a file or does not exist".to_string());
      }
    } else if self.files.len() == 0 {
      self.bottom_message = Some("No files are open, so can only do :e(dit)".to_string());
    } else if first == "w" || first == "write" {
      let current_file = &self.files[self.current_file_index];
      write(&current_file.path, &current_file.content.join("\n"));
      self.files[self.current_file_index].changed = false;
    } else if first == "q" || first == "quit" {
      self.files.remove(self.current_file_index);
      self.current_file_index = self.current_file_index.checked_sub(1).unwrap_or(0);
    } else if first == "p" || first == "tabp" {
      self.current_file_index = self.current_file_index.checked_sub(1).unwrap_or(self.files.len() - 1);
    } else if first == "n" || first == "tabn" {
      self.current_file_index += 1;
      if self.current_file_index == self.files.len() {
        self.current_file_index = 0;
      }
    } else {
      self.bottom_message = Some("Not a command".to_string());
    }
  }
}


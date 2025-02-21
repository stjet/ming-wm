use std::vec::Vec;
use std::vec;
use std::fmt;
use std::path::PathBuf;
use std::fs::{ read_to_string, write };

use ming_wm::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest, ShortcutType };
use ming_wm::themes::ThemeInfo;
use ming_wm::framebuffer::Dimensions;
use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::utils::{ calc_actual_lines, calc_new_cursor_pos, Substring };
use ming_wm::ipc::listen;

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 18;
const PADDING: usize = 2;
const BAND_HEIGHT: usize = 19;

struct FileInfo {
  pub name: String,
  pub path: String,
  pub changed: bool,
  pub top_line_pos: usize,
  pub line_pos: usize,
  //max is length (yeah, not length + 1)
  pub cursor_pos: usize,
  pub content: Vec<String>,
  //
}

#[derive(Default, PartialEq)]
enum State {
  #[default]
  None,
  Replace,
  MaybeDelete,
  Maybeg,
  Find,
  BackFind,
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
struct Malvim {
  dimensions: Dimensions,
  state: State,
  mode: Mode,
  command: Option<String>,
  bottom_message: Option<String>,
  maybe_num: Option<usize>,
  files: Vec<FileInfo>,
  current_file_index: usize,
  current: Current,
  autoindent: bool,
}

impl WindowLike for Malvim {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        let mut changed = true;
        let mut new = false;
        if key_press.key == '𐘃' { //escape key
          self.mode = Mode::Normal;
          self.state = State::None;
          changed = false;
        } else if key_press.key == ':' && self.mode == Mode::Normal && self.state == State::None {
          self.mode = Mode::Command;
          self.command = Some(String::new());
          changed = false;
        } else if (key_press.key == 'i' || key_press.key == 'A') && self.mode == Mode::Normal && self.state == State::None && self.files.len() > 0 {
          if key_press.key == 'A' {
            let current_file = &mut self.files[self.current_file_index];
            current_file.cursor_pos = current_file.content[current_file.line_pos].len();
          }
          self.mode = Mode::Insert;
          changed = false;
        } else if self.mode == Mode::Insert {
          let current_file = &mut self.files[self.current_file_index];
          let current_length = current_file.content[current_file.line_pos].len();
          let line = &current_file.content[current_file.line_pos];
          if key_press.key == '𐘂' { //the enter key
            let mut line = line.clone();
            let (left, right) = line.split_at_mut(current_file.cursor_pos);
            current_file.content[current_file.line_pos] = left.to_string();
            let spaces = if self.autoindent {
              //find out how many spaces the line starts with, copy that to the new line
              let mut spaces = 0;
              for c in left.chars() {
                if c == ' ' {
                  spaces += 1;
                } else {
                  break;
                }
              }
              spaces
            } else {
              0
            };
            current_file.content.insert(current_file.line_pos + 1, " ".repeat(spaces) + right);
            current_file.line_pos += 1;
            current_file.cursor_pos = spaces;
          } else if key_press.key == '𐘁' { //backspace
            if current_length > 0 && current_file.cursor_pos > 0 {
              current_file.content[current_file.line_pos] = line.remove(current_file.cursor_pos - 1, 1);
              current_file.cursor_pos -= 1;
            } else {
              if current_file.line_pos > 0 {
                //merge the prev line and this line
                let old_previous_line_length = current_file.content[current_file.line_pos - 1].len();
                let removed = current_file.content.remove(current_file.line_pos);
                current_file.content[current_file.line_pos - 1] += &removed; 
                current_file.line_pos -= 1;
                current_file.cursor_pos = old_previous_line_length;
              }
            }
          } else {
            current_file.content[current_file.line_pos] = line.substring(0, current_file.cursor_pos).to_string() + &key_press.key.to_string() + line.substring(current_file.cursor_pos, line.len());
            current_file.cursor_pos += 1;
          }
        } else if self.mode == Mode::Normal && self.files.len() > 0 {
          let current_file = &mut self.files[self.current_file_index];
          let current_length = current_file.content[current_file.line_pos].len();
          let mut numbered = false;
          //
          if self.state == State::Replace {
            if current_length > 0 && current_file.cursor_pos < current_length {
              let line = &current_file.content[current_file.line_pos];
              current_file.content[current_file.line_pos] = line.substring(0, current_file.cursor_pos).to_string() + &key_press.key.to_string() + line.substring(current_file.cursor_pos + 1, line.len());
            }
            self.state = State::None;
          } else if self.state == State::MaybeDelete {
            if key_press.key == 'd' {
              current_file.content.remove(current_file.line_pos);
              if current_file.content.len() == 0 {
                current_file.content = vec![String::new()];
              } else if current_file.line_pos == current_file.content.len() {
                current_file.line_pos = current_file.content.len() - 1;
              }
              let new_length = current_file.content[current_file.line_pos].len();
              current_file.cursor_pos = calc_new_cursor_pos(current_file.cursor_pos, new_length);
            } else if key_press.key == 'w' {
              //todo: currently doesn't work on a single space?
              let line = &current_file.content[current_file.line_pos];
              if line.len() > 0 {
                //offset until space or eol
                let offset = line.chars().skip(current_file.cursor_pos).position(|c| c == ' ').unwrap_or(line.chars().count() - current_file.cursor_pos);
                current_file.content[current_file.line_pos] = line.remove(current_file.cursor_pos, offset);
                let new_length = current_file.content[current_file.line_pos].len();
                current_file.cursor_pos = calc_new_cursor_pos(current_file.cursor_pos, new_length);
              }
            }
            self.state = State::None;
          } else if self.state == State::Maybeg {
            if key_press.key == 'g' {
              current_file.line_pos = self.maybe_num.unwrap_or(1) - 1;
              if current_file.line_pos >= current_file.content.len() {
                current_file.line_pos = current_file.content.len() - 1;
              }
              let new_length = current_file.content[current_file.line_pos].len();
              current_file.cursor_pos = calc_new_cursor_pos(current_file.cursor_pos, new_length);
            }
            changed = false;
            self.state = State::None;
          } else if self.state == State::Find || self.state == State::BackFind {
            let old_pos = current_file.cursor_pos;
            let find_pos = if self.state == State::Find {
              if old_pos < current_file.content[current_file.line_pos].len() {
                old_pos + current_file.content[current_file.line_pos].chars().skip(old_pos + 1).position(|c| c == key_press.key).unwrap_or(0) + 1
              } else {
                old_pos
              }
            } else {
              //how does this work again? no idea
              if old_pos != 0 {
                old_pos - current_file.content[current_file.line_pos].chars().rev().skip(current_length - old_pos + 1).position(|c| c == key_press.key).unwrap_or(0) - 2
              } else {
                old_pos //0
              }
            };
            current_file.cursor_pos = find_pos;
            changed = false;
            self.state = State::None;
          } else if key_press.key == 'x' {
            if current_length > 0 && current_file.cursor_pos < current_length {
              let line = &current_file.content[current_file.line_pos];
              current_file.content[current_file.line_pos] = line.remove(current_file.cursor_pos, 1);
              if current_length == 1 {
                current_file.cursor_pos = 0;
              }
            }
          } else if key_press.key == 'h' {
            current_file.cursor_pos = current_file.cursor_pos.checked_sub(1).unwrap_or(0);
            changed = false;
          } else if key_press.key == 'j' || key_press.key == 'k' {
            if key_press.key == 'j' {
              current_file.line_pos += 1;
              if current_file.line_pos == current_file.content.len() {
                current_file.line_pos = current_file.content.len() - 1;
              }
            } else {
              current_file.line_pos = current_file.line_pos.checked_sub(1).unwrap_or(0);
            }
            let new_length = current_file.content[current_file.line_pos].len();
            current_file.cursor_pos = calc_new_cursor_pos(current_file.cursor_pos, new_length);
            changed = false;
          } else if key_press.key == 'l' {
            if current_length > 0 {
              current_file.cursor_pos += 1;
              if current_file.cursor_pos > current_file.content[current_file.line_pos].len() {
                current_file.cursor_pos = current_file.content[current_file.line_pos].len();
              }
            }
            changed = false;
          } else if key_press.key == '0' && self.maybe_num.is_none() {
            current_file.cursor_pos = 0;
            changed = false;
          } else if key_press.key == '$' {
            //yeah, no `- 1`, that's right
            current_file.cursor_pos = current_file.content[current_file.line_pos].len();
            changed = false;
          } else if key_press.key == '^' {
            current_file.cursor_pos = current_file.content[current_file.line_pos].chars().position(|c| c != ' ').unwrap_or(0);
            changed = false;
          } else if key_press.key == 'r' {
            self.state = State::Replace;
            changed = false;
          } else if key_press.key == 'd' {
            self.state = State::MaybeDelete;
            changed = false;
          } else if key_press.key == 'g' {
            self.state = State::Maybeg;
            changed = false;
          } else if key_press.key == 'G' {
            current_file.line_pos = current_file.content.len() - 1;
            let new_length = current_file.content[current_file.line_pos].len();
            current_file.cursor_pos = calc_new_cursor_pos(current_file.cursor_pos, new_length);
            changed = false;
          } else if key_press.key == 'f' {
            self.state = State::Find;
            changed = false;
          } else if key_press.key == 'F' {
            self.state = State::BackFind;
            changed = false;
          } else if key_press.key.is_digit(10) {
            self.maybe_num = Some(self.maybe_num.unwrap_or(0) * 10 + key_press.key.to_digit(10).unwrap() as usize);
            numbered = true;
            changed = false;
          }
          if !numbered && self.state != State::Maybeg {
            self.maybe_num = None;
          }
          //
        } else if self.mode == Mode::Command {
          self.bottom_message = None;
          let command = self.command.clone().unwrap_or("".to_string());
          if key_press.key == '𐘂' { //the enter key
            new = self.process_command();
            self.command = None;
            self.mode = Mode::Normal;
          } else if key_press.key == '𐘁' { //backspace
            if command.len() > 0 {
              self.command = Some(command[..command.len() - 1].to_string());
            }
          } else {
            self.command = Some(command.to_string() + &key_press.key.to_string());
          }
          changed = false;
        } else {
          return WindowMessageResponse::DoNothing;
        }
        if changed || new {
          self.calc_current(); //too over zealous but whatever
          if changed {
            self.files[self.current_file_index].changed = true;
          }
        }
        self.calc_top_line_pos();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::ClipboardCopy => {
            if self.files.len() > 0 {
              let current_file = &mut self.files[self.current_file_index];
              WindowMessageResponse::Request(WindowManagerRequest::ClipboardCopy(current_file.content[current_file.line_pos].clone()))
            } else {
              WindowMessageResponse::DoNothing
            }
          },
          ShortcutType::ClipboardPaste(copy_string) => {
            if self.mode == Mode::Insert {
              let current_file = &mut self.files[self.current_file_index];
              let line = &current_file.content[current_file.line_pos];
              current_file.content[current_file.line_pos] = line.substring(0, current_file.cursor_pos).to_string() + &copy_string + line.substring(current_file.cursor_pos, line.len());
              current_file.cursor_pos += copy_string.len();
              self.calc_top_line_pos();
              self.calc_current(); //too over zealous but whatever
              self.files[self.current_file_index].changed = true;
              WindowMessageResponse::JustRedraw
            } else {
              WindowMessageResponse::DoNothing
            }
          },
          _ => WindowMessageResponse::DoNothing,
        }
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
      let future_used_width = used_width + 4 + (file_info.name.len() + if file_info.changed { 2 } else { 0 }) * MONO_WIDTH as usize;
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
        DrawInstructions::Rect([used_width, 2], [future_used_width, BAND_HEIGHT - 2], background),
        DrawInstructions::Text([used_width + 2, 2], vec!["nimbus-romono".to_string()], if file_info.changed { "+ ".to_string() } else { String::new() } + &file_info.name, theme_info.alt_text, background, Some(0), Some(MONO_WIDTH)),
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
          instructions.push(DrawInstructions::Text([PADDING, y0], vec!["nimbus-romono".to_string()], (line.1 + 1).to_string(), theme_info.alt_secondary, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
          sub_line_num = 0;
        }
        let x1 = current.line_num_width + PADDING * 2;
        //write actual line
        //line.2
        instructions.push(DrawInstructions::Text([x1, y0], vec!["nimbus-romono".to_string()], line.2.clone(), theme_info.alt_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
        sub_line_num += 1;
        let max = sub_line_num * current.max_chars_per_line;
        let min = max - current.max_chars_per_line;
        if line.1 == current_file.line_pos && current_file.cursor_pos >= min && current_file.cursor_pos < max {
          let top_left = [x1 + (current_file.cursor_pos - min) * MONO_WIDTH as usize, y0];
          //the cursor is on this line, draw it
          instructions.push(DrawInstructions::Rect(top_left, [MONO_WIDTH as usize, LINE_HEIGHT], theme_info.top));
          //draw the char over it
          if line.2.len() > 0 {
            instructions.push(DrawInstructions::Text(top_left, vec!["nimbus-romono".to_string()], line.2.chars().nth(current_file.cursor_pos - min).unwrap().to_string(), theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
          }
        }
      }
    }
    //bottom blue band stuff
    //write mode
    instructions.push(DrawInstructions::Text([0, self.dimensions[1] - BAND_HEIGHT * 2 + 2], vec!["nimbus-romono".to_string()], self.mode.to_string(), theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
    let file_status;
    if self.files.len() > 0 {
      file_status = self.files[self.current_file_index].name.clone();
    } else {
      file_status = "No file open".to_string();
    }
    instructions.push(DrawInstructions::Text([self.dimensions[0] - file_status.len() * (MONO_WIDTH as usize), self.dimensions[1] - BAND_HEIGHT * 2 + 2], vec!["nimbus-romono".to_string()], file_status, theme_info.top_text, theme_info.top, Some(0), Some(MONO_WIDTH)));
    //write command or bottom message
    if self.mode == Mode::Command {
      instructions.push(DrawInstructions::Text([0, self.dimensions[1] - BAND_HEIGHT + 2], vec!["nimbus-romono".to_string()], ":".to_string() + &self.command.clone().unwrap_or("".to_string()), theme_info.top_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
    } else if self.mode == Mode::Normal && self.bottom_message.is_some() {
      instructions.push(DrawInstructions::Text([0, self.dimensions[1] - BAND_HEIGHT + 2], vec!["nimbus-romono".to_string()], self.bottom_message.clone().unwrap(), theme_info.top_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
    }
    instructions
  }

  fn title(&self) -> String {
    "Malvim".to_string()
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

  fn calc_top_line_pos(&mut self) {
    if self.files.len() == 0 {
      return;
    }
    //now, see if the line_pos is still visible from the top_line_pos,
    //if not, move top_line_pos down until it is
    let current_file = &self.files[self.current_file_index];
    let actual_line_pos = self.current.actual_lines.iter().position(|l| l.1 == current_file.line_pos).unwrap();
    if current_file.top_line_pos + self.current.max_lines <= actual_line_pos {
      self.files[self.current_file_index].top_line_pos = actual_line_pos.checked_sub(self.current.max_lines - 1).unwrap_or(0);
    } else if actual_line_pos < current_file.top_line_pos {
      self.files[self.current_file_index].top_line_pos = actual_line_pos;
    }
  }
  
  fn calc_current(&mut self) {
    if self.files.len() == 0 {
      return;
    }
    let current_file = &self.files[self.current_file_index];
    let line_num_width = current_file.content.len().to_string().len() * MONO_WIDTH as usize;
    let max_chars_per_line = (self.dimensions[0] - line_num_width - PADDING * 2) / MONO_WIDTH as usize;
    let actual_lines = calc_actual_lines(current_file.content.iter(), max_chars_per_line, true);
    let max_lines = (self.dimensions[1] - BAND_HEIGHT * 3 - PADDING) / LINE_HEIGHT;
    self.current = Current {
      actual_lines,
      line_num_width,
      max_lines,
      max_chars_per_line,
    };
  }

  fn process_command(&mut self) -> bool {
    let mut parts = self.command.as_ref().unwrap().split(" ");
    let first = parts.next().unwrap();
    let arg = parts.next().unwrap_or("");
    if first == "autoindent" {
      self.autoindent = !self.autoindent;
      self.bottom_message = Some("Autoindent: ".to_string() + &self.autoindent.to_string());
    } else if first == "e" || first == "edit" || ((first == "t" || first == "tabe") && self.files.len() > 0) {
      //find the file and open it
      let mut failed = false;
      let mut new_path = if self.files.len() > 0 && !arg.starts_with("/") {
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
          return true;
        } else {
          self.bottom_message = Some("Failed to open that file".to_string());
        }
      } else {
        self.bottom_message = Some("That is not a file or does not exist".to_string());
      }
    } else if self.files.len() == 0 {
      self.bottom_message = Some("No files are open, so can only do :e(dit)".to_string());
    } else if first.starts_with("/") {
      let current_file = &mut self.files[self.current_file_index];
      if current_file.content.len() > 0 {
        let found_line_no = current_file.content.iter().skip(current_file.line_pos + 1).position(|line| {
          line.contains(&first[1..])
        });
        if let Some(found_line_no) = found_line_no {
          current_file.line_pos = found_line_no + current_file.line_pos + 1;
          current_file.cursor_pos = 0;
        }
      }
    } else if first == "w" || first == "write" {
      let current_file = &self.files[self.current_file_index];
      let _ = write(&current_file.path, &current_file.content.join("\n"));
      self.files[self.current_file_index].changed = false;
      self.bottom_message = Some("Written".to_string());
    } else if first == "q" || first == "quit" {
      self.files.remove(self.current_file_index);
      self.current_file_index = self.current_file_index.checked_sub(1).unwrap_or(0);
      return true;
    } else if first == "p" || first == "tabp" {
      self.current_file_index = self.current_file_index.checked_sub(1).unwrap_or(self.files.len() - 1);
      return true;
    } else if first == "n" || first == "tabn" {
      self.current_file_index += 1;
      if self.current_file_index == self.files.len() {
        self.current_file_index = 0;
      }
      return true;
    } else {
      self.bottom_message = Some("Not a command".to_string());
    }
    false
  }
}

pub fn main() {
  listen(Malvim::new());
}

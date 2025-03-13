use std::vec::Vec;
use std::vec;
use std::sync::mpsc::{ channel, Receiver, Sender };
use std::thread;
use std::process::{ Child, Stdio };
use std::io::{ Read, Write };
use std::time::Duration;
use std::path::PathBuf;
use std::fmt;
use std::fs::read_dir;

use pty_process::blocking;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse, ShortcutType };
use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::utils::{ concat_paths, Substring };
use ming_wm_lib::dirs::home;
use ming_wm_lib::ipc::listen;

//todo: support copy and paste

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 15;
const PADDING: usize = 4;

//at least the ones that starts with ESC[
fn strip_ansi_escape_codes(line: String) -> String {
  let mut new_line = String::new();
  let mut in_ansi = false;
  let mut lc = line.chars().peekable();
  loop {
    let c = lc.next();
    if c.is_none() {
      break;
    }
    let c = c.unwrap();
    if c == '\x1B' && lc.peek() == Some(&'[') {
      in_ansi = true;
    } else if in_ansi {
      if c.is_alphabetic() {
        in_ansi = false;
      }
    } else {
      new_line += &c.to_string()
    }
  }
  new_line
}

#[derive(Default, PartialEq)]
enum Mode {
  #[default]
  Input, //typing in to run command
  Running, //running command, key presses trigger writing output
  Stdin, //key presses writing to stdin of a running command
}

impl fmt::Display for Mode {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
    let write_str = match self {
      Mode::Input=> "INPUT",
      Mode::Running => "RUNNING ('i' to stdin, else output)",
      Mode::Stdin => "STDIN ('esc' to return, 'enter' to send)",
    };
    fmt.write_str(write_str)?;
    Ok(())
  }
}

#[derive(Default)]
pub struct Terminal {
  dimensions: Dimensions,
  mode: Mode,
  lines: Vec<String>,
  actual_lines: Vec<String>, //wrapping
  actual_line_num: usize, //what line # is at the top, for scrolling
  current_input: String,
  current_stdin_input: String,
  current_path: String,
  running_process: Option<Child>,
  process_current_line: Vec<u8>, //bytes of line
  pty_outerr_rx: Option<Receiver<u8>>,
  pty_in_tx: Option<Sender<String>>,
  last_command: Option<String>,
}

//for some reason key presses, then moving the window leaves the old window still there, behind it. weird

impl WindowLike for Terminal {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.current_path = home().unwrap_or(PathBuf::from("/")).to_string_lossy().to_string();
        self.lines = vec!["Mingde Terminal".to_string(), "".to_string()];
        self.calc_actual_lines();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        self.calc_actual_lines();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        match self.mode {
          Mode::Input => {
            if key_press.key == '𐘁' { //backspace
              if self.current_input.len() > 0 {
                self.current_input = self.current_input.remove_last();
              } else {
                return WindowMessageResponse::DoNothing;
              }
            } else if key_press.key == '𐘂' { //the enter key
              self.lines.push("$ ".to_string() + &self.current_input);
              self.last_command = Some(self.current_input.clone());
              self.mode = self.process_command();
              self.current_input = String::new();
            } else if key_press.key == '\t' { //tab
              //autocomplete assuming it's a file system path
              //...mostly working
              let mut useless_tab = true;
              if self.current_input.len() > 0 {
                let partial_path = self.current_input.split(" ").last().unwrap();
                if let Ok(new_path) = concat_paths(&self.current_path, partial_path) {
                  let partial_name;
                  let parent;
                  if self.current_input.ends_with("/") {
                    partial_name = "".to_string();
                    parent = new_path.as_path();
                  } else {
                    //this is just silly
                    partial_name = new_path.clone().file_name().unwrap().to_os_string().to_string_lossy().to_string();
                    parent = new_path.parent().unwrap();
                  };
                  for entry in read_dir(parent).unwrap() {
                    let name = entry.unwrap().path().file_name().unwrap().to_os_string().to_string_lossy().to_string();
                    if name.starts_with(&partial_name) {
                      self.current_input += &name[partial_name.len()..];
                      useless_tab = false;
                      break;
                    }
                  }
                }
              }
              if useless_tab {
                return WindowMessageResponse::DoNothing;
              }
            } else {
              self.current_input += &key_press.key.to_string();
            }
            self.calc_actual_lines();
            self.actual_line_num = self.actual_lines.len().checked_sub(self.get_max_lines()).unwrap_or(0);
            WindowMessageResponse::JustRedraw
          },
          Mode::Running => {
            //update
            let mut changed = false;
            loop {
              if let Ok(ci) = self.pty_outerr_rx.as_mut().unwrap().recv_timeout(Duration::from_millis(5)) {
                if char::from(ci) == '\n' {
                  let pcl_len = self.process_current_line.len();
                  self.lines.push(strip_ansi_escape_codes(String::from_utf8(self.process_current_line.clone()).unwrap_or("?".repeat(pcl_len))));
                  self.process_current_line = Vec::new();
                } else if char::from(ci) == '\r' {
                  //for now, ignore
                  //
                } else {
                  self.process_current_line.push(ci);
                }
                changed = true;
              } else {
                break;
              }
            }
            let running_process = self.running_process.as_mut().unwrap();
            if let Some(_status) = running_process.try_wait().unwrap() {
              //process exited
              self.pty_outerr_rx = None;
              self.mode = Mode::Input;
              self.process_current_line = Vec::new();
              changed = true;
            } else {
              if key_press.key == 'i' {
                self.mode = Mode::Stdin;
                changed = true;
              }
            }
            if changed {
              self.calc_actual_lines();
              WindowMessageResponse::JustRedraw
            } else {
              WindowMessageResponse::DoNothing
            }
          },
          Mode::Stdin => {
            if key_press.key == '𐘃' {
              //esc
              self.mode = Mode::Running;
            } else if key_press.key == '𐘂' {
              //enter
              let _ = self.pty_in_tx.as_mut().unwrap().send(self.current_stdin_input.clone());
              self.mode = Mode::Running;
              let pcl_len = self.process_current_line.len();
              self.lines.push(strip_ansi_escape_codes(String::from_utf8(self.process_current_line.clone()).unwrap_or("?".repeat(pcl_len))) + &self.current_stdin_input);
              self.current_stdin_input = String::new();
              self.process_current_line = Vec::new();
            } else if key_press.key == '𐘁' {
              //backspace
              if self.current_stdin_input.len() > 0 {
                self.current_stdin_input = self.current_stdin_input.remove_last();
              } else {
                return WindowMessageResponse::DoNothing;
              }
            } else {
              self.current_stdin_input += &key_press.key.to_string();
            }
            self.calc_actual_lines();
            WindowMessageResponse::JustRedraw
          },
        }
      },
      WindowMessage::CtrlKeyPress(key_press) => {
        if self.mode == Mode::Running && key_press.key == 'c' {
          //kills and running_process is now None
          let _ = self.running_process.take().unwrap().kill();
          self.mode = Mode::Input;
          WindowMessageResponse::JustRedraw
        } else if self.mode == Mode::Input && (key_press.key == 'p' || key_press.key == 'n') {
          //only the last command is saved unlike other terminals. good enough for me
          if key_press.key == 'p' && self.last_command.is_some() {
            self.current_input = self.last_command.clone().unwrap();
            self.calc_actual_lines();
            WindowMessageResponse::JustRedraw
          } else if key_press.key == 'n' {
            self.current_input = String::new();
            self.calc_actual_lines();
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::ClipboardPaste(copy_string) => {
            if self.mode == Mode::Input || self.mode == Mode::Stdin {
              if self.mode == Mode::Input {
                self.current_input += &copy_string;
              } else {
                self.current_stdin_input += &copy_string;
              }
              self.calc_actual_lines();
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
      instructions.push(DrawInstructions::Text([PADDING, text_y], vec!["nimbus-romono".to_string()], line, theme_info.alt_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
      text_y += LINE_HEIGHT;
    }
    instructions.push(DrawInstructions::Text([PADDING, self.dimensions[1] - LINE_HEIGHT], vec!["nimbus-romono".to_string()], self.mode.to_string(), theme_info.alt_text, theme_info.alt_background, Some(0), Some(MONO_WIDTH)));
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
    (self.dimensions[1] - PADDING * 2 - LINE_HEIGHT) / LINE_HEIGHT
  }

  fn process_command(&mut self) -> Mode {
    if self.current_input.starts_with("clear ") || self.current_input == "clear" {
      self.lines = Vec::new();
      Mode::Input
    } else if self.current_input.starts_with("cd ") {
      let mut cd_split = self.current_input.split(" ");
      cd_split.next().unwrap();
      let arg = cd_split.next().unwrap();
      if let Ok(new_path) = concat_paths(&self.current_path, arg) {
        if new_path.is_dir() {
          self.current_path = new_path.to_str().unwrap().to_string();
        } else {
          self.lines.push("Path not found or not directory".to_string());
        }
      }
      Mode::Input
    } else {
      let (pty, pts) = blocking::open().unwrap();
      self.running_process = Some(blocking::Command::new("sh").arg("-c").arg(&self.current_input).current_dir(&self.current_path).stdin(Stdio::piped()).spawn(pts).unwrap());
      let (tx1, rx1) = channel();
      thread::spawn(move || {
        for ci in pty.bytes() {
          if let Ok(ci) = ci {
            tx1.send(ci).unwrap();
          } else {
            //the process has exited. dead process = dead pty = os input/output error
            break;
          }
        }
      });
      let mut stdin = self.running_process.as_mut().unwrap().stdin.take().unwrap();
      let (tx2, rx2) = channel();
      thread::spawn(move || {
        loop {
          if let Ok(write_line) = rx2.recv() {
            let write_line: String = write_line + "\n";
            stdin.write(write_line.as_bytes()).unwrap();
          } else {
            break;
          }
        }
      });
      self.pty_outerr_rx = Some(rx1);
      self.pty_in_tx = Some(tx2);
      self.process_current_line = Vec::new();
      Mode::Running
    }
  }

  fn calc_actual_lines(&mut self) {
    self.actual_lines = Vec::new();
    let max_chars_per_line = (self.dimensions[0] - PADDING * 2) / MONO_WIDTH as usize;
    let lines_len = self.lines.len();
    let end = if self.mode != Mode::Running || self.process_current_line.len() > 0 {
      lines_len
    } else {
      lines_len - 1
    };
    for line_num in 0..=end {
      let mut working_line = if line_num >= lines_len {
        if self.mode == Mode::Input {
          //must_add_current_line will be false
          "$ ".to_string() + &self.current_input + "█"
        } else {
          let pcl_len = self.process_current_line.len();
          strip_ansi_escape_codes(String::from_utf8(self.process_current_line.clone()).unwrap_or("?".repeat(pcl_len))) + &self.current_stdin_input.clone() + "█"
        }
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


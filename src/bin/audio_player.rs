use std::vec::Vec;
use std::vec;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::{ read_to_string, File };

use rodio::{ Decoder, OutputStream, Sink, Source };
use rand::prelude::*;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::Dimensions;
use ming_wm::themes::ThemeInfo;
use ming_wm::utils::{ concat_paths, format_seconds };
use ming_wm::fs::get_all_files;
use ming_wm::ipc::listen;

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 18;

#[derive(Default)]
struct AudioPlayer {
  dimensions: Dimensions,
  base_directory: String,
  queue: Vec<(PathBuf, u64)>,
  stream: Option<Box<OutputStream>>,
  sink: Option<Sink>,
  command: String,
  response: String,
}

impl WindowLike for AudioPlayer {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == '𐘂' { //the enter key
          self.response = self.process_command();
          self.command = String::new();
        } else if key_press.key == '𐘁' { //backspace
          if self.command.len() > 0 {
            self.command = self.command[..self.command.len() - 1].to_string();
          }
        } else {
          self.command += &key_press.key.to_string();
        }
        WindowMessageResponse::JustRerender
      },
      _ => {
        WindowMessageResponse::DoNothing
      },
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![DrawInstructions::Text([2, self.dimensions[1] - LINE_HEIGHT], "times-new-roman".to_string(), if self.command.len() > 0 { self.command.clone() } else { self.response.clone() }, theme_info.text, theme_info.background, None, None)];
    if let Some(sink) = &self.sink {
      let current = &self.queue[self.queue.len() - sink.len()];
      let current_name = current.0.file_name().unwrap().to_string_lossy().into_owned();
      instructions.push(DrawInstructions::Text([self.dimensions[0] / 2 - current_name.len() * MONO_WIDTH as usize / 2, 2], "times-new-romono".to_string(), current_name.clone(), theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)));
      let time_string = format!("{}/{}", format_seconds(sink.get_pos().as_secs()), format_seconds(current.1));
      instructions.push(DrawInstructions::Text([self.dimensions[0] / 2 - time_string.len() * MONO_WIDTH as usize / 2, LINE_HEIGHT + 2], "times-new-romono".to_string(), time_string, theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)));
    }
    //
    instructions
  }

  //properties

  fn title(&self) -> String {
    "Audio Player".to_string()
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [500, 200]
  }

  fn resizable(&self) -> bool {
    true
  }
}

impl AudioPlayer {
  pub fn new() -> Self {
    let mut ap: Self = Default::default();
    ap.base_directory = "/".to_string();
    ap
  }

  //t: toggle pause/play
  //l: next/skip
  //j: volume down
  //k: volume up
  //b <dir>: set base directory
  //p <dir>/<playlist file>: play directory or playlist in random order
  //todo: h for help?
  //just hit enter or any key to refresh
  fn process_command(&mut self) -> String {
    if self.command.len() == 1 {
      if let Some(sink) = &mut self.sink {
        if self.command == "t" {
          if sink.is_paused() {
            sink.play();
            return "Resumed".to_string();
          } else {
            sink.pause();
            return "Paused".to_string();
          }
        } else if self.command == "h" {
          //
        } else if self.command == "l" {
          sink.skip_one();
          return "Skipped".to_string();
        } else if self.command == "j" {
          sink.set_volume(sink.volume() - 0.1);
          return "Volume decreased".to_string();
        } else if self.command == "k" {
          sink.set_volume(sink.volume() + 0.1);
          return "Volume increased".to_string();
        }
      }
    } else {
      let parts: Vec<&str> = self.command.split(" ").collect();
      if self.command.starts_with("p ") {
        if parts.len() == 2 {
          if let Ok(new_path) = concat_paths(&self.base_directory, parts[1]) {
            if new_path.exists() {
              if let Some(sink) = &mut self.sink {
                sink.clear();
              }
              let mut queue = if parts[1].ends_with(".playlist") {
                let mut queue = Vec::new();
                let contents = read_to_string(new_path).unwrap();
                for line in contents.split("\n") {
                  //todo: handle more edge cases later
                  if line.ends_with("/*") {
                    queue.extend(get_all_files(concat_paths(&self.base_directory, &line[..line.len() - 2]).unwrap()));
                  } else if line.len() > 0 {
                    queue.push(concat_paths(&self.base_directory, &(line.to_owned() + ".mp3")).unwrap());
                  }
                }
                queue
              } else {
                get_all_files(PathBuf::from(new_path))
              };
              let mut rng = rand::thread_rng();
              queue.shuffle(&mut rng);
              let (stream, stream_handle) = OutputStream::try_default().unwrap();
              let sink = Sink::try_new(&stream_handle).unwrap();
              self.queue = Vec::new();
              for item in &queue {
                let file = BufReader::new(File::open(item).unwrap());
                let decoded = Decoder::new(file).unwrap();
                self.queue.push((item.clone(), decoded.total_duration().unwrap().as_secs()));
                sink.append(decoded);
              }
              self.stream = Some(Box::new(stream));
              self.sink = Some(sink);
              return "Playing".to_string();
            }
          }
        }
      } else if self.command.starts_with("b ") {
        if parts.len() == 2 {
          if let Ok(new_path) = concat_paths(&self.base_directory, parts[1]) {
            if new_path.exists() {
              self.base_directory = new_path.to_str().unwrap().to_string();
              return "Set new base directory".to_string();
            }
          }
        }
      }
    }
    String::new()
  }
}

pub fn main() {
  listen(AudioPlayer::new());
}


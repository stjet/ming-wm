use std::vec::Vec;
use std::vec;
use std::io::BufReader;
use std::path::PathBuf;
use std::fs::{ read_to_string, File };
use std::time::{ Duration, SystemTime, UNIX_EPOCH };
use std::thread;
use std::sync::{ Arc, Mutex };

use rodio::{ Decoder, OutputStream, Sink, Source };
use rand::{ SeedableRng, prelude::SliceRandom, rngs::SmallRng };
use id3::TagLike;
use mp4ameta;
use metaflac;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::utils::{ concat_paths, get_all_files, path_autocomplete, format_seconds, Substring };
use ming_wm_lib::dirs::home;
use ming_wm_lib::ipc::listen;

fn get_artist(path: &PathBuf) -> Option<String> {
  let ext = path.extension().unwrap();
  if ext == "mp4" {
    let tag = mp4ameta::Tag::read_from_path(path).unwrap();
    tag.artist().map(|s| s.to_string())
  } else if ext == "flac" {
    let tag = metaflac::Tag::read_from_path(path).unwrap();
    let x = if let Some(mut artists) = tag.get_vorbis("Artist") {
      Some(artists.next().unwrap().to_string()) //get the first one
    } else {
      None
    };
    x
  } else if ext == "mp3" {
    let tag = id3::Tag::read_from_path(path).unwrap();
    tag.artist().map(|s| s.to_string())
  } else {
    None
  }
}

const MONO_WIDTH: u8 = 10;
const LINE_HEIGHT: usize = 18;

type QueueItem = (PathBuf, u64, Option<String>);

struct InternalPlayer {
  pub queue: Vec<QueueItem>,
  pub sink: Sink,
}

impl InternalPlayer {
  fn add(internal: Arc<Mutex<InternalPlayer>>, queue: Vec<PathBuf>) {
    thread::spawn(move || {
      for item in queue {
        let file = BufReader::new(File::open(&item).unwrap());
        //slightly faster for mp3s? since it doesn't need to check if it is .wav, etc. but maybe not
        let decoded = if item.ends_with(".mp3") { Decoder::new_mp3(file) } else { Decoder::new(file) }.unwrap();
        let mut internal_locked = internal.lock().unwrap();
        (*internal_locked).queue.push((item.clone(), decoded.total_duration().unwrap().as_secs(), get_artist(&item)));
        (*internal_locked).sink.append(decoded);
        (*internal_locked).sink.play();
      }
    });
  }
}

pub struct AudioPlayer {
  dimensions: Dimensions,
  base_directory: String,
  _stream: Box<OutputStream>,
  internal: Arc<Mutex<InternalPlayer>>,
  command: String,
  response: String,
}

impl WindowLike for AudioPlayer {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    //
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == '𐘂' { //the enter key
          self.response = self.process_command();
          self.command = String::new();
        } else if key_press.key == '𐘁' { //backspace
          if self.command.len() > 0 {
            self.command = self.command.remove_last();
          }
        } else if key_press.key == '\t' { //tab
          let mut parts = self.command.split(" ");
          let parts_len = parts.clone().count();
          if parts_len == 2 {
            if let Some(add) = path_autocomplete(&self.base_directory, parts.nth(1).unwrap()) {
              self.command += &add;
            } else {
              return WindowMessageResponse::DoNothing;
            }
          } else {
            return WindowMessageResponse::DoNothing;
          }
        } else if key_press.is_regular() {
          self.command += &key_press.key.to_string();
        } else {
          return WindowMessageResponse::DoNothing
        }
        WindowMessageResponse::JustRedraw
      },
      _ => {
        WindowMessageResponse::DoNothing
      },
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![DrawInstructions::Text([2, self.dimensions[1] - LINE_HEIGHT], vec!["nimbus-roman".to_string()], if self.command.len() > 0 { self.command.clone() } else { self.response.clone() }, theme_info.text, theme_info.background, None, None)];
    let internal_locked = self.internal.lock().unwrap();
    let sink_len = internal_locked.sink.len();
    if sink_len > 0 {
      let queue = &internal_locked.queue;
      let current = &queue[queue.len() - sink_len];
      let current_name = current.0.file_name().unwrap().to_string_lossy().into_owned();
      instructions.push(DrawInstructions::Text([self.dimensions[0] / 2 - current_name.len() * MONO_WIDTH as usize / 2, 2], vec!["nimbus-romono".to_string(), "shippori-mincho".to_string()], current_name.clone(), theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)));
      if let Some(artist) = &current.2 {
        let artist_string = "by ".to_string() + &artist;
        instructions.push(DrawInstructions::Text([self.dimensions[0] / 2 - artist_string.len() * MONO_WIDTH as usize / 2, LINE_HEIGHT + 2], vec!["nimbus-romono".to_string()], artist_string, theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)));
      }
      let time_string = format!("{}/{}", format_seconds(internal_locked.sink.get_pos().as_secs()), format_seconds(current.1));
      instructions.push(DrawInstructions::Text([self.dimensions[0] / 2 - time_string.len() * MONO_WIDTH as usize / 2, LINE_HEIGHT * 2 + 2], vec!["nimbus-romono".to_string()], time_string, theme_info.text, theme_info.background, Some(0), Some(MONO_WIDTH)));
    } else {
      instructions.push(DrawInstructions::Text([2, 2], vec!["nimbus-roman".to_string()], "type to write commands, enter to execute.".to_string(), theme_info.text, theme_info.background, None, None));
      instructions.push(DrawInstructions::Text([2, 2 + LINE_HEIGHT], vec!["nimbus-roman".to_string()], "See help in start menu for commands.".to_string(), theme_info.text, theme_info.background, None, None));
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
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    Self {
      dimensions: Default::default(),
      base_directory: home().unwrap_or(PathBuf::from("/")).to_string_lossy().to_string(),
      _stream: Box::new(stream),
      internal: Arc::new(Mutex::new(InternalPlayer {
        queue: Vec::new(),
        sink: Sink::try_new(&stream_handle).unwrap(),
      })),
      command: Default::default(),
      response: Default::default(),
    }
  }

  //t: toggle pause/play
  //l: next/skip
  //j: volume down
  //k: volume up
  //b <dir>: set base directory
  //p <dir>/<playlist file>: play directory or playlist in random order
  //a <dir>/<playlist file>: same as p but appends to queue instead of clearing
  //just hit enter or any key to refresh
  fn process_command(&mut self) -> String {
    if self.command.len() == 1 {
      let sink = &(*self.internal.lock().unwrap()).sink;
      if self.command == "t" {
        if sink.is_paused() {
          sink.play();
          return "Resumed".to_string();
        } else {
          sink.pause();
          return "Paused".to_string();
        }
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
    } else {
      let parts: Vec<&str> = self.command.split(" ").collect();
      if self.command.starts_with("p ") || self.command.starts_with("a ") {
        if parts.len() == 2 {
          if let Ok(new_path) = concat_paths(&self.base_directory, parts[1]) {
            if new_path.exists() {
              let mut queue = if parts[1].ends_with(".playlist") {
                let mut queue = Vec::new();
                let contents = read_to_string(new_path).unwrap();
                for line in contents.split("\n") {
                  //todo: handle more edge cases later
                  if line.ends_with("/*") {
                    queue.extend(get_all_files(concat_paths(&self.base_directory, &line[..line.len() - 2]).unwrap()));
                  } else if line.len() > 0 {
                    //if no file ext, assumes mp3
                    queue.push(concat_paths(&self.base_directory, &(line.to_owned() + if line.contains(".") { "" } else { ".mp3" })).unwrap());
                  }
                }
                queue
              } else if parts[1].ends_with(".mp3") {
                vec![concat_paths(&self.base_directory, parts[1]).unwrap()]
              } else {
                get_all_files(PathBuf::from(new_path))
              };
              let mut rng = SmallRng::seed_from_u64(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
              queue.shuffle(&mut rng);
              if self.command.starts_with("p ") {
                let mut locked_internal = self.internal.lock().unwrap();
                (*locked_internal).sink.clear();
                (*locked_internal).queue = Vec::new();
              }
              InternalPlayer::add(Arc::clone(&self.internal), queue);
              //to hopefully allow the first file to be loaded so info displays
              thread::sleep(Duration::from_millis(10));
              return if self.command.starts_with("p ") {
                "Playing".to_string()
              } else {
                "Appended".to_string()
              };
            }
          }
        }
      } else if self.command.starts_with("b ") {
        if parts.len() == 2 {
          let new_path = if parts[1].starts_with("/") { Ok(PathBuf::from(parts[1])) } else { concat_paths(&self.base_directory, parts[1]) };
          if let Ok(new_path) = new_path {
            if new_path.exists() {
              self.base_directory = new_path.to_str().unwrap().to_string();
              return "Set new base directory".to_string();
            } else {
              return "Failed to set new base directory".to_string();
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


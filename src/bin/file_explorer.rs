use std::vec::Vec;
use std::vec;
use std::fs::{ read_dir, metadata, Metadata };
use std::path::PathBuf;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::ipc::listen;

const HEIGHT: usize = 20;

struct DirectoryChild {
  //if some, use instead of file/dir name
  override_name: Option<String>,
  path: PathBuf,
  is_file: bool,
}

#[derive(Default, PartialEq)]
enum State {
  #[default]
  List,
  Info,
}

#[derive(Default)]
pub struct FileExplorer {
  dimensions: Dimensions,
  current_path: PathBuf,
  current_dir_contents: Vec<DirectoryChild>,
  //for scrolling and selecting dirs
  position: usize,
  top_position: usize,
  state: State,
  metadata: Option<Metadata>,
}

impl WindowLike for FileExplorer {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.current_path = PathBuf::from("/");
        self.dimensions = dimensions;
        self.current_dir_contents = self.get_current_dir_contents();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        self.state = State::List;
        if key_press.key == '𐘂' { //the enter key
          if self.current_dir_contents.len() > 0 {
            let selected_entry = &self.current_dir_contents[self.position];
            if !selected_entry.is_file {
              self.current_path = selected_entry.path.clone();
              self.current_dir_contents = self.get_current_dir_contents();
              self.position = 0;
              self.top_position = 0;
              return WindowMessageResponse::JustRedraw;
            }
          }
          WindowMessageResponse::DoNothing
        } else if key_press.key == 'j' || key_press.key == 'k' {
          if key_press.key == 'j' {
            //down
            if self.position == self.current_dir_contents.len() - 1 {
              self.position = 0;
            } else {
              self.position += 1;
            }
          } else {
            //up
            if self.position == 0 {
              self.position = self.current_dir_contents.len() - 1;
            } else {
              self.position -= 1;
            }
          }
          //calculate position
          let max_height = self.dimensions[1] - HEIGHT;
          if self.position > self.top_position {
            let current_height = (self.position - self.top_position + 1) * HEIGHT;
            if current_height > self.dimensions[1] {
              //somehow this is slightly off sometimes
              self.top_position += (current_height - max_height).div_ceil(HEIGHT);
            }
          } else {
            self.top_position = self.position;
          };
          WindowMessageResponse::JustRedraw
        } else if key_press.key == 'i' {
          self.state = State::Info;
          let selected_entry = &self.current_dir_contents[self.position];
          self.metadata = Some(metadata(&selected_entry.path).unwrap());
          WindowMessageResponse::JustRedraw
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = Vec::new();
    if self.state == State::List {
      //top bar with path name
      instructions.push(DrawInstructions::Text([5, 0], vec!["nimbus-roman".to_string(), "shippori-mincho".to_string()], "Current: ".to_string() + &self.current_path.to_string_lossy().to_string(), theme_info.text, theme_info.background, None, None));
      //the actual files and directories
      let mut start_y = HEIGHT;
      let mut i = self.top_position;
      for entry in self.current_dir_contents.iter().skip(self.top_position) {
        if start_y > self.dimensions[1] {
          break;
        }
        let is_selected = i == self.position;
        if is_selected {
          instructions.push(DrawInstructions::Rect([0, start_y], [self.dimensions[0], HEIGHT], theme_info.top));
        }
        //unwrap_or not used because "Arguments passed to unwrap_or are eagerly evaluated", apparently
        let name = entry.override_name.clone();
        let name = if name.is_none() {
          entry.path.file_name().unwrap().to_os_string().into_string().unwrap()
        } else {
          name.unwrap()
        };
        instructions.push(DrawInstructions::Text([5, start_y], vec!["nimbus-roman".to_string(), "shippori-mincho".to_string()], name, if is_selected { theme_info.top_text } else { theme_info.text }, if is_selected { theme_info.top } else { theme_info.background }, None, None));
        start_y += HEIGHT;
        i += 1;
      }
    } else if self.state == State::Info {
      let metadata = self.metadata.clone().unwrap();
      let mut start_y = HEIGHT;
      let bytes_len = metadata.len();
      instructions.push(DrawInstructions::Text([5, start_y], vec!["nimbus-roman".to_string()], format!("Size: {} mb ({} b)", bytes_len / (1024_u64).pow(2), bytes_len), theme_info.text, theme_info.background, None, None));
      start_y += HEIGHT;
      //todo: other stuff
      //
    }
    instructions
  }

  fn title(&self) -> String {
    "File Explorer".to_string()
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

impl FileExplorer {
  pub fn new() -> Self {
    Default::default()
  }

  //should include .. if not /
  fn get_current_dir_contents(&self) -> Vec<DirectoryChild> {
    let mut contents = Vec::new();
    if self.current_path != PathBuf::from("/") {
      contents.push(DirectoryChild {
        override_name: Some("..".to_string()),
        is_file: false,
        path: self.current_path.parent().unwrap().to_owned(),
      });
    }
    contents.extend(read_dir(&self.current_path).unwrap().map(|entry| {
      let path = entry.unwrap().path();
      DirectoryChild {
        override_name: None,
        is_file: path.is_file(),
        path,
      }
    }));
    contents
  }
}

pub fn main() {
  listen(FileExplorer::new());
}

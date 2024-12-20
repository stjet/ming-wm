use std::vec::Vec;
use std::vec;
use std::fs::read_dir;
use std::path::PathBuf;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::Dimensions;
use ming_wm::themes::ThemeInfo;
use ming_wm::ipc::listen;

struct DirectoryChild {
  //if some, use instead of file/dir name
  override_name: Option<String>,
  path: PathBuf,
  is_file: bool,
  //can only be true if dir
  //if true, means the contents of this dir should be visible too, even though it isn't the current path. like a tree
  tree_open: bool,
}

#[derive(Default)]
pub struct FileExplorer {
  dimensions: Dimensions,
  current_path: PathBuf,
  current_dir_contents: Vec<DirectoryChild>,
  //for scrolling and selecting dirs
  position: usize,
}

impl WindowLike for FileExplorer {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.current_path = PathBuf::from("/");
        self.dimensions = dimensions;
        self.current_dir_contents = self.get_current_dir_contents();
        WindowMessageResponse::JustRerender
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == '𐘂' { //the enter key
          if self.current_dir_contents.len() > 0 {
            let selected_entry = &self.current_dir_contents[self.position];
            if !selected_entry.is_file {
              self.current_path = selected_entry.path.clone();
              self.current_dir_contents = self.get_current_dir_contents();
              self.position = 0;
              return WindowMessageResponse::JustRerender;
            }
          }
          WindowMessageResponse::DoNothing
        } else if key_press.key == 'j' {
          //down
          if self.position == self.current_dir_contents.len() - 1 {
            self.position = 0;
          } else {
            self.position += 1;
          }
          WindowMessageResponse::JustRerender
        } else if key_press.key == 'k' {
          //up
          if self.position == 0 {
            self.position = self.current_dir_contents.len() - 1;
          } else {
            self.position -= 1;
          }
          WindowMessageResponse::JustRerender
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = Vec::new();
    //top bar with path name and editing
    //
    //the actual files and directories
    let mut start_y = 0;
    let mut i = 0;
    for entry in &self.current_dir_contents {
      if start_y > self.dimensions[1] {
        break;
      }
      let is_selected = i == self.position;
      if is_selected {
        instructions.push(DrawInstructions::Rect([0, start_y], [self.dimensions[0], 20], theme_info.top));
      }
      //unwrap_or not used because "Arguments passed to unwrap_or are eagerly evaluated", apparently
      let name = entry.override_name.clone();
      let name = if name.is_none() {
        entry.path.file_name().unwrap().to_os_string().into_string().unwrap()
      } else {
        name.unwrap()
      };
      instructions.push(DrawInstructions::Text([5, start_y], vec!["times-new-roman".to_string(), "shippori-mincho".to_string()], name, if is_selected { theme_info.top_text } else { theme_info.text }, if is_selected { theme_info.top } else { theme_info.background }, None, None));
      start_y += 20;
      i += 1;
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
        tree_open: false,
        path: self.current_path.parent().unwrap().to_owned(),
      });
    }
    contents.extend(read_dir(&self.current_path).unwrap().map(|entry| {
      let path = entry.unwrap().path();
      DirectoryChild {
        override_name: None,
        is_file: path.is_file(),
        tree_open: false,
        path,
      }
    }));
    contents
  }
}

pub fn main() {
  listen(FileExplorer::new());
}


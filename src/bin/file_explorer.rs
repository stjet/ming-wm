use std::vec::Vec;
use std::vec;
use std::fs::read_dir;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::Dimensions;
use ming_wm::themes::ThemeInfo;
use ming_wm::ipc::listen;

#[derive(Default)]
pub struct FileExplorer {
  dimensions: Dimensions,
  current_path: String,
  //current_dir_contents:
}

impl WindowLike for FileExplorer {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.current_path = "/".to_string();
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        //
        WindowMessageResponse::DoNothing
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    //top bar with path name and editing
    vec![]
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
  fn read_current_dir_contents(&self) {
    //
  }
}

pub fn main() {
  listen(FileExplorer::new());
}


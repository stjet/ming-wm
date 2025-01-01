use std::vec::Vec;
use std::boxed::Box;
use std::fs::{ read_to_string, read_dir };
use std::path::PathBuf;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::paragraph::Paragraph;
use crate::components::Component;

pub struct Help {
  dimensions: Dimensions,
  file_index: usize,
  files: Vec<PathBuf>,
  paragraph: Option<Box<Paragraph<()>>>,
}

impl WindowLike for Help {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.paragraph = Some(Box::new(Paragraph::new("help".to_string(), [2, 22], [self.dimensions[0] - 4, self.dimensions[1] - 24], "Press the 'h' and 'l' keys to read the different help pages".to_string(), ())));
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.key == 'h' || key_press.key == 'l' {
          if key_press.key == 'h' {
            if self.file_index == 0 {
              self.file_index = self.files.len() - 1;
            } else {
              self.file_index -= 1;
            }
          } else {
            if self.file_index == self.files.len() - 1 {
              self.file_index = 0;
            } else {
              self.file_index += 1;
            }
          }
          self.paragraph.as_mut().unwrap().new_text(read_to_string(self.files[self.file_index].clone()).unwrap());
          WindowMessageResponse::JustRedraw
        } else if self.paragraph.as_mut().unwrap().handle_message(WindowMessage::KeyPress(key_press)).is_some() {
          WindowMessageResponse::JustRedraw
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![DrawInstructions::Text([2, 2], vec!["times-new-romono".to_string()], self.files[self.file_index].clone().file_name().unwrap().to_string_lossy().to_string(), theme_info.text, theme_info.background, Some(0), None)];
    instructions.extend(self.paragraph.as_ref().unwrap().draw(theme_info));
    instructions
  }

  //properties
  fn title(&self) -> String {
    "About".to_string()
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [500, 600]
  }
}

impl Help {
  pub fn new() -> Self {
    let mut files = vec![PathBuf::from("docs/system/shortcuts.md")];
    for entry in read_dir("docs/window-likes").unwrap() {
      files.push(entry.unwrap().path());
    }
    Self {
      dimensions: [0, 0],
      file_index: 0,
      files,
      paragraph: None,
    }
  }
}


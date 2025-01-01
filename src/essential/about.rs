use std::vec::Vec;
use std::boxed::Box;
use std::fs::read_to_string;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::paragraph::Paragraph;

pub struct About {
  dimensions: Dimensions,
  components: Vec<Box<dyn Component<()> + Send>>,
}

impl WindowLike for About {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.components.push(Box::new(Paragraph::new("help".to_string(), [2, 2], [self.dimensions[0] - 4, self.dimensions[1] - 4], read_to_string("docs/system/README.md").unwrap_or("docs/system/README.md not found".to_string()), ())));
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        if self.components[0].handle_message(WindowMessage::KeyPress(key_press)).is_some() {
          WindowMessageResponse::JustRedraw
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    self.components[0].draw(theme_info)
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

impl About {
  pub fn new() -> Self {
    Self {
      dimensions: [0, 0],
      components: Vec::new(),
    }
  }
}


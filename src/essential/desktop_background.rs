use std::vec;
use std::vec::Vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, TASKBAR_HEIGHT, INDICATOR_HEIGHT };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

pub struct DesktopBackground {
  dimensions: Dimensions,
}

impl WindowLike for DesktopBackground {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  //simple
  fn draw(&self, _theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![DrawInstructions::Rect([0, 0], self.dimensions, [0, 128, 128])]
  }

  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::DesktopBackground
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    [dimensions[0], dimensions[1] - TASKBAR_HEIGHT - INDICATOR_HEIGHT]
  }
}

impl DesktopBackground {
  pub fn new() -> Self {
    Self { dimensions: [0, 0] }
  }
}


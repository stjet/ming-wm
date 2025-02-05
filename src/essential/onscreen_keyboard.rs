use std::vec;
use std::vec::Vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

#[derive(Default)]
pub struct OnscreenKeyboard {
  dimensions: Dimensions,
  //
}

impl WindowLike for OnscreenKeyboard {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      //
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![DrawInstructions::Rect([0, 0], self.dimensions, theme_info.background)];
    //
    instructions
  }
  //
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::OnscreenKeyboard
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    [dimensions[0] - 175, 250]
  }
}

impl OnscreenKeyboard {
  pub fn new() -> Self {
    Self::default()
  }
}


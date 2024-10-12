use std::vec;
use std::vec::Vec;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, INDICATOR_HEIGHT };
use crate::messages::{ WindowMessage, WindowMessageResponse, ShortcutType };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

const WIDTH: usize = 15;

pub struct WorkspaceIndicator {
  dimensions: Dimensions,
  current_workspace: u8,
}

impl WindowLike for WorkspaceIndicator {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::SwitchWorkspace(workspace) => {
            self.current_workspace = workspace;
            WindowMessageResponse::JustRerender
          }
          _ => WindowMessageResponse::DoNothing,
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  //simple
  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      //background
      DrawInstructions::Rect([0, 0], [self.dimensions[0], self.dimensions[1] - 1], theme_info.background),
      //bottom border
      DrawInstructions::Rect([0, self.dimensions[1] - 1], [self.dimensions[0], 1], theme_info.border_right_bottom),
    ];
    for w in 0..9 {
      if w == self.current_workspace as usize {
        instructions.push(DrawInstructions::Rect([w * WIDTH, 0], [WIDTH, self.dimensions[1]], theme_info.top));
        instructions.push(DrawInstructions::Text([w * WIDTH + 5, 4], "times-new-roman", (w + 1).to_string(), theme_info.text_top, theme_info.top, None));
      } else {
        instructions.push(DrawInstructions::Text([w * WIDTH + 5, 4], "times-new-roman", (w + 1).to_string(), theme_info.text, theme_info.background, None));
      }
    }
    instructions
  }

  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::WorkspaceIndicator
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    [dimensions[0], INDICATOR_HEIGHT]
  }
}

impl WorkspaceIndicator {
  pub fn new() -> Self {
    Self {
      dimensions: [0, 0],
      current_workspace: 0,
    }
  }
}



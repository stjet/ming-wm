use std::vec;
use std::vec::Vec;
use std::time::{ SystemTime, UNIX_EPOCH };

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, INDICATOR_HEIGHT };
use crate::messages::{ WindowMessage, WindowMessageResponse, ShortcutType };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;

const WIDTH: usize = 15;
const ONE_MINUTE: u64 = 60;
const ONE_HOUR: u64 = 60 * ONE_MINUTE;
const ONE_DAY: u64 = 24 * ONE_HOUR;

pub struct WorkspaceIndicator {
  dimensions: Dimensions,
  current_workspace: u8,
}

impl WindowLike for WorkspaceIndicator {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::SwitchWorkspace(workspace) => {
            self.current_workspace = workspace;
            WindowMessageResponse::JustRedraw
          },
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
        instructions.push(DrawInstructions::Text([w * WIDTH + 5, 4], vec!["times-new-roman".to_string()], (w + 1).to_string(), theme_info.top_text, theme_info.top, None, None));
      } else {
        instructions.push(DrawInstructions::Text([w * WIDTH + 5, 4], vec!["times-new-roman".to_string()], (w + 1).to_string(), theme_info.text, theme_info.background, None, None));
      }
    }
    //also add the utc time in the right edge
    let today_secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() % ONE_DAY;
    let hours = (today_secs / ONE_HOUR).to_string();
    let minutes = ((today_secs % ONE_HOUR) / ONE_MINUTE).to_string();
    let time_string = format!("{}:{}~ UTC", if hours.len() == 1 { "0".to_string() + &hours } else { hours }, if minutes.len() == 1 { "0".to_string() + &minutes } else { minutes });
    instructions.push(DrawInstructions::Text([self.dimensions[0] - 90, 4], vec!["times-new-roman".to_string()], time_string, theme_info.text, theme_info.background, None, None));
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



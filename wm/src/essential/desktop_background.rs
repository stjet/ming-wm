use std::vec;
use std::vec::Vec;
use std::fs::File;
use std::io::Read;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType, TASKBAR_HEIGHT, INDICATOR_HEIGHT };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse, ShortcutType };
use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::utils::{ hex_to_u8, is_hex };
use ming_wm_lib::dirs::config_dir;

pub struct DesktopBackground {
  dimensions: Dimensions,
  current_workspace: u8,
}

impl WindowLike for DesktopBackground {
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
  fn draw(&self, _theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    if let Ok(mut file) = File::open(format!("{}/ming-wm/desktop-background", config_dir().unwrap().into_os_string().into_string().unwrap())) {
      let mut contents = String::new();
      file.read_to_string(&mut contents).unwrap();
      let lines: Vec<&str> = contents.split("\n").collect();
      if lines.len() > self.current_workspace.into() {
        let line = lines[self.current_workspace as usize];
        if line.starts_with("#") && line.len() == 7 {
          let line_hex = &line[1..];
          //if all characters are valid hex
          if line_hex.find(|c| !is_hex(c)).is_none() {
            let mut chars = line_hex.chars();
            let color = [hex_to_u8(chars.next().unwrap(), chars.next().unwrap()), hex_to_u8(chars.next().unwrap(), chars.next().unwrap()), hex_to_u8(chars.next().unwrap(), chars.next().unwrap())];
            return vec![DrawInstructions::Rect([0, 0], self.dimensions, color)];
          }
        } else if line.len() > 1 {
          //first character of line is either r or any other character, but is not part of the path
          return vec![DrawInstructions::Bmp([0, 0], line[1..].to_string(), line.starts_with('r'))];
        }
      }
    }
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
    Self {
      dimensions: [0, 0],
      current_workspace: 0,
    }
  }
}


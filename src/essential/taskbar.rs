use std::vec;
use std::vec::Vec;
use std::boxed::Box;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, TASKBAR_HEIGHT };
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest, ShortcutType, InfoType, WindowsVec };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::toggle_button::{ ToggleButton, ToggleButtonAlignment };

const PADDING: usize = 4;
const META_WIDTH: usize = 175; //of the window button

#[derive(Clone)]
enum TaskbarMessage {
  ShowStartMenu,
  HideStartMenu,
  Nothing,
  //
}

pub struct Taskbar {
  dimensions: Dimensions,
  components: Vec<Box<dyn Component<TaskbarMessage> + Send>>,
  windows_in_workspace: WindowsVec,
  focused_id: usize,
}

impl WindowLike for Taskbar {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.components = vec![
          Box::new(ToggleButton::new("start-button".to_string(), [PADDING, PADDING], [44, self.dimensions[1] - (PADDING * 2)], "Start".to_string(), TaskbarMessage::ShowStartMenu, TaskbarMessage::HideStartMenu, false, Some(ToggleButtonAlignment::Left))),
        ];
        WindowMessageResponse::JustRerender
      },
      WindowMessage::Shortcut(shortcut) => {
        match shortcut {
          ShortcutType::StartMenu => {
            let start_index = self.components.iter().position(|c| c.name() == "start-button").unwrap();
            let start_response = self.components[start_index].handle_message(WindowMessage::FocusClick);
            self.handle_taskbar_message(start_response)
          }
          _ => WindowMessageResponse::DoNothing,
        }
      },
      WindowMessage::Info(info) => {
        match info {
          InfoType::WindowsInWorkspace(windows, focused_id) => {
            self.windows_in_workspace = windows;
            self.focused_id = focused_id;
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
      //top thin white border
      DrawInstructions::Rect([0, 0], [self.dimensions[0], 1], theme_info.border_left_top),
      //the actual taskbar background
      DrawInstructions::Rect([0, 1], [self.dimensions[0], self.dimensions[1] - 1], theme_info.background),
    ];
    for component in &self.components {
      instructions.extend(component.draw(theme_info));
    }
    for wi in 0..self.windows_in_workspace.len() {
      //if too many windows to fit in taskbar...
      if wi > (self.dimensions[0] - 200) / META_WIDTH {
        //
        break;
      }
      let info = &self.windows_in_workspace[wi];
      let name = &info.1;
      let mut b = ToggleButton::new(name.to_string() + "-window", [PADDING * 2 + 44 + (META_WIDTH + PADDING) * wi, PADDING], [META_WIDTH, self.dimensions[1] - (PADDING * 2)], name.to_string(), TaskbarMessage::Nothing, TaskbarMessage::Nothing, false, Some(ToggleButtonAlignment::Left));
      b.inverted = info.0 == self.focused_id;
      instructions.extend(b.draw(theme_info));
    }
    instructions
  }

  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Taskbar
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    [dimensions[0], TASKBAR_HEIGHT]
  }
}

impl Taskbar {
  pub fn new() -> Self {
    Self {
      dimensions: [0, 0],
      components: Vec::new(),
      windows_in_workspace: Vec::new(),
      focused_id: 0,
    }
  }

  fn handle_taskbar_message(&mut self, message: Option<TaskbarMessage>) -> WindowMessageResponse {
    if let Some(message) = message {
      match message {
        TaskbarMessage::ShowStartMenu => {
          WindowMessageResponse::Request(WindowManagerRequest::OpenWindow("StartMenu".to_string()))
        },
        TaskbarMessage::HideStartMenu => {
          WindowMessageResponse::Request(WindowManagerRequest::CloseStartMenu)
        },
        _ => WindowMessageResponse::DoNothing,
      }
    } else {
      //maybe should be JustRerender?
      WindowMessageResponse::DoNothing
    }
  }
}



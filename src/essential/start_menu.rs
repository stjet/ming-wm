use std::vec;
use std::vec::Vec;
use std::boxed::Box;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest };
use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::dirs::exe_dir;
use crate::fs::{ ExeWindowInfos, get_all_executable_windows };
use crate::components::Component;
use crate::components::highlight_button::HighlightButton;

static CATEGORIES: [&'static str; 9] = ["About", "Utils", "Games", "Editing", "Files", "Internet", "Misc", "Help", "Logout"];

#[derive(Clone)]
enum StartMenuMessage {
  CategoryClick(&'static str),
  WindowClick(String),
  Back,
  ChangeAcknowledge,
}

#[derive(Default)]
pub struct StartMenu {
  dimensions: Dimensions,
  executable_windows: ExeWindowInfos,
  components: Vec<Box<HighlightButton<StartMenuMessage>>>,
  current_focus: String,
  y_each: usize,
}

impl WindowLike for StartMenu {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.y_each = (self.dimensions[1] - 1) / CATEGORIES.len();
        self.add_category_components();
        self.executable_windows = get_all_executable_windows();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        //up and down
        if key_press.key == 'k' || key_press.key == 'j' {
          let old_focus_index = self.get_focus_index().unwrap();
          self.components[old_focus_index].handle_message(WindowMessage::Unfocus);
          let current_focus_index = if key_press.key == 'j' {
              if old_focus_index + 1 == self.components.len() {
                0
              } else {
                old_focus_index + 1
              }
          } else {
            if old_focus_index == 0 {
              self.components.len() - 1
            } else {
              old_focus_index - 1
            }
          };
          self.current_focus = self.components[current_focus_index].name().to_string();
          self.components[current_focus_index].handle_message(WindowMessage::Focus);
          WindowMessageResponse::JustRedraw
        } else if key_press.key == '𐘂' { //the enter key
          let focus_index = self.get_focus_index();
          if let Some(focus_index) = focus_index {
            let r = self.components[focus_index].handle_message(WindowMessage::FocusClick);
            self.handle_start_menu_message(r)
          } else {
            WindowMessageResponse::DoNothing
          }
        } else {
          let current_focus_index = self.get_focus_index().unwrap();
          if key_press.key.is_lowercase() {
            //look forwards to see category/window that starts with that char
            if let Some(n_index) = self.components[current_focus_index..].iter().position(|c| c.text.chars().next().unwrap_or('𐘂').to_lowercase().next().unwrap() == key_press.key) {
              //now old focus, not current focus
              self.components[current_focus_index].handle_message(WindowMessage::Unfocus);
              self.current_focus = self.components[current_focus_index + n_index].name().to_string();
              self.components[current_focus_index + n_index].handle_message(WindowMessage::Focus);
              WindowMessageResponse::JustRedraw
            } else {
              WindowMessageResponse::DoNothing
            }
          } else {
            //look backwards to see category/window that starts with that char
            if let Some(n_index) = self.components[..current_focus_index].iter().rev().position(|c| c.text.chars().next().unwrap_or('𐘂').to_uppercase().next().unwrap() == key_press.key) {
              //now old focus, not current focus
              self.components[current_focus_index].handle_message(WindowMessage::Unfocus);
              self.current_focus = self.components[current_focus_index - n_index - 1].name().to_string();
              self.components[current_focus_index - n_index - 1].handle_message(WindowMessage::Focus);
              WindowMessageResponse::JustRedraw
            } else {
              WindowMessageResponse::DoNothing
            }
          }
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      //top thin border
      DrawInstructions::Rect([0, 0], [self.dimensions[0], 1], theme_info.border_left_top),
      //right thin border
      DrawInstructions::Rect([self.dimensions[0] - 1, 0], [1, self.dimensions[1]], theme_info.border_right_bottom),
      //background
      DrawInstructions::Rect([0, 1], [self.dimensions[0] - 1, self.dimensions[1] - 1], theme_info.background),
      //mingde logo
      DrawInstructions::Bmp([2, 2], exe_dir(Some("bmps/mingde.bmp")).to_string_lossy().to_string(), false),
      //I truly don't know why, it should be - 44 but - 30 seems to work better :shrug:
      DrawInstructions::Gradient([2, 42], [40, self.dimensions[1] - 30], [255, 201, 14], [225, 219, 77], 15),
    ];
    for component in &self.components {
      instructions.extend(component.draw(theme_info));
    }
    instructions
  }
  
  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::StartMenu
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [175, 250]
  }
}

impl StartMenu {
  pub fn new() -> Self {
    Default::default()
  }

  fn handle_start_menu_message(&mut self, message: Option<StartMenuMessage>) -> WindowMessageResponse {
    if let Some(message) = message {
      match message {
        StartMenuMessage::CategoryClick(name) => {
          if name == "Logout" {
            WindowMessageResponse::Request(WindowManagerRequest::Lock)
          } else if name == "About" || name == "Help" {
            //todo above: also do the same for Help
            WindowMessageResponse::Request(WindowManagerRequest::OpenWindow(name.to_string()))
          } else {
            self.current_focus = "Back".to_string();
            self.components = vec![
              Box::new(HighlightButton::new(
                "Back".to_string(), [42, 1], [self.dimensions[0] - 42 - 1, self.y_each], "Back".to_string(), StartMenuMessage::Back, StartMenuMessage::ChangeAcknowledge, true
              ))
            ];
            //add window buttons
            if let Some(to_add) = self.executable_windows.get(&("ming".to_string() + name)) {
              for a in 0..to_add.len() {
               self.components.push(Box::new(HighlightButton::new(
                  to_add[a].1.to_string(), [42, (a + 1) * self.y_each], [self.dimensions[0] - 42 - 1, self.y_each], to_add[a].0.to_string(), StartMenuMessage::WindowClick(to_add[a].1.clone()), StartMenuMessage::ChangeAcknowledge, false
                )));
              }
            }
            WindowMessageResponse::JustRedraw
          }
        },
        StartMenuMessage::WindowClick(name) => {
          //open the selected window
          WindowMessageResponse::Request(WindowManagerRequest::OpenWindow(name.to_string()))
        },
        StartMenuMessage::Back => {
          self.add_category_components();
          WindowMessageResponse::JustRedraw
        },
        StartMenuMessage::ChangeAcknowledge => {
          //
          WindowMessageResponse::JustRedraw
        },
      }
    } else {
      //maybe should be JustRedraw?
      WindowMessageResponse::DoNothing
    }
  }

  pub fn add_category_components(&mut self) {
    self.current_focus = "About".to_string();
    self.components = Vec::new();
    for c in 0..CATEGORIES.len() {
      let name = CATEGORIES[c];
      self.components.push(Box::new(HighlightButton::new(
        name.to_string(), [42, self.y_each * c + 1], [self.dimensions[0] - 42 - 1, self.y_each], name.to_string(), StartMenuMessage::CategoryClick(name), StartMenuMessage::ChangeAcknowledge, c == 0
      )));
    }
  }

  pub fn get_focus_index(&self) -> Option<usize> {
    self.components.iter().filter(|c| c.focusable()).position(|c| c.name() == &self.current_focus)
  }
}


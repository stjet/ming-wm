use std::vec;
use std::vec::Vec;
use std::boxed::Box;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::highlight_button::HighlightButton;

static CATEGORIES: [&'static str; 9] = ["About", "Utils", "Games", "Editing", "Files", "System", "Misc", "Help", "Logout"];

#[derive(Clone)]
enum StartMenuMessage {
  CategoryClick(&'static str),
  WindowClick(&'static str),
  Back,
  ChangeAcknowledge,
}

pub struct StartMenu {
  dimensions: Dimensions,
  components: Vec<Box<dyn Component<StartMenuMessage> + Send>>,
  current_focus: String,
  old_focus: String,
  y_each: usize,
}

impl WindowLike for StartMenu {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.y_each = (self.dimensions[1] - 1) / CATEGORIES.len();
        self.add_category_components();
        WindowMessageResponse::JustRerender
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
          self.old_focus = self.current_focus.to_string();
          self.current_focus = self.components[current_focus_index].name().to_string();
          self.components[current_focus_index].handle_message(WindowMessage::Focus);
          WindowMessageResponse::JustRerender
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
          if let Some(n_index) = self.components[current_focus_index..].iter().position(|c| c.name().chars().next().unwrap_or('𐘂').to_lowercase().next().unwrap() == key_press.key) {
            //now old focus, not current focus
            self.components[current_focus_index].handle_message(WindowMessage::Unfocus);
            self.old_focus = self.current_focus.clone();
            self.current_focus = self.components[current_focus_index + n_index].name().to_string();
            self.components[current_focus_index + n_index].handle_message(WindowMessage::Focus);
            WindowMessageResponse::JustRerender
          } else {
            WindowMessageResponse::DoNothing
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
      DrawInstructions::Mingde([2, 2]),
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
    Self {
      dimensions: [0, 0],
      components: Vec::new(),
      current_focus: String::new(), //placeholder, will be set in init
      old_focus: String::new(),
      y_each: 0, //will be set in add_category_components
    }
  }

  fn handle_start_menu_message(&mut self, message: Option<StartMenuMessage>) -> WindowMessageResponse {
    if let Some(message) = message {
      match message {
        StartMenuMessage::CategoryClick(name) => {
          if name == "Logout" {
            WindowMessageResponse::Request(WindowManagerRequest::Lock)
          } else {
            self.current_focus = "Back".to_string();
            self.components = vec![
              Box::new(HighlightButton::new(
                "Back".to_string(), [42, 1], [self.dimensions[0] - 42 - 1, self.y_each], "Back", StartMenuMessage::Back, StartMenuMessage::ChangeAcknowledge, true
              ))
            ];
            //add window buttons
            let mut to_add: Vec<&str> = Vec::new();
            if name == "Games" {
              to_add.push("Minesweeper");
            } else if name == "Editing" {
              to_add.push("Malvim");
            } else if name == "Files" {
              to_add.push("Terminal");
            }
            //
            for a in 0..to_add.len() {
              let w_name = to_add[a];
              self.components.push(Box::new(HighlightButton::new(
                w_name.to_string(), [42, (a + 1) * self.y_each], [self.dimensions[0] - 42 - 1, self.y_each], w_name, StartMenuMessage::WindowClick(w_name), StartMenuMessage::ChangeAcknowledge, false
              )));
            }
            WindowMessageResponse::JustRerender
          }
        },
        StartMenuMessage::WindowClick(name) => {
          //open the selected window
          WindowMessageResponse::Request(WindowManagerRequest::OpenWindow(name))
        },
        StartMenuMessage::Back => {
          self.add_category_components();
          WindowMessageResponse::JustRerender
        },
        StartMenuMessage::ChangeAcknowledge => {
          //
          WindowMessageResponse::JustRerender
        },
      }
    } else {
      //maybe should be JustRerender?
      WindowMessageResponse::DoNothing
    }
  }

  pub fn add_category_components(&mut self) {
    self.current_focus = "About".to_string();
    self.components = Vec::new();
    for c in 0..CATEGORIES.len() {
      let name = CATEGORIES[c];
      self.components.push(Box::new(HighlightButton::new(
        name.to_string(), [42, self.y_each * c + 1], [self.dimensions[0] - 42 - 1, self.y_each], name, StartMenuMessage::CategoryClick(name), StartMenuMessage::ChangeAcknowledge, c == 0
      )));
    }
  }

  pub fn get_focus_index(&self) -> Option<usize> {
    self.components.iter().filter(|c| c.focusable()).position(|c| c.name() == &self.current_focus)
  }
}


use std::vec;
use std::vec::Vec;
use std::collections::HashMap;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, KeyChar };
use crate::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::press_button::PressButton;
use crate::utils::point_inside;

//seems like framebuffer only updates if (real) key is pressed...
//on mobile, volume down button seems to work but is annoying

const PADDING_Y: usize = 15;
const PADDING_X: usize = 15;
//padding in between keys in the x direction
const KEY_PADDING_X: usize = 5;
const KEY_PADDING_Y: usize = 5;

#[derive(Clone, Default, Eq, PartialEq, Hash)]
enum Board {
  #[default]
  Regular,
  Shift,
  Symbols,
  SymbolsShift,
}

impl Board {
  fn inc(&mut self) -> Self {
    match self {
      Board::Regular => Board::Shift,
      Board::Shift => Board::Symbols,
      Board::Symbols => Board::SymbolsShift,
      Board::SymbolsShift => Board::Regular,
    }
  }
}

#[derive(Clone)]
enum KeyResponse {
  Key(char),
  Alt,
  Ctrl,
  SwitchBoard,
}

//if alt is true and ctrl is true, only alt will be sent.
//because I don't care about ctrl+alt stuff, and won't use it.
//(and probably not supported by this with a real keyboard anyways)
#[derive(Default)]
pub struct OnscreenKeyboard {
  dimensions: Dimensions,
  components: Vec<Box<PressButton<KeyResponse>>>,
  alt: bool,
  ctrl: bool,
  board: Board,
}

impl WindowLike for OnscreenKeyboard {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.set_key_components();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::Touch(x, y) => {
        for c in &mut self.components {
          if point_inside([x, y], c.top_left, c.size) {
            let returned = c.handle_message(WindowMessage::Touch(x, y));
            if let Some(returned) = returned {
              return match returned {
                KeyResponse::Key(ch) => {
                  let kc = if self.alt {
                    self.alt = false;
                    KeyChar::Alt(ch)
                  } else if self.ctrl {
                    self.ctrl = false;
                    KeyChar::Ctrl(ch)
                  } else {
                    KeyChar::Press(ch)
                  };
                  WindowMessageResponse::Request(WindowManagerRequest::DoKeyChar(kc))
                },
                KeyResponse::Alt => {
                  self.alt = !self.alt;
                  WindowMessageResponse::DoNothing
                },
                KeyResponse::Ctrl => {
                  self.ctrl = !self.ctrl;
                  WindowMessageResponse::DoNothing
                },
                KeyResponse::SwitchBoard => {
                  self.board = self.board.inc();
                  WindowMessageResponse::DoNothing
                },
              };
            }
          }
        }
        WindowMessageResponse::DoNothing
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![DrawInstructions::Rect([0, 0], self.dimensions, theme_info.background)];
    for component in &self.components {
      instructions.extend(component.draw(theme_info));
    }
    instructions
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::OnscreenKeyboard
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    [dimensions[0] - 175, 250]
  }
}

impl OnscreenKeyboard {
  pub fn new() -> Self {
    Default::default()
  }

  fn set_key_components(&mut self) {
    self.components = Vec::new();
    let rows: [HashMap<Board, Vec<char>>; 4] = [
      HashMap::from([
        (Board::Regular, vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p']),
        (Board::Shift, vec!['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P']),
        (Board::Symbols, vec!['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']),
        (Board::SymbolsShift, vec![]), //empty
      ]),
      HashMap::from([
        (Board::Regular, vec!['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l']),
        (Board::Shift, vec!['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L']),
        (Board::Symbols, vec!['|', '=', '$', '%', '&', '-', '+', '(', ')']),
        (Board::SymbolsShift, vec!['`', '@', '#', '^']),
      ]),
      HashMap::from([
        (Board::Regular, vec!['𐘃', 'z', 'x', 'c', 'v', 'b', 'n', 'm', '𐘁']), //escape and backspace
        (Board::Shift, vec!['𐘃', 'z', 'x', 'c', 'v', 'b', 'n', 'm', '𐘁']), //escape and backspace
        (Board::Symbols, vec!['~', '*', '"', '\'', ':', ';', '!', '?', '𐘁']), //backspace
        (Board::SymbolsShift, vec!['[', ']', '{', '}', '𐘁']), //backspace
      ]),
      HashMap::from([
        (Board::Regular, vec!['𐘧', '𐘎', ' ', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), enter
        (Board::Shift, vec!['𐘧', '𐘎', ' ', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), enter
        (Board::Symbols, vec!['𐘧', '𐘎', ',', '_', ' ', '/', '.', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), enter
        (Board::SymbolsShift, vec!['𐘧', '𐘎', '\\', '<', ' ', '>', '𐘾', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), ctrl (special case, not a real key), enter
        //ctrl = shimazu
      ]),
    ];
    //hardcoded for now
    let mut y = PADDING_Y;
    let key_height = (self.dimensions[1] - PADDING_Y * 2 - KEY_PADDING_Y * (rows.len() - 1)) / rows.len();
    let reg_key_width = (self.dimensions[0] - PADDING_X * 2 - KEY_PADDING_X * (10 - 1)) / 10;
    for row in rows {
      let row_keys = &row[&self.board];
      //centre
      let mut x = PADDING_X + (10 - row_keys.len()) * (reg_key_width + KEY_PADDING_X) / 2;
      for key in row_keys {
        let press_return = if key == &'𐘧' {
          KeyResponse::SwitchBoard
        } else if key == &'𐘎' {
          KeyResponse::Alt
        } else if key == &'𐘾' {
          KeyResponse::Ctrl
        } else {
          KeyResponse::Key(*key)
        };
        let mut text = key.to_string();
        if text == "𐘧" {
          text = "Switch".to_string();
        } else if text == "𐘎" {
          text = "Alt".to_string();
        } else if text == "𐘂" {
          text = "Enter".to_string();
        } else if text == "𐘁" {
          text = "Back".to_string();
        } else if text == "𐘃" {
          text = "Esc".to_string();
        } else if text == "𐘾" {
          text = "Ctrl".to_string();
        }
        self.components.push(Box::new(PressButton::new([x, y], [reg_key_width, key_height], text, press_return)));
        x += reg_key_width + KEY_PADDING_X;
      }
      y += key_height + KEY_PADDING_Y;
    }
  }
}


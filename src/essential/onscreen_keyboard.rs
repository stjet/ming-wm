use std::vec;
use std::vec::Vec;
use std::collections::HashMap;

use crate::window_manager::{ DrawInstructions, WindowLike, WindowLikeType, KeyChar };
use crate::messages::{ WindowMessage, WindowMessageResponse };
use crate::framebuffer::Dimensions;
use crate::themes::ThemeInfo;
use crate::components::Component;
use crate::components::press_button::PressButton;

const padding_y: usize = 15;
const padding_x: usize = 15;
//padding in between keys in the x direction
const key_padding_x: usize = 5;
const key_padding_y: usize = 5;

#[derive(Default, Eq, PartialEq, Hash)]
enum Board {
  #[default]
  Regular,
  Shift,
  Symbols,
  SymbolsShift,
}

#[derive(Clone)]
enum KeyResponse {
  Key(char),
  Alt,
  Ctrl,
  SwitchBoard,
}

#[derive(Default)]
pub struct OnscreenKeyboard {
  dimensions: Dimensions,
  components: Vec<Box<PressButton<KeyResponse>>>,
  alt: bool,
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
      //
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
        (Board::Symbols, vec!['~', '*', '"', '\'', ':', ';', '!', '?', '𐘁']), //basckspace
        (Board::SymbolsShift, vec!['[', ']', '{', '}', '𐘁']), //backspace
      ]),
      HashMap::from([
        (Board::Regular, vec!['𐘧', '𐘎', ' ', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), enter
        (Board::Shift, vec!['𐘧', '𐘎', ' ', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), enter
        (Board::Symbols, vec!['𐘧', '𐘎', ',', '_', ' ', '/', '.', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), enter
        (Board::SymbolsShift, vec!['𐘧', '𐘎', '\\', '<', ' ', '>', '𐘾', '𐘂']), //switch board (special case, not a real key), alt (special case, not a real key), ctrl (special case, not a real key), enter
      ]),
    ];
    //hardcoded for now
    let mut y = padding_y;
    let key_height = (self.dimensions[1] - padding_y * 2 - key_padding_y * (rows.len() - 1)) / rows.len();
    let reg_key_width = (self.dimensions[0] - padding_x * 2 - key_padding_x * (10 - 1)) / 10;
    for row in rows {
      let row_keys = &row[&self.board];
      //centre
      let mut x = padding_x + (10 - row_keys.len()) * (reg_key_width + key_padding_x) / 2;
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
        self.components.push(Box::new(PressButton::new([x, y], [reg_key_width, key_height], key.to_string(), press_return)));
        x += reg_key_width + key_padding_x;
      }
      y += key_height + key_padding_y;
    }
  }
}


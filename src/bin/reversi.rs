use std::vec::Vec;
use std::vec;
use std::fmt;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::{ Dimensions, RGBColor };
use ming_wm::themes::ThemeInfo;
use ming_wm::ipc::listen;

#[derive(Default, PartialEq)]
enum Tile {
  #[default]
  Empty,
  White,
  Black,
}

impl Tile {
  pub fn to_color(&self) -> Option<RGBColor> {
    match self {
      Tile::Empty => None,
      Tile::White => Some([255, 255, 255]),
      Tile::Black => Some([0, 0, 0]),
    }
  }
}

#[derive(Default)]
struct Reversi {
  dimensions: Dimensions,
  tiles: [[Tile; 8]; 8],
  //
}

impl WindowLike for Reversi {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.new_tiles();
        WindowMessageResponse::JustRerender
      },
      //
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      DrawInstructions::Rect([0, 0], self.dimensions, [72, 93, 63]),
    ];
    let square_width = (self.dimensions[0] - 10) / 8;
    for l in 0..9 {
      instructions.extend([
        DrawInstructions::Rect([5 + square_width * l, 5], [2, self.dimensions[1] - 10], [0, 0, 0]),
        DrawInstructions::Rect([5, 5 + square_width * l], [self.dimensions[0] - 10, 2], [0, 0, 0]),
      ]);
    }
    instructions.extend([
      DrawInstructions::Circle([5 + square_width * 2, 5 + square_width * 2], 4, [0, 0, 0]),
      DrawInstructions::Circle([5 + square_width * 6, 5 + square_width * 2], 4, [0, 0, 0]),
      DrawInstructions::Circle([5 + square_width * 2, 5 + square_width * 6], 4, [0, 0, 0]),
      DrawInstructions::Circle([5 + square_width * 6, 5 + square_width * 6], 4, [0, 0, 0]),
    ]);
    for y in 0..8 {
      for x in 0..8 {
        let tile = &self.tiles[y][x];
        if tile == &Tile::Empty {
          //
        } else {
          instructions.push(DrawInstructions::Circle([x * square_width + square_width / 2 + 5, y * square_width + square_width / 2 + 5], square_width / 2 - 3, tile.to_color().unwrap()));
        }
      }
    }
    instructions
  }

  //properties

  fn title(&self) -> String {
    "Reversi".to_string()
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [300, 300]
  }
}

impl Reversi {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn new_tiles(&mut self) {
    self.tiles = Default::default();
    self.tiles[3][3] = Tile::White;
    self.tiles[4][3] = Tile::Black;
    self.tiles[3][4] = Tile::Black;
    self.tiles[4][4] = Tile::White;
  }
}

pub fn main() {
  listen(Reversi::new());
}


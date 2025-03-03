use std::vec::Vec;
use std::vec;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm_lib::framebuffer_types::{ Dimensions, RGBColor };
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::ipc::listen;

const REVERSI_GREEN: RGBColor = [72, 93, 63];

struct ValidMove {
  pub point: [usize; 2],
  pub will_flip: Vec<[usize; 2]>,
}

//tried to do some PartialEq implementation but didn't play nice with .contains
//so just do this instead
fn valid_moves_contains(valid_moves: &Vec<ValidMove>, point: &[usize; 2]) -> Option<Vec<[usize; 2]>> {
  for valid_move in valid_moves {
    if &valid_move.point == point {
      return Some(valid_move.will_flip.clone());
    }
  }
  None
}

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

#[derive(Default, PartialEq)]
enum State {
  #[default]
  InProgress,
  WhiteWin,
  BlackWin,
  Tie,
}

#[derive(Default)]
struct Reversi {
  dimensions: Dimensions,
  tiles: [[Tile; 8]; 8],
  current_number: Option<u8>, //the first number of the tile that user wants to place piece on
  valid_moves: Vec<ValidMove>,
  white_turn: bool, //if false, black turn
  state: State,
}

impl WindowLike for Reversi {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        self.new_tiles();
        self.valid_moves = self.get_valid_moves();
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        if self.state != State::InProgress {
          self.state = State::InProgress;
          self.new_tiles();
          self.valid_moves = self.get_valid_moves();
        } else if let Ok(n) = key_press.key.to_string().parse::<u8>() {
          if let Some(current_number) = self.current_number {
            let y = current_number as usize;
            let x = n as usize;
            if let Some(mut will_flip) = valid_moves_contains(&self.valid_moves, &[x, y]) {
              self.tiles[y][x] = if self.white_turn { Tile::White } else { Tile::Black };
              will_flip.push([x, y]);
              for point in will_flip {
                self.tiles[point[1]][point[0]] = if self.white_turn { Tile::White } else { Tile::Black };
              }
              self.white_turn = !self.white_turn;
              self.valid_moves = self.get_valid_moves();
              if self.valid_moves.len() == 0 {
                //game has ended
                let mut white_tiles = 0;
                let mut black_tiles = 0;
                for row in &self.tiles {
                  for tile in row {
                    if tile == &Tile::White {
                      white_tiles += 1;
                    } else if tile == &Tile::Black {
                      black_tiles += 1;
                    }
                  }
                }
                if white_tiles == black_tiles {
                  self.state = State::Tie;
                } else if white_tiles > black_tiles {
                  self.state = State::WhiteWin;
                } else {
                  self.state = State::BlackWin;
                }
              }
            }
            self.current_number = None;
          } else {
            self.current_number = Some(n);
          }
        } else if key_press.key == '𐘁' { //backspace
          self.current_number = None;
        }
        WindowMessageResponse::JustRedraw
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = vec![
      DrawInstructions::Rect([0, 0], self.dimensions, REVERSI_GREEN),
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
          instructions.push(DrawInstructions::Text([x * square_width + square_width / 2, y * square_width + square_width / 2], vec!["nimbus-roman".to_string()], format!("{}{}", y, x), theme_info.text, REVERSI_GREEN, None, None));
          if valid_moves_contains(&self.valid_moves, &[x, y]).is_some() {
            //yellow border
            instructions.extend([
              DrawInstructions::Rect([5 + x * square_width, 5 + y * square_width], [square_width + 2, 2], [255, 255, 0]),
              DrawInstructions::Rect([5 + x * square_width, 5 + y * square_width], [2, square_width], [255, 255, 0]),
              DrawInstructions::Rect([5 + (x + 1) * square_width, 5 + y * square_width], [2, square_width], [255, 255, 0]),
              DrawInstructions::Rect([5 + x * square_width + 2, 5 + (y + 1) * square_width], [square_width + 2, 2], [255, 255, 0]),
            ]);
          }
        } else {
          instructions.push(DrawInstructions::Circle([x * square_width + square_width / 2 + 5, y * square_width + square_width / 2 + 5], square_width / 2 - 3, tile.to_color().unwrap()));
        }
      }
    }
    if self.state != State::InProgress {
      instructions.push(DrawInstructions::Rect([0, 0], [self.dimensions[0], 25], theme_info.background));
      instructions.push(DrawInstructions::Text([4, 4], vec!["nimbus-roman".to_string()], if self.state == State::WhiteWin {
        "White wins, press any key to restart"
      } else if self.state == State::BlackWin {
        "Black wins, press any key to restart"
      } else {
        "Tie, press any key to restart"
      }.to_string(), theme_info.text, theme_info.background, None, None));
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

  pub fn get_valid_moves(&self) -> Vec<ValidMove> {
    let mut valid_moves = Vec::new();
    for y0 in 0..8 {
      for x0 in 0..8 {
        let current_tile = &self.tiles[y0][x0];
        if (current_tile == &Tile::White && self.white_turn) || (current_tile == &Tile::Black && !self.white_turn) {
          for t in 0..8 {
            let mut potential_move = false; //true once opposite colour tile found
            let mut point = [x0, y0];
            let mut will_flip = Vec::new();
            loop {
              let x = point[0];
              let y = point[1];
              if t == 0 {
                //up left
                if y > 0 && x > 0 {
                  point = [x - 1, y - 1];
                } else {
                  break;
                }
              } else if t == 1 {
                //up
                if y > 0 {
                  point = [x, y - 1];
                } else {
                  break;
                }
              } else if t == 2 {
                //up right
                if y > 0 && x < 7 {
                  point = [x, y - 1];
                } else {
                  break;
                }
              } else if t == 3 {
                //left
                if x > 0 {
                  point = [x - 1, y];
                } else {
                  break;
                }
              } else if t == 4 {
                //right
                if x < 7 {
                  point = [x + 1, y];
                } else {
                  break;
                }
              } else if t == 5 {
                //down left
                if y < 7 && x > 0 {
                  point = [x - 1, y + 1];
                } else {
                  break;
                }
              } else if t == 6 {
                //down
                if y < 7 {
                  point = [x, y + 1];
                } else {
                  break;
                }
              } else if t == 7 {
                //down right
                if y < 7 && x < 7 {
                  point = [x + 1, y + 1];
                } else {
                  break;
                }
              }
              let tile = &self.tiles[point[1]][point[0]];
              if tile == &Tile::Empty && potential_move {
                valid_moves.push(ValidMove {
                  point,
                  will_flip,
                });
                break;
              } else if (tile == &Tile::Black && self.white_turn) || (tile == &Tile::White && !self.white_turn) {
                will_flip.push(point);
                potential_move = true;
              } else {
                break;
              }
            }
          }
        }
      }
    }
    valid_moves
  }
}

pub fn main() {
  listen(Reversi::new());
}


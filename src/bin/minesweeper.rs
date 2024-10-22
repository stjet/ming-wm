use std::vec::Vec;
use std::vec;
use std::collections::VecDeque;
use core::convert::TryFrom;

use ming_wm::window_manager::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm::framebuffer::Dimensions;
use ming_wm::themes::ThemeInfo;
use ming_wm::ipc::listen;

const HEX_CHARS: [char; 16] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

fn u8_to_hex(u: u8) -> String {
  let mut h = String::new();
  h.push(HEX_CHARS[(u / 16) as usize]);
  h.push(HEX_CHARS[(u % 16) as usize]);
  h
}

fn hex_to_u8(c1: char, c2: char) -> u8 {
  (HEX_CHARS.iter().position(|c| c == &c1).unwrap() * 16 + HEX_CHARS.iter().position(|c| c == &c2).unwrap()) as u8
}

//16x16 with 40 mines

#[derive(Default)]
struct MineTile {
  mine: bool,
  revealed: bool,
  touching: u8,
}

#[derive(Default, PartialEq)]
enum State {
  #[default]
  Seed,
  BeforePlaying,
  Playing,
  Won,
  Lost,
}

#[derive(Default)]
pub struct Minesweeper {
  dimensions: Dimensions,
  state: State,
  tiles: [[MineTile; 16]; 16],
  random_chars: String,
  random_seed: u32, //user types in random keyboard stuff at beginning
  first_char: char, //defaults to '\0'
}

impl WindowLike for Minesweeper {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRerender
      },
      WindowMessage::KeyPress(key_press) => {
        if self.state == State::Seed {
          if self.random_chars.len() == 4 {
            let mut r_chars = self.random_chars.chars();
            self.random_seed = ((r_chars.next().unwrap() as u8 as u32) << 24) | ((r_chars.next().unwrap() as u8 as u32) << 16) | ((r_chars.next().unwrap() as u8 as u32) << 8) | (r_chars.next().unwrap() as u8 as u32);
            self.random_chars = String::new();
            self.state = State::BeforePlaying;
          } else {
            if u8::try_from(key_press.key).is_ok() {
              self.random_chars.push(key_press.key);
            }
          }
          WindowMessageResponse::JustRerender
        } else if self.state == State::BeforePlaying || self.state == State::Playing {
          if key_press.key == '𐘁' { //backspace
            self.first_char = '\0';
            WindowMessageResponse::DoNothing
          } else if self.first_char == '\0' {
            if HEX_CHARS.iter().find(|c| c == &&key_press.key).is_some() {
              self.first_char = key_press.key;
            }
            WindowMessageResponse::DoNothing
          } else if HEX_CHARS.iter().find(|c| c == &&key_press.key).is_some() {
            let u = hex_to_u8(self.first_char, key_press.key) as usize;
            let y = u / 16;
            let x = u % 16;
            if HEX_CHARS.iter().find(|c| c == &&key_press.key).is_some() {
              if self.state == State::BeforePlaying {
                loop {
                  self.new_tiles();
                  if self.tiles[y][x].touching == 0 && !self.tiles[y][x].mine {
                    break;
                  }
                }
              }
              self.state = State::Playing;
              //if that tile not reveal it, reveal it and all adjacent zero touching squares, etc
              if self.tiles[y][x].mine {
                self.tiles[y][x].revealed = true;
                self.state = State::Lost;
              } else if self.tiles[y][x].touching == 0 {
                let mut queue = VecDeque::new();
                queue.push_back([x, y]);
                let mut to_reveal = Vec::new();
                while queue.len() > 0 {
                  let current = queue.pop_front().unwrap();
                  self.on_adjacent_tiles(current[0], current[1], |x2, y2| {
                    if !queue.contains(&[x2, y2]) && !to_reveal.contains(&[x2, y2]) {
                      if self.tiles[y2][x2].touching == 0 {
                        queue.push_back([x2, y2]);
                      } else {
                        to_reveal.push([x2, y2]);
                      }
                    }
                  }, false);
                  to_reveal.push(current);
                }
                for r in to_reveal {
                  self.tiles[r[1]][r[0]].revealed = true;
                }
              } else {
                self.tiles[y][x].revealed = true;
              }
              self.first_char = '\0';
              if self.state != State::Lost {
                //check for win
                let mut won = true;
                for y in 0..16 {
                  for x in 0..16 {
                    let tile = &self.tiles[y][x];
                    if !tile.revealed && !tile.mine {
                      won = false;
                    }
                  }
                }
                if won {
                  self.state = State::Won;
                }
              }
              WindowMessageResponse::JustRerender
            } else {
              WindowMessageResponse::DoNothing
            }
          } else {
            WindowMessageResponse::DoNothing
          }
        } else {
          self.tiles = Default::default();
          self.state = State::Seed;
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    if self.state == State::Seed {
      vec![
        DrawInstructions::Text([4, 4], "times-new-roman".to_string(), "Type in random characters to initalise the seed".to_string(), theme_info.text, theme_info.background, None, None),
        DrawInstructions::Text([4, 4 + 16], "times-new-roman".to_string(), self.random_chars.clone(), theme_info.text, theme_info.background, None, None),
      ]
    } else {
      let mut instructions = vec![
        //top border
        DrawInstructions::Rect([1, 0], [self.dimensions[0] - 7, 5], [128, 128, 128]),
        DrawInstructions::Rect([self.dimensions[0] - 6, 0], [4, 1], [128, 128, 128]),
        DrawInstructions::Rect([self.dimensions[0] - 6, 1], [3, 1], [128, 128, 128]),
        DrawInstructions::Rect([self.dimensions[0] - 6, 2], [2, 1], [128, 128, 128]),
        DrawInstructions::Rect([self.dimensions[0] - 6, 3], [1, 1], [128, 128, 128]),
        DrawInstructions::Rect([self.dimensions[0] - 6, 4], [1, 1], [128, 128, 128]),
        //left border
        DrawInstructions::Rect([1, 0], [5, self.dimensions[1] - 5], [128, 128, 128]),
        DrawInstructions::Rect([1, self.dimensions[1] - 5], [1, 4], [128, 128, 128]),
        DrawInstructions::Rect([2, self.dimensions[1] - 5], [1, 3], [128, 128, 128]),
        DrawInstructions::Rect([3, self.dimensions[1] - 5], [1, 2], [128, 128, 128]),
        DrawInstructions::Rect([4, self.dimensions[1] - 5], [1, 1], [128, 128, 128]),
        //bottom border
        DrawInstructions::Rect([6, self.dimensions[1] - 6], [self.dimensions[0] - 2, 5], [255, 255, 255]),
        DrawInstructions::Rect([5, self.dimensions[1] - 5], [1, 4], [255, 255, 255]),
        DrawInstructions::Rect([4, self.dimensions[1] - 4], [1, 3], [255, 255, 255]),
        DrawInstructions::Rect([3, self.dimensions[1] - 3], [1, 2], [255, 255, 255]),
        DrawInstructions::Rect([2, self.dimensions[1] - 2], [1, 1], [255, 255, 255]),
        //right border
        DrawInstructions::Rect([self.dimensions[0] - 6, 5], [5, self.dimensions[1]], [255, 255, 255]),
        DrawInstructions::Rect([self.dimensions[0] - 2, 0], [1, 5], [255, 255, 255]),
        DrawInstructions::Rect([self.dimensions[0] - 3, 1], [1, 4], [255, 255, 255]),
        DrawInstructions::Rect([self.dimensions[0] - 4, 2], [1, 3], [255, 255, 255]),
        DrawInstructions::Rect([self.dimensions[0] - 5, 3], [1, 2], [255, 255, 255]),
        DrawInstructions::Rect([self.dimensions[0] - 6, 4], [1, 1], [255, 255, 255]),
      ];
      let tile_size = (self.dimensions[0] - 10) / 16;
      for y in 0..16 {
        for x in 0..16 {
          let tile = &self.tiles[y][x];
          if tile.revealed {
            if tile.mine {
              instructions.push(DrawInstructions::Text([x * tile_size + tile_size / 2 + 2, y * tile_size + tile_size / 2], "times-new-roman".to_string(), "x".to_string(), [255, 0, 0], theme_info.background, None, None));
            } else {
              let color = match tile.touching {
                1 => [0, 0, 255],
                2 => [0, 255, 0],
                3 => [255, 0, 0],
                4 => [128, 0, 128],
                5 => [176, 48, 96],
                6 => [127, 255, 212],
                7 => [0, 0, 0],
                //8
                _ => [128, 128, 128],
              };
              instructions.push(DrawInstructions::Text([x * tile_size + tile_size / 2 + 5, y * tile_size + tile_size / 2 + 2], "times-new-roman".to_string(), tile.touching.to_string(), color, theme_info.background, None, None));
            }
          } else {
            let top_left = [x * tile_size + 6, y * tile_size + 5];
            //do not do the corners in respect of our poor poor heap (vector size too big would be bad)
            instructions.extend(vec![
              //top border
              DrawInstructions::Rect([top_left[0], top_left[1]], [tile_size - 3, 3], [255, 255, 255]),
              //
              //left border
              DrawInstructions::Rect([top_left[0], top_left[1]], [3, tile_size - 3], [255, 255, 255]),
              //
              //bottom border
              DrawInstructions::Rect([top_left[0] + 3, top_left[1] + tile_size - 4], [tile_size - 4, 3], [128, 128, 128]),
              //
              //right bottom
              DrawInstructions::Rect([top_left[0] + tile_size - 4, top_left[1] + 3], [3, tile_size - 4], [128, 128, 128]),
              //
              DrawInstructions::Text([x * tile_size + tile_size / 2 - 2, y * tile_size + tile_size / 2], "times-new-roman".to_string(), u8_to_hex((y * 16 + x) as u8), theme_info.text, theme_info.background, None, None),
            ]);
          }
        }
      }
      if self.state == State::Lost {
        instructions.extend(vec![DrawInstructions::Text([4, 4], "times-new-roman".to_string(), "You LOST!!! Press a key to play again.".to_string(), theme_info.text, theme_info.background, None, None)]);
      } else if self.state == State::Won {
        instructions.extend(vec![DrawInstructions::Text([4, 4], "times-new-roman".to_string(), "You WON!!! Press a key to play again.".to_string(), theme_info.text, theme_info.background, None, None)]);
      }
      instructions
    }
  }

  //properties

  fn title(&self) -> String {
    "Minesweeper".to_string()
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [410, 410]
  }
}

impl Minesweeper {
  pub fn new() -> Self {
    Default::default()
  }

  //https://en.wikipedia.org/wiki/Xorshift
  //from 0 to 15
  pub fn random(&mut self) -> usize {
    let mut x = self.random_seed;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.random_seed = x;
    self.random_seed as usize % 16
  }

  pub fn on_adjacent_tiles(&self, x: usize, y: usize, mut action: impl FnMut(usize, usize) -> (), if_mine: bool) {
    if y > 0 {
      //above
      if self.tiles[y - 1][x].mine == if_mine {
        action(x, y - 1);
      }
      if x > 0 {
        //above to the left
        if self.tiles[y - 1][x - 1].mine == if_mine {
          action(x - 1, y - 1);
        }
      }
      if x < 15 {
        //above to the right
        if self.tiles[y - 1][x + 1].mine == if_mine {
          action(x + 1, y - 1);
        }
      }
    }
    if x > 0 {
      //to the left
      if self.tiles[y][x - 1].mine == if_mine {
        action(x - 1, y);
      }
    }
    if x < 15 {
      //to the right
      if self.tiles[y][x + 1].mine == if_mine {
        action(x + 1, y);
      }
    }
    if y < 15 {
      //below
      if self.tiles[y + 1][x].mine == if_mine {
        action(x, y + 1);
      }
      if x > 0 {
        //below to the left
        if self.tiles[y + 1][x - 1].mine == if_mine {
          action(x - 1, y + 1);
        }
      }
      if x < 15 {
        //below to the right
        if self.tiles[y + 1][x + 1].mine == if_mine {
          action(x + 1, y + 1);
        }
      }
    }
  }

  pub fn new_tiles(&mut self) {
    self.tiles = Default::default();
    //40 mines
    for _ in 0..40 {
      loop {
        let x = self.random();
        let y = self.random();
        //
        if !self.tiles[y][x].mine {
          self.tiles[y][x].mine = true;
          break;
        }
      }
    }
    //calculate touching
    for y in 0..16 {
      for x in 0..16 {
        let mut touching = 0;
        self.on_adjacent_tiles(x, y, |_, _| {
          touching += 1;
        }, true);
        self.tiles[y][x].touching = touching;
      }
    }
  }
  //
}

pub fn main() {
  listen(Minesweeper::new());
}


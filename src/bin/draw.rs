use std::vec::Vec;
use std::vec;

use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse };
use ming_wm_lib::framebuffer_types::{ Dimensions, Point, RGBColor };
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::utils::{ hex_to_u8, HEX_CHARS, Substring };
use ming_wm_lib::ipc::listen;

enum DrawAction {
  Line(Point, Option<Point>, usize, RGBColor),
  Rect(Point, Option<Dimensions>, RGBColor),
  Circle(Point, Option<usize>, RGBColor),
}

impl DrawAction {
  fn name(&self) -> String {
    match self {
      DrawAction::Line(_, _, _, _) => "Line",
      DrawAction::Rect(_, _, _) => "Rect",
      DrawAction::Circle(_, _, _) => "Circle",
    }.to_string()
  }
}

#[derive(Default, PartialEq)]
enum Mode {
  #[default]
  Move,
  Input,
}

#[derive(Default)]
struct Draw {
  mode: Mode,
  dimensions: Dimensions,
  draw_actions: Vec<DrawAction>,
  current_location: Point,
  current_input: String,
  current_color: RGBColor,
  current_linewidth: usize,
  current_action: Option<DrawAction>,
}

impl WindowLike for Draw {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::ChangeDimensions(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.is_escape() && (self.current_action.is_some() || self.mode != Mode::Move) {
          self.current_action = None;
          self.mode = Mode::Move;
          self.current_input = String::new();
          WindowMessageResponse::JustRedraw
        } else if self.mode == Mode::Input {
          if key_press.is_backspace() && self.current_input.len() > 0 {
            self.current_input = self.current_input.remove_last();
            WindowMessageResponse::JustRedraw
          } else if key_press.is_enter() {
            //process current input
            let mut parts = self.current_input.split(" ");
            match parts.next().unwrap() {
              "line" | "l" => {
                self.current_action = Some(DrawAction::Line(self.current_location, None, self.current_linewidth, self.current_color));
              },
              "rect" | "r" => {
                self.current_action = Some(DrawAction::Rect(self.current_location, None, self.current_color));
              },
              "circle" | "c" => {
                self.current_action = Some(DrawAction::Circle(self.current_location, None, self.current_color));
              },
              "colour" | "color" | "co" => {
                //hex to u8
                if let Some(hex_color) = parts.next() {
                  if hex_color.len() == 6 && hex_color.chars().all(|c| HEX_CHARS.contains(&c)) {
                    let mut hex_chars = hex_color.chars();
                    self.current_color = [hex_to_u8(hex_chars.next().unwrap(), hex_chars.next().unwrap()), hex_to_u8(hex_chars.next().unwrap(), hex_chars.next().unwrap()), hex_to_u8(hex_chars.next().unwrap(), hex_chars.next().unwrap())];
                  }
                }
              },
              "linewidth" | "lw" => {
                if let Ok(linewidth) = parts.next().unwrap_or("").parse::<usize>() {
                  self.current_linewidth = linewidth;
                }
              },
              "undo" | "u" => {
                self.draw_actions.pop();
              },
              "clear" | "cl" => {
                self.draw_actions = Vec::new();
              },
              _ => {},
            };
            self.mode = Mode::Move;
            self.current_input = String::new();
            WindowMessageResponse::JustRedraw
          } else if key_press.is_regular() {
            self.current_input += &key_press.key.to_string();
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else if key_press.key == 'i' && self.current_action.is_none() {
          self.mode = Mode::Input;
          WindowMessageResponse::JustRedraw
        } else if key_press.is_enter() {
          if let Some(current_action) = &self.current_action {
            self.draw_actions.push(match current_action {
              DrawAction::Line(p, _, u, r) => DrawAction::Line(*p, Some(self.current_location), *u, *r),
              DrawAction::Rect(p, _, r) => {
                let d = [
                  if self.current_location[0] > p[0] {
                    self.current_location[0] - p[0]
                  } else {
                    p[0] - self.current_location[0]
                  },
                  if self.current_location[1] > p[1] {
                    self.current_location[1] - p[1]
                  } else {
                    p[1] - self.current_location[1]
                  }
                ];
                //find top left corner
                let tl = [
                  if p[0] < self.current_location[0] {
                    p[0]
                  } else {
                    self.current_location[0]
                  },
                  if p[1] < self.current_location[1] {
                    p[1]
                  } else {
                    self.current_location[1]
                  }
                ];
                DrawAction::Rect(tl, Some(d), *r)
              },
              DrawAction::Circle(p, _, c) => {
                let r = ((self.current_location[1] as f64 - p[1] as f64).powi(2) + (self.current_location[0] as f64 - p[0] as f64).powi(2)).sqrt();
                DrawAction::Circle(*p, Some(r.round() as usize), *c)
              },
            });
            self.current_action = None;
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else if key_press.is_up_arrow() || key_press.key == 'k' {
          if self.current_location[1] > 0 {
            self.current_location[1] -= 1;
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else if key_press.is_down_arrow() || key_press.key == 'j' {
          if self.current_location[1] + 1 < self.dimensions[1] {
            self.current_location[1] += 1;
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else if key_press.is_left_arrow() || key_press.key == 'h' {
          if self.current_location[0] > 0 {
            self.current_location[0] -= 1;
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else if key_press.is_right_arrow() || key_press.key == 'l' {
          if self.current_location[0] + 1 < self.dimensions[0] {
            self.current_location[0] += 1;
            WindowMessageResponse::JustRedraw
          } else {
            WindowMessageResponse::DoNothing
          }
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    let mut instructions = Vec::new();
    //draw previous actions
    for action in &self.draw_actions {
      instructions.push(match action {
        DrawAction::Line(p1, p2, lw, c) => DrawInstructions::Line(*p1, p2.unwrap(), *lw, *c),
        DrawAction::Rect(p, d, c) => DrawInstructions::Rect(*p, d.unwrap(), *c),
        DrawAction::Circle(p, r, c) => DrawInstructions::Circle(*p, r.unwrap(), *c),
      });
    }
    //draw cursor (crosshair)
    let crosshair_min_x = self.current_location[0].checked_sub(6).unwrap_or(0);
    let crosshair_min_y = self.current_location[1].checked_sub(6).unwrap_or(0);
    //^going over should be handled by the drawer, probably?
    instructions.push(DrawInstructions::Line([crosshair_min_x, self.current_location[1]], [self.current_location[0] + 6, self.current_location[1]], 1, self.current_color));
    instructions.push(DrawInstructions::Line([self.current_location[0], crosshair_min_y], [self.current_location[0], self.current_location[1] + 6], 1, self.current_color));
    //draw info or current input
    instructions.push(DrawInstructions::Text([2, self.dimensions[1] - 19], vec!["nimbus-roman".to_string()], if self.current_input == String::new() {
      if let Some(current_action) = &self.current_action {
        current_action.name()
      } else if self.mode == Mode::Move {
        "'i' to enter input mode".to_string()
      } else {
        "Awaiting input".to_string()
      }
    } else {
      self.current_input.clone()
    }, theme_info.text, theme_info.background, None, None));
    instructions
  }

  //properties

  fn title(&self) -> String {
    "Draw".to_string()
  }

  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::Window
  }

  fn ideal_dimensions(&self, _dimensions: Dimensions) -> Dimensions {
    [410, 410]
  }

  fn resizable(&self) -> bool {
    true
  }
}

impl Draw {
  pub fn new() -> Self {
    //apparently this is legal. thanks clippy
    Self {
      current_linewidth: 1,
      ..Default::default() //no comma here allowed though??
    }
  }
}

pub fn main() {
  listen(Draw::new());
}

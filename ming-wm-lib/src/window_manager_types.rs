use crate::framebuffer_types::{ Point, Dimensions, RGBColor };
use crate::themes::ThemeInfo;
use crate::messages::*;

pub const TASKBAR_HEIGHT: usize = 38;
pub const INDICATOR_HEIGHT: usize = 20;

#[derive(Clone, Debug, PartialEq)]
pub enum KeyChar {
  Press(char),
  Alt(char),
  Ctrl(char),
}

#[derive(Debug)]
pub enum DrawInstructions {
  /// Top left point, dimensions, colour
  Rect(Point, Dimensions, RGBColor),
  /// Top left point, fonts, text, colour, background colour, horizontal spacing, monospace width
  Text(Point, Vec<String>, String, RGBColor, RGBColor, Option<usize>, Option<u8>),
  /// Top left point, dimensions, start colour, end colour, steps
  Gradient(Point, Dimensions, RGBColor, RGBColor, usize),
  /// Top left point, path to file, reverse
  Bmp(Point, String, bool),
  /// Centre point, radius, colour
  Circle(Point, usize, RGBColor),
  /// Start point, end point, line width, line colour
  Line(Point, Point, usize, RGBColor),
}

#[derive(Debug, PartialEq)]
pub enum WindowLikeType {
  LockScreen,
  Window,
  DesktopBackground,
  Taskbar,
  StartMenu,
  WorkspaceIndicator,
  OnscreenKeyboard,
}

pub trait WindowLike {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse;

  fn draw(&self, theme_info: &ThemeInfo) -> Vec<DrawInstructions>;

  //properties
  fn title(&self) -> String {
    String::new()
  }

  fn resizable(&self) -> bool {
    false
  }

  fn subtype(&self) -> WindowLikeType;

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions; //needs &self or its not object safe or some bullcrap
}


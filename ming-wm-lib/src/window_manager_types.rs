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
  Rect(Point, Dimensions, RGBColor),
  Text(Point, Vec<String>, String, RGBColor, RGBColor, Option<usize>, Option<u8>), //font and text
  Gradient(Point, Dimensions, RGBColor, RGBColor, usize),
  Bmp(Point, String, bool),
  Circle(Point, usize, RGBColor),
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


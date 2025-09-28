use std::boxed::Box;
use std::vec::Vec;

use crate::framebuffer_types::Dimensions;
use crate::window_manager_types::{ WindowLike, KeyChar };

/// Window manager internal usage
pub enum WindowManagerMessage {
  KeyChar(KeyChar),
  Touch(usize, usize),
  //
}

/// Window manager internal usage
pub type WindowBox = Box<dyn WindowLike>;

/*
impl PartialEq for WindowBox {
  fn eq(&self, _other: &Self) -> bool {
    //lol
    true
  }
}
*/

#[derive(Debug, PartialEq)]
pub enum WindowManagerRequest {
  OpenWindow(String),
  //may not work in \x1E, \x1F or \x1D are in the paste string
  ClipboardCopy(String),
  CloseStartMenu,
  Unlock,
  Lock,
  DoKeyChar(KeyChar),
  //
}

#[derive(PartialEq, Debug)]
pub enum WindowMessageResponse {
  Request(WindowManagerRequest),
  JustRedraw,
  DoNothing,
}

impl WindowMessageResponse {
  pub fn is_key_char_request(&self) -> bool {
    matches!(self, WindowMessageResponse::Request(WindowManagerRequest::DoKeyChar(_)))
  }
}

//struct because may add more fields later (so struct is better for code backward compatibility)
pub struct KeyPress {
  pub key: char,
}

impl KeyPress {
  pub fn is_enter(&self) -> bool {
    self.key == '𐘂'
  }

  pub fn is_backspace(&self) -> bool {
    self.key == '𐘁'
  }

  pub fn is_escape(&self) -> bool {
    self.key == '𐘃'
  }

  pub fn is_up_arrow(&self) -> bool {
    self.key == '𐙘'
  }

  pub fn is_down_arrow(&self) -> bool {
    self.key == '𐘞'
  }

  pub fn is_left_arrow(&self) -> bool {
    self.key == '𐙣'
  }

  pub fn is_right_arrow(&self) -> bool {
    self.key == '𐙥'
  }

  pub fn is_arrow(&self) -> bool {
    self.is_up_arrow() || self.is_down_arrow() || self.is_left_arrow() || self.is_right_arrow()
  }

  /// Is not enter, backspace, arrow keys (the Linear A stuff)
  pub fn is_regular(&self) -> bool {
    !self.is_enter() && !self.is_backspace() && !self.is_escape() && !self.is_arrow()
  }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
  Left,
  Down,
  Up,
  Right,
}

//todo, rename to CommandType
#[derive(PartialEq)]
pub enum ShortcutType {
  StartMenu,
  SwitchWorkspace(u8),
  MoveWindowToWorkspace(u8),
  FocusPrevWindow,
  FocusNextWindow,
  QuitWindow,
  MoveWindow(Direction),
  MoveWindowToEdge(Direction),
  ChangeWindowSize(Direction),
  CenterWindow,
  FullscreenWindow,
  HalfWidthWindow, //half width, full height
  ClipboardCopy,
  //may not work in \x1E, \x1F or \x1D are in the paste string
  ClipboardPaste(String),
  //
}

pub type WindowsVec = Vec<(usize, String)>;

#[non_exhaustive]
pub enum InfoType {
  /// Let taskbar know what the current windows in the workspace are
  WindowsInWorkspace(WindowsVec, usize), //Vec<(id, name)>, focused id
  //
}

pub enum WindowMessage {
  Init(Dimensions),
  KeyPress(KeyPress),
  CtrlKeyPress(KeyPress),
  Shortcut(ShortcutType),
  Info(InfoType),
  Focus,
  Unfocus,
  FocusClick,
  ChangeDimensions(Dimensions),
  /// For onscreen keyboard only
  Touch(usize, usize),
  //
}


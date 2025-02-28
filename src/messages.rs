use std::boxed::Box;
use std::vec::Vec;

use crate::framebuffer::Dimensions;
use crate::window_manager::{ WindowLike, KeyChar };

pub enum WindowManagerMessage {
  KeyChar(KeyChar),
  Touch(usize, usize),
  //
}

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
    if let WindowMessageResponse::Request(WindowManagerRequest::DoKeyChar(_)) = self {
      true
    } else {
      false
    }
  }
}

pub struct KeyPress {
  pub key: char,
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

pub enum InfoType {
  //let taskbar know what the current windows in the workspace are
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
  Touch(usize, usize), //for onscreen keyboard only
  //
}


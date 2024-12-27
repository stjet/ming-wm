use std::boxed::Box;
use std::fmt;
use std::vec::Vec;

use serde::{ Deserialize, Serialize };

use crate::keyboard::KeyChar;
use crate::framebuffer::Dimensions;
use crate::window_manager::WindowLike;

pub enum WindowManagerMessage {
  KeyChar(KeyChar),
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

#[derive(PartialEq, Serialize, Deserialize)]
pub enum WindowManagerRequest {
  OpenWindow(String),
  ClipboardCopy(String),
  CloseStartMenu,
  Unlock,
  Lock,
  //
}

impl fmt::Debug for WindowManagerRequest{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "WindowManagerRequest lmao")
  }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum WindowMessageResponse {
  Request(WindowManagerRequest),
  JustRerender,
  DoNothing,
}

#[derive(Serialize, Deserialize)]
pub struct KeyPress {
  pub key: char,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
  Left,
  Down,
  Up,
  Right,
}

//todo, rename to CommandType
#[derive(PartialEq, Serialize, Deserialize)]
pub enum ShortcutType {
  StartMenu,
  SwitchWorkspace(u8),
  MoveWindowToWorkspace(u8),
  FocusPrevWindow,
  FocusNextWindow,
  QuitWindow,
  MoveWindow(Direction),
  MoveWindowToEdge(Direction),
  CenterWindow,
  FullscreenWindow,
  HalfWidthWindow, //half width, full height
  ClipboardCopy,
  ClipboardPaste(String),
  //
}

pub type WindowsVec = Vec<(usize, String)>;

#[derive(Serialize, Deserialize)]
pub enum InfoType {
  //let taskbar know what the current windows in the workspace are
  WindowsInWorkspace(WindowsVec, usize), //Vec<title, name)>, focused id
  //
}

#[derive(Serialize, Deserialize)]
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
  //
}

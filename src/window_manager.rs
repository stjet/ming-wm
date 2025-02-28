use std::vec::Vec;
use std::vec;
use std::collections::{ HashMap, VecDeque };
use std::fmt;
use std::boxed::Box;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;

use linux_framebuffer::Framebuffer;

use crate::framebuffer::{ FramebufferWriter, Point, Dimensions, RGBColor };
use crate::themes::{ ThemeInfo, Themes, get_theme_info };
use crate::utils::{ min, point_inside };
use crate::messages::*;
use crate::dirs::config_dir;
use crate::proxy_window_like::ProxyWindowLike;
use crate::essential::desktop_background::DesktopBackground;
use crate::essential::taskbar::Taskbar;
use crate::essential::lock_screen::LockScreen;
use crate::essential::workspace_indicator::WorkspaceIndicator;
use crate::essential::start_menu::StartMenu;
use crate::essential::about::About;
use crate::essential::help::Help;
use crate::essential::onscreen_keyboard::OnscreenKeyboard;
//use crate::logging::log;

//todo: a lot of the usize should be changed to u16

pub const TASKBAR_HEIGHT: usize = 38;
pub const INDICATOR_HEIGHT: usize = 20;
const WINDOW_TOP_HEIGHT: usize = 26;

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

#[derive(PartialEq)]
pub enum Workspace {
  All,
  Workspace(u8), //goes from 0-8
}

pub struct WindowLikeInfo {
  id: usize,
  window_like: WindowBox,
  top_left: Point,
  dimensions: Dimensions,
  workspace: Workspace,
  fullscreen: bool,
}

impl fmt::Debug for WindowLikeInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("WindowLikeInfo").field("id", &self.id).field("top_left", &self.top_left).field("dimensions", &self.dimensions).field("window_like", &"todo: print this out too").finish()
  }
}

pub struct WindowManager {
  writer: RefCell<FramebufferWriter>,
  rotate: bool,
  grayscale: bool,
  id_count: usize,
  window_infos: Vec<WindowLikeInfo>,
  osk: Option<WindowLikeInfo>,
  dimensions: Dimensions,
  theme: Themes,
  focused_id: usize,
  pub locked: bool,
  current_workspace: u8,
  framebuffer: Framebuffer,
  clipboard: Option<String>,
}

//1 is up, 2 is down

impl WindowManager {
  pub fn new(writer: FramebufferWriter, framebuffer: Framebuffer, dimensions: Dimensions, rotate: bool, grayscale: bool) -> Self {
    //println!("bg: {}x{}", dimensions[0], dimensions[1] - TASKBAR_HEIGHT - INDICATOR_HEIGHT);
    let mut wm = WindowManager {
      writer: RefCell::new(writer),
      rotate,
      grayscale,
      id_count: 0,
      window_infos: Vec::new(),
      osk: None,
      dimensions,
      theme: Default::default(),
      focused_id: 0,
      locked: false,
      current_workspace: 0,
      framebuffer,
      clipboard: None,
    };
    wm.lock();
    wm.change_theme();
    wm
  }

  pub fn add_window_like(&mut self, mut window_like: Box<dyn WindowLike>, top_left: Point, dimensions: Option<Dimensions>) {
    let subtype = window_like.subtype();
    let dimensions = dimensions.unwrap_or(window_like.ideal_dimensions(self.dimensions));
    self.id_count = self.id_count + 1;
    let id = self.id_count;
    window_like.handle_message(WindowMessage::Init(dimensions));
    let dimensions = if window_like.subtype() == WindowLikeType::Window { [dimensions[0], dimensions[1] + WINDOW_TOP_HEIGHT] } else { dimensions };
    let window_info = WindowLikeInfo {
      id,
      window_like,
      top_left,
      dimensions,
      workspace: if subtype == WindowLikeType::Window {
        Workspace::Workspace(self.current_workspace)
      } else {
        Workspace::All
      },
      fullscreen: false,
    };
    if subtype == WindowLikeType::OnscreenKeyboard {
      self.osk = Some(window_info);
    } else {
      self.focused_id = id;
      self.window_infos.push(window_info);
    }
  }

  fn get_focused_index(&self) -> Option<usize> {
    self.window_infos.iter().position(|w| w.id == self.focused_id)
  }

  //used to return Vec<&WindowLikeInfo>, doesn't anymore for good reason
  fn get_windows_in_workspace(&self, include_non_window: bool) -> Vec<&WindowLikeInfo> {
    self.window_infos.iter().filter(|w| {
      match w.workspace {
        Workspace::Workspace(workspace) => workspace == self.current_workspace,
        _ => include_non_window, //filter out taskbar, indicator, background, start menu, etc if true
      }
    }).collect()
  }

  fn lock(&mut self) {
    self.locked = true;
    self.window_infos = Vec::new();
    self.add_window_like(Box::new(LockScreen::new()), [0, 0], None);
  }

  fn unlock(&mut self) {
    self.locked = false;
    self.window_infos = Vec::new();
    self.add_window_like(Box::new(DesktopBackground::new()), [0, INDICATOR_HEIGHT], None);
    self.add_window_like(Box::new(Taskbar::new()), [0, self.dimensions[1] - TASKBAR_HEIGHT], None);
    self.add_window_like(Box::new(WorkspaceIndicator::new()), [0, 0], None);
  }

  fn change_theme(&mut self) {
    self.theme = Default::default();
    if let Ok(mut file) = File::open(format!("{}/ming-wm/themes", config_dir().unwrap().into_os_string().into_string().unwrap())) {
      let mut contents = String::new();
      file.read_to_string(&mut contents).unwrap();
      let lines: Vec<&str> = contents.split("\n").collect();
      if lines.len() > self.current_workspace.into() {
        self.theme = Themes::from_str(lines[self.current_workspace as usize]).unwrap_or(Default::default());
      }
    }
  }

  //if off_only is true, also handle request
  //written confusingly but it works I promise
  fn toggle_start_menu(&mut self, off_only: bool) -> WindowMessageResponse {
    let start_menu_exists = self.window_infos.iter().find(|w| w.window_like.subtype() == WindowLikeType::StartMenu).is_some();
    if (start_menu_exists && off_only) || !off_only {
      let taskbar_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap();
      self.focused_id = self.window_infos[taskbar_index].id;
      if off_only {
        self.handle_request(WindowManagerRequest::CloseStartMenu);
      }
      self.window_infos[taskbar_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::StartMenu))
    } else {
      WindowMessageResponse::DoNothing
    }
  }

  fn taskbar_update_windows(&mut self) {
    let taskbar_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap();
    let mut relevant: WindowsVec = self.get_windows_in_workspace(false).iter().map(|w| (w.id, w.window_like.title().to_string())).collect();
    relevant.sort_by(|a, b| a.0.cmp(&b.0)); //sort by ids so order is consistent
    let message = WindowMessage::Info(InfoType::WindowsInWorkspace(
      relevant,
      self.focused_id
    ));
    self.window_infos[taskbar_index].window_like.handle_message(message);
  }

  fn move_index_to_top(&mut self, index: usize) {
    let removed = self.window_infos.remove(index);
    self.window_infos.push(removed);
  }

  pub fn handle_message(&mut self, message: WindowManagerMessage) {
    let mut use_saved_buffer = false;
    let mut redraw_ids = None;
    let response: WindowMessageResponse = match message {
      WindowManagerMessage::KeyChar(key_char) => {
        //check if is special key (key releases are guaranteed to be special keys)
        //eg: ctrl, alt, command/windows, shift, or caps lock
        match key_char {
          KeyChar::Alt(c) => {
            let mut press_response = WindowMessageResponse::DoNothing;
            if !self.locked {
              //keyboard shortcut
              let shortcuts = HashMap::from([
                //alt+E kills ming-wm when it is unlocked, but that is handled at a higher level
                ('s', ShortcutType::StartMenu),
                ('[', ShortcutType::FocusPrevWindow),
                (']', ShortcutType::FocusNextWindow),
                ('q', ShortcutType::QuitWindow),
                ('c', ShortcutType::CenterWindow),
                ('f', ShortcutType::FullscreenWindow),
                ('w', ShortcutType::HalfWidthWindow),
                ('C', ShortcutType::ClipboardCopy),
                ('P', ShortcutType::ClipboardPaste(String::new())),
                //move window a small amount
                ('h', ShortcutType::MoveWindow(Direction::Left)),
                ('j', ShortcutType::MoveWindow(Direction::Down)),
                ('k', ShortcutType::MoveWindow(Direction::Up)),
                ('l', ShortcutType::MoveWindow(Direction::Right)),
                //move window to edges
                ('H', ShortcutType::MoveWindowToEdge(Direction::Left)),
                ('J', ShortcutType::MoveWindowToEdge(Direction::Down)),
                ('K', ShortcutType::MoveWindowToEdge(Direction::Up)),
                ('L', ShortcutType::MoveWindowToEdge(Direction::Right)),
                //expand window size
                ('n', ShortcutType::ChangeWindowSize(Direction::Right)),
                ('m', ShortcutType::ChangeWindowSize(Direction::Down)),
                //shrink window size
                ('N', ShortcutType::ChangeWindowSize(Direction::Left)),
                ('M', ShortcutType::ChangeWindowSize(Direction::Up)),
                //no 10th workspace
                ('1', ShortcutType::SwitchWorkspace(0)),
                ('2', ShortcutType::SwitchWorkspace(1)),
                ('3', ShortcutType::SwitchWorkspace(2)),
                ('4', ShortcutType::SwitchWorkspace(3)),
                ('5', ShortcutType::SwitchWorkspace(4)),
                ('6', ShortcutType::SwitchWorkspace(5)),
                ('7', ShortcutType::SwitchWorkspace(6)),
                ('8', ShortcutType::SwitchWorkspace(7)),
                ('9', ShortcutType::SwitchWorkspace(8)),
                //shfit + num key
                ('!', ShortcutType::MoveWindowToWorkspace(0)),
                ('@', ShortcutType::MoveWindowToWorkspace(1)),
                ('#', ShortcutType::MoveWindowToWorkspace(2)),
                ('$', ShortcutType::MoveWindowToWorkspace(3)),
                ('%', ShortcutType::MoveWindowToWorkspace(4)),
                ('^', ShortcutType::MoveWindowToWorkspace(5)),
                ('&', ShortcutType::MoveWindowToWorkspace(6)),
                ('*', ShortcutType::MoveWindowToWorkspace(7)),
                ('(', ShortcutType::MoveWindowToWorkspace(8)),
                //
              ]);
              if let Some(shortcut) = shortcuts.get(&c) {
                match shortcut {
                  &ShortcutType::StartMenu => {
                    //send to taskbar
                    press_response = self.toggle_start_menu(false);
                    if press_response != WindowMessageResponse::Request(WindowManagerRequest::CloseStartMenu) {
                      //only thing that needs to be redrawed is the start menu and taskbar
                      let start_menu_id = self.id_count + 1;
                      let taskbar_id = self.window_infos.iter().find(|w| w.window_like.subtype() == WindowLikeType::Taskbar).unwrap().id;
                      redraw_ids = Some(vec![start_menu_id, taskbar_id]);
                    }
                  },
                  &ShortcutType::MoveWindow(direction) | &ShortcutType::MoveWindowToEdge(direction) => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let focused_info = &self.window_infos[focused_index];
                      if focused_info.window_like.subtype() == WindowLikeType::Window && !focused_info.fullscreen {
                        let delta = 15;
                        let window_x = self.window_infos[focused_index].top_left[0];
                        let window_y = self.window_infos[focused_index].top_left[1];
                        let mut changed = true;
                        if direction == Direction::Left {
                          if window_x == 0 {
                            changed = false;
                          } else if window_x < delta || shortcut == &ShortcutType::MoveWindowToEdge(direction) {
                            self.window_infos[focused_index].top_left[0] = 0;
                          } else {
                            self.window_infos[focused_index].top_left[0] -= delta;
                          }
                        } else if direction == Direction::Down {
                          let max_y = self.dimensions[1] - TASKBAR_HEIGHT - focused_info.dimensions[1];
                          if window_y == max_y {
                            changed = false;
                          } else if window_y > (max_y - delta) || shortcut == &ShortcutType::MoveWindowToEdge(direction) {
                            self.window_infos[focused_index].top_left[1] = max_y;
                          } else {
                            self.window_infos[focused_index].top_left[1] += delta;
                          }
                        } else if direction == Direction::Up {
                          let min_y = INDICATOR_HEIGHT;
                          if window_y == min_y {
                            changed = false;
                          } else if window_y < (min_y + delta) || shortcut == &ShortcutType::MoveWindowToEdge(direction) {
                            self.window_infos[focused_index].top_left[1] = min_y;
                          } else {
                            self.window_infos[focused_index].top_left[1] -= delta;
                          }
                        } else if direction == Direction::Right {
                          let max_x = self.dimensions[0] - focused_info.dimensions[0];
                          if window_x == max_x {
                            changed = false;
                          } else if window_x > (max_x - delta) || shortcut == &ShortcutType::MoveWindowToEdge(direction) {
                            self.window_infos[focused_index].top_left[0] = max_x;
                          } else {
                            self.window_infos[focused_index].top_left[0] += delta;
                          }
                        }
                        if changed {
                          press_response = WindowMessageResponse::JustRedraw;
                          //avoid drawing everything under the moving window, much more efficient
                          use_saved_buffer = true;
                          redraw_ids = Some(vec![self.focused_id]);
                        }
                      }
                    }
                  },
                  &ShortcutType::ChangeWindowSize(direction) => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let focused_info = &self.window_infos[focused_index];
                      if focused_info.window_like.subtype() == WindowLikeType::Window && focused_info.window_like.resizable() && !focused_info.fullscreen {
                        //mostly arbitrary
                        let min_window_size = [100, WINDOW_TOP_HEIGHT + 5];
                        let mut changed = false;
                        let delta = 15;
                        let window = &mut self.window_infos[focused_index];
                        if direction == Direction::Right {
                          //expand x
                          if window.dimensions[0] + delta != self.dimensions[0] {
                            window.dimensions[0] += delta;
                            let max_width = self.dimensions[0] - window.top_left[0];
                            if window.dimensions[0] > max_width {
                              window.dimensions[0] = max_width;
                            }
                            changed = true;
                          }
                        } else if direction == Direction::Down {
                          //expand y
                          let max_height = self.dimensions[1] - window.top_left[1] - INDICATOR_HEIGHT - TASKBAR_HEIGHT;
                          if window.dimensions[1] + delta != max_height {
                            window.dimensions[1] += delta;
                            if window.dimensions[1] > max_height {
                              window.dimensions[1] = max_height;
                            }
                            changed = true;
                          }
                        } else if direction == Direction::Left {
                          //shrink x
                          if window.dimensions[0] - delta != min_window_size[0] {
                            window.dimensions[0] -= delta;
                            if window.dimensions[0] < min_window_size[0] {
                              window.dimensions[0] = min_window_size[0];
                            }
                            changed = true;
                          }
                        } else if direction == Direction::Up {
                          //shrink y
                          if window.dimensions[1] - delta != min_window_size[1] {
                            window.dimensions[1] -= delta;
                            if window.dimensions[1] < min_window_size[1] {
                              window.dimensions[1] = min_window_size[1];
                            }
                            changed = true;
                          }
                        }
                        if changed {
                          let new_dimensions = [window.dimensions[0], window.dimensions[1] - WINDOW_TOP_HEIGHT];
                          self.window_infos[focused_index].window_like.handle_message(WindowMessage::ChangeDimensions(new_dimensions));
                          press_response = WindowMessageResponse::JustRedraw;
                          use_saved_buffer = true;
                          redraw_ids = Some(vec![self.focused_id]);
                        }
                      }
                    }
                  },
                  &ShortcutType::SwitchWorkspace(workspace) => {
                    if self.current_workspace != workspace {
                      //close start menu if open
                      self.toggle_start_menu(true);
                      self.current_workspace = workspace;
                      //change theme
                      self.change_theme();
                      //send to desktop background
                      let desktop_background_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::DesktopBackground).unwrap();
                      self.window_infos[desktop_background_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::SwitchWorkspace(self.current_workspace)));
                      //send to workspace indicator
                      let indicator_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::WorkspaceIndicator).unwrap();
                      self.focused_id = self.window_infos[indicator_index].id;
                      self.window_infos[indicator_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::SwitchWorkspace(self.current_workspace)));
                      self.taskbar_update_windows();
                      press_response = WindowMessageResponse::JustRedraw;
                    }
                  },
                  &ShortcutType::MoveWindowToWorkspace(workspace) => {
                    if self.current_workspace != workspace {
                      if let Some(focused_index) = self.get_focused_index() {
                        if self.window_infos[focused_index].window_like.subtype() == WindowLikeType::Window {
                          self.window_infos[focused_index].workspace = Workspace::Workspace(workspace);
                          self.taskbar_update_windows();
                          press_response = WindowMessageResponse::JustRedraw;
                        }
                      }
                    }
                  },
                  &ShortcutType::FocusPrevWindow | &ShortcutType::FocusNextWindow => {
                    let current_index = self.get_focused_index().unwrap_or(0);
                    let mut new_focus_index = current_index;
                    loop {
                      if shortcut == &ShortcutType::FocusPrevWindow {
                        if new_focus_index == 0 {
                          new_focus_index = self.window_infos.len() - 1;
                        } else {
                          new_focus_index -= 1;
                        }
                      } else {
                        new_focus_index += 1;
                        if new_focus_index == self.window_infos.len() {
                          new_focus_index = 0;
                        }
                      }
                      if self.window_infos[new_focus_index].window_like.subtype() == WindowLikeType::Window && self.window_infos[new_focus_index].workspace == Workspace::Workspace(self.current_workspace) {
                        //switch focus to this
                        self.focused_id = self.window_infos[new_focus_index].id;
                        //elevate it to the top
                        self.move_index_to_top(new_focus_index);
                        self.taskbar_update_windows();
                        press_response = WindowMessageResponse::JustRedraw;
                        break;
                      } else if new_focus_index == current_index {
                        break; //did a full loop, found no windows
                      }
                    }
                  },
                  &ShortcutType::QuitWindow => {
                    if let Some(focused_index) = self.get_focused_index() {
                      if self.window_infos[focused_index].window_like.subtype() == WindowLikeType::Window {
                        self.window_infos.remove(focused_index);
                        self.taskbar_update_windows();
                        press_response = WindowMessageResponse::JustRedraw;
                      }
                    }
                  },
                  &ShortcutType::CenterWindow => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let window_dimensions = &self.window_infos[focused_index].dimensions;
                      self.window_infos[focused_index].top_left = [self.dimensions[0] / 2 - window_dimensions[0] / 2, self.dimensions[1] / 2 - window_dimensions[1] / 2];
                      use_saved_buffer = true;
                      press_response = WindowMessageResponse::JustRedraw;
                    }
                  },
                  &ShortcutType::FullscreenWindow => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let window_like = &self.window_infos[focused_index].window_like;
                      if window_like.subtype() == WindowLikeType::Window && window_like.resizable() {
                        //toggle fullscreen
                        self.window_infos[focused_index].fullscreen ^= true;
                        //todo: send message to window about resize
                        let new_dimensions;
                        if self.window_infos[focused_index].fullscreen {
                          new_dimensions = [self.dimensions[0], self.dimensions[1] - TASKBAR_HEIGHT - INDICATOR_HEIGHT];
                          self.window_infos[focused_index].top_left = [0, INDICATOR_HEIGHT];
                          redraw_ids = Some(vec![self.window_infos[focused_index].id]);
                        } else {
                          new_dimensions = self.window_infos[focused_index].dimensions;
                        }
                        self.window_infos[focused_index].window_like.handle_message(WindowMessage::ChangeDimensions([new_dimensions[0], new_dimensions[1] - WINDOW_TOP_HEIGHT]));
                        press_response = WindowMessageResponse::JustRedraw;
                      }
                    }
                  },
                  &ShortcutType::HalfWidthWindow => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let window_like = &self.window_infos[focused_index].window_like;
                      if window_like.subtype() == WindowLikeType::Window && window_like.resizable() {
                        self.window_infos[focused_index].fullscreen = false;
                        let top_left = &mut self.window_infos[focused_index].top_left;
                        if top_left[0] > self.dimensions[0] / 2 {
                          top_left[0] = self.dimensions[0] / 2;
                        } else {
                          top_left[0] = 0;
                        }
                        top_left[1] = INDICATOR_HEIGHT;
                        //full height, half width
                        let new_dimensions = [self.dimensions[0] / 2, self.dimensions[1] - INDICATOR_HEIGHT - TASKBAR_HEIGHT];
                        self.window_infos[focused_index].dimensions = new_dimensions;
                        self.window_infos[focused_index].window_like.handle_message(WindowMessage::ChangeDimensions([new_dimensions[0], new_dimensions[1] - WINDOW_TOP_HEIGHT]));
                        press_response = WindowMessageResponse::JustRedraw;
                      }
                    }
                  },
                  &ShortcutType::ClipboardCopy => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let window_like = &self.window_infos[focused_index].window_like;
                      if window_like.subtype() == WindowLikeType::Window {
                        press_response = self.window_infos[focused_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::ClipboardCopy));
                      }
                    }
                  },
                  &ShortcutType::ClipboardPaste(_) => {
                    if let Some(focused_index) = self.get_focused_index() {
                      let window_like = &self.window_infos[focused_index].window_like;
                      if window_like.subtype() == WindowLikeType::Window && self.clipboard.is_some() {
                        press_response = self.window_infos[focused_index].window_like.handle_message(WindowMessage::Shortcut(ShortcutType::ClipboardPaste(self.clipboard.clone().unwrap())));
                      }
                    }
                  },
                };
              }
            }
            press_response
          },
          KeyChar::Press(c) | KeyChar::Ctrl(c) => {
            let mut press_response = WindowMessageResponse::DoNothing;
            //send to focused window
            if let Some(focused_index) = self.get_focused_index() {
              press_response = self.window_infos[focused_index].window_like.handle_message(if key_char == KeyChar::Press(c) {
                WindowMessage::KeyPress(KeyPress {
                  key: c,
                })
              } else {
                WindowMessage::CtrlKeyPress(KeyPress {
                  key: c,
                })
              });
              //at most, only the focused window needs to be redrawed
              redraw_ids = Some(vec![self.window_infos[focused_index].id]);
              //requests can result in window openings and closings, etc
              if press_response != WindowMessageResponse::JustRedraw {
                redraw_ids = None;
              }
            }
            press_response
          },
        }
      },
      WindowManagerMessage::Touch(x, y) => {
        if x < 100 && y < 100 {
          //toggle onscreen keyboard if top left keyboard clicked
          if self.osk.is_some() {
            self.osk = None;
          } else {
            let osk = Box::new(OnscreenKeyboard::new());
            let ideal_dimensions = osk.ideal_dimensions(self.dimensions);
            self.add_window_like(osk, [175, self.dimensions[1] - TASKBAR_HEIGHT - 250], Some(ideal_dimensions));
          }
          WindowMessageResponse::JustRedraw
        } else {
          //see if in onscreen keyboard, if so send to it after offsetting coords
          if self.osk.is_some() {
            let osk = self.osk.as_mut().unwrap();
            if point_inside([x, y], osk.top_left, osk.dimensions) {
              osk.window_like.handle_message(WindowMessage::Touch(x - osk.top_left[0], y - osk.top_left[1]))
            } else {
              WindowMessageResponse::DoNothing
            }
          } else {
            WindowMessageResponse::DoNothing
          }
        }
      }
    };
    if response != WindowMessageResponse::DoNothing {
      let is_key_char_request = response.is_key_char_request();
      match response {
        WindowMessageResponse::Request(request) => self.handle_request(request),
        _ => {},
      };
      if !is_key_char_request {
        self.draw(redraw_ids, use_saved_buffer);
      }
    }
  }
  
  pub fn handle_request(&mut self, request: WindowManagerRequest) {
    let subtype = if let Some(focused_index) = self.get_focused_index() {
      Some(self.window_infos[focused_index].window_like.subtype())
    } else {
      None
    };
    match request {
      WindowManagerRequest::OpenWindow(w) => {
        let subtype = subtype.unwrap();
        if subtype != WindowLikeType::Taskbar && subtype != WindowLikeType::StartMenu {
          return;
        }
        let w: Option<WindowBox> = match w.as_str() {
          "Minesweeper" => Some(Box::new(ProxyWindowLike::new_rust("minesweeper"))),
          "Reversi" => Some(Box::new(ProxyWindowLike::new_rust("reversi"))),
          "Malvim" => Some(Box::new(ProxyWindowLike::new_rust("malvim"))),
          "Terminal" => Some(Box::new(ProxyWindowLike::new_rust("terminal"))),
          "Audio Player" => Some(Box::new(ProxyWindowLike::new_rust("audio_player"))),
          "File Explorer" => Some(Box::new(ProxyWindowLike::new_rust("file_explorer"))),
          "StartMenu" => Some(Box::new(StartMenu::new())),
          "About" => Some(Box::new(About::new())),
          "Help" => Some(Box::new(Help::new())),
          _ => None,
        };
        if w.is_none() {
          return;
        }
        let w = w.unwrap();
        //close start menu if open
        self.toggle_start_menu(true);
        let ideal_dimensions = w.ideal_dimensions(self.dimensions);
        let top_left = match w.subtype() {
          WindowLikeType::StartMenu => [0, self.dimensions[1] - TASKBAR_HEIGHT - ideal_dimensions[1]],
          WindowLikeType::Window => [42, 42],
          _ => [0, 0],
        };
        self.add_window_like(w, top_left, Some(ideal_dimensions));
        self.taskbar_update_windows();
      },
      WindowManagerRequest::CloseStartMenu => {
        let subtype = subtype.unwrap();
        if subtype != WindowLikeType::Taskbar && subtype != WindowLikeType::StartMenu {
          return;
        }
        let start_menu_index = self.window_infos.iter().position(|w| w.window_like.subtype() == WindowLikeType::StartMenu);
        if let Some(start_menu_index) = start_menu_index {
          self.window_infos.remove(start_menu_index);
        }
      },
      WindowManagerRequest::Unlock => {
        if subtype.unwrap() != WindowLikeType::LockScreen {
          return;
        }
        self.unlock();
      },
      WindowManagerRequest::Lock => {
        if subtype.unwrap() != WindowLikeType::StartMenu {
          return;
        }
        self.lock();
      },
      WindowManagerRequest::ClipboardCopy(content) => {
        self.clipboard = Some(content);
      },
      WindowManagerRequest::DoKeyChar(kc) => {
        self.handle_message(WindowManagerMessage::KeyChar(kc));
      },
    };
  }

  fn get_true_top_left(top_left: &Point, is_window: bool) -> Point {
    [top_left[0], top_left[1] + if is_window { WINDOW_TOP_HEIGHT } else { 0 }]
  }

  //another issue with a huge vector of draw instructions; it takes up heap memory
  pub fn draw(&mut self, maybe_redraw_ids: Option<Vec<usize>>, use_saved_buffer: bool) {
    let theme_info = get_theme_info(&self.theme).unwrap();
    //use in conjunction with redraw ids, so a window moving can work without redrawing everything,
    //can just redraw the saved state + window
    if use_saved_buffer {
      self.writer.borrow_mut().write_saved_buffer_to_raw();
    }
    //get windows to redraw
    let redraw_ids = maybe_redraw_ids.unwrap_or(Vec::new());
    let mut all_in_workspace = self.get_windows_in_workspace(true);
    if let Some(osk) = &self.osk {
      all_in_workspace.push(osk);
    }
    let maybe_length = all_in_workspace.len();
    let redraw_windows = all_in_workspace.iter().filter(|w| {
      //basically, maybe_redraw_ids was None
      if redraw_ids.len() > 0 {
        redraw_ids.contains(&w.id) || w.window_like.subtype() == WindowLikeType::OnscreenKeyboard
      } else {
        true
      }
    });
    //these are needed to decide when to snapshot
    let max_index = if redraw_ids.len() > 0 { redraw_ids.len() } else { maybe_length } - 1;
    let mut w_index = 0;
    for window_info in redraw_windows {
      let window_dimensions = if window_info.fullscreen {
        [self.dimensions[0], self.dimensions[1] - TASKBAR_HEIGHT - INDICATOR_HEIGHT]
      } else {
        window_info.dimensions
      };
      let mut instructions = VecDeque::from(window_info.window_like.draw(&theme_info));
      let is_window = window_info.window_like.subtype() == WindowLikeType::Window;
      if is_window {
        //if this is the top most window to draw, snapshot
        if w_index == max_index && !use_saved_buffer && redraw_ids.len() == 0 {
          self.writer.borrow_mut().save_buffer();
        }
        //offset top left by the window top height for windows (because windows can't draw in that region)
        instructions = instructions.iter().map(|instruction| {
          match instruction {
            DrawInstructions::Rect(top_left, dimensions, color) => DrawInstructions::Rect(WindowManager::get_true_top_left(top_left, is_window), *dimensions, *color),
            DrawInstructions::Circle(centre, radius, color) => DrawInstructions::Circle(WindowManager::get_true_top_left(centre, is_window), *radius, *color),
            DrawInstructions::Text(top_left, fonts, text, color, bg_color, horiz_spacing, mono_width) => DrawInstructions::Text(WindowManager::get_true_top_left(top_left, is_window), fonts.clone(), text.clone(), *color, *bg_color, *horiz_spacing, *mono_width),
            DrawInstructions::Bmp(top_left, path, reverse) => DrawInstructions::Bmp(WindowManager::get_true_top_left(top_left, is_window), path.to_string(), *reverse),
            DrawInstructions::Gradient(top_left, dimensions, start_color, end_color, steps) => DrawInstructions::Gradient(WindowManager::get_true_top_left(top_left, is_window), *dimensions, *start_color, *end_color, *steps),
          }
        }).collect();
        //draw window background
        instructions.push_front(DrawInstructions::Rect([0, 0], window_dimensions, theme_info.background));
        //draw window top decorations and what not
        instructions.extend(vec![
          //left top border
          DrawInstructions::Rect([0, 0], [window_dimensions[0], 1], theme_info.border_left_top),
          DrawInstructions::Rect([0, 0], [1, window_dimensions[1]], theme_info.border_left_top),
          //top
          DrawInstructions::Rect([1, 1], [window_dimensions[0] - 2, WINDOW_TOP_HEIGHT - 3], theme_info.top),
          DrawInstructions::Text([4, 4], vec!["nimbus-roman".to_string()], window_info.window_like.title().to_string(), theme_info.top_text, theme_info.top, None, None),
          //top bottom border
          DrawInstructions::Rect([1, WINDOW_TOP_HEIGHT - 2], [window_dimensions[0] - 2, 2], theme_info.border_left_top),
          //right bottom border
          DrawInstructions::Rect([window_dimensions[0] - 1, 1], [1, window_dimensions[1] - 1], theme_info.border_right_bottom),
          DrawInstructions::Rect([1, window_dimensions[1] - 1], [window_dimensions[0] - 1, 1], theme_info.border_right_bottom),
        ]);
      }
      let mut framebuffer_info = self.writer.borrow().get_info();
      let bytes_per_pixel = framebuffer_info.bytes_per_pixel;
      let window_width = window_dimensions[0];
      let window_height = window_dimensions[1];
      framebuffer_info.width = window_width;
      framebuffer_info.height = window_height;
      framebuffer_info.stride = window_width;
      framebuffer_info.byte_len = window_width * window_height * bytes_per_pixel;
      //make a writer just for the window
      let mut window_writer: FramebufferWriter = FramebufferWriter::new(self.grayscale);
      window_writer.init(framebuffer_info);
      for instruction in instructions {
        //unsafe { SERIAL1.lock().write_text(&format!("{:?}\n", instruction)); }
        match instruction {
          DrawInstructions::Rect(top_left, dimensions, color) => {
            //try and prevent overflows out of the window
            let true_dimensions = [
              min(dimensions[0], window_dimensions[0] - top_left[0]),
              min(dimensions[1], window_dimensions[1] - top_left[1]),
            ];
            window_writer.draw_rect(top_left, true_dimensions, color);
          },
          DrawInstructions::Circle(centre, radius, color) => {
            window_writer.draw_circle(centre, radius, color);
          },
          DrawInstructions::Text(top_left, fonts, text, color, bg_color, horiz_spacing, mono_width) => {
            window_writer.draw_text(top_left, fonts, &text, color, bg_color, horiz_spacing.unwrap_or(1), mono_width);
          },
          DrawInstructions::Bmp(top_left, path, reverse) => {
            window_writer.draw_bmp(top_left, path, reverse);
          },
          DrawInstructions::Gradient(top_left, dimensions, start_color, end_color, steps) => {
            window_writer.draw_gradient(top_left, dimensions, start_color, end_color, steps);
          },
        }
      }
      self.writer.borrow_mut().draw_buffer(window_info.top_left, window_dimensions[1], window_dimensions[0] * bytes_per_pixel, &window_writer.get_buffer());
      w_index += 1;
    }
    //could probably figure out a way to do borrow() when self.rotate is false but does it matter?
    let mut writer_borrow = self.writer.borrow_mut();
    let frame = if self.rotate { writer_borrow.get_transposed_buffer() } else { writer_borrow.get_buffer() };
    self.framebuffer.write_frame(frame);
  }
}

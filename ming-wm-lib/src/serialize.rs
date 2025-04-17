use std::fmt::Display;

use crate::themes::ThemeInfo;
use crate::messages::{ WindowMessageResponse, WindowManagerRequest, KeyPress, WindowMessage, Direction, ShortcutType, InfoType };
use crate::window_manager_types::{ KeyChar, DrawInstructions, WindowLikeType };
use crate::framebuffer_types::Dimensions;
use crate::utils::get_rest_of_split;

//serde + ron but worse! yay
//not same as ron - simplified
//very messy

//todo: bug with extra byte when copy/pasting because of this... maybe it's the newline or something?

//I can't do `impl fmt::Display for RGBColor` which is annoying

//to type \x1F, do ctrl+7, for \x1E, do ctrl+6

fn array_to_string<T: Display>(array: &[T]) -> String {
  let mut output = String::new();
  for item in array {
    output += &format!("{}{}", if output == String::new() {
      ""
    } else {
      "\x1F"
    }, item);
  }
  output
}

fn option_to_string<T: Display>(option: &Option<T>) -> String {
  if let Some(value) = option {
    format!("S{}", value)
  } else {
    "N".to_string()
  }
}

fn get_color(serialized: &str) -> Result<[u8; 3], ()> {
  let rgb = serialized.split("\x1F");
  let mut color = [0; 3];
  //won't return error if rgb is 0, 1, or 2 elements.
  //I guess that's okay, since it doesn't panic either
  //c_i is the loop counter. enumerate(), you are awesome
  for (c_i, c) in rgb.enumerate() {
    if c_i == 3 {
      return Err(());
    }
    if let Ok(c) = c.parse() {
      color[c_i] = c;
    } else {
      return Err(());
    }
  }
  Ok(color)
}

fn get_two_array(serialized: &str) -> Result<[usize; 2], ()> {
  let mut arg = serialized.split("\x1F");
  let mut a = [0; 2];
  for i in 0..2 {
    if let Some(n) = arg.next() {
      if let Ok(n) = n.parse() {
        a[i] = n;
        continue
      }
    }
    return Err(());
  }
  Ok(a)
}

pub trait Serializable {
  fn serialize(&self) -> String;
  fn deserialize(serialized: &str) -> Result<Self, ()> where Self: Sized;
}

//ripe for macros when I figure them out

impl Serializable for ThemeInfo {
  fn serialize(&self) -> String {
    format!("{}:{}:{}:{}:{}:{}:{}:{}:{}", array_to_string(&self.top), array_to_string(&self.background), array_to_string(&self.border_left_top), array_to_string(&self.border_right_bottom), array_to_string(&self.text), array_to_string(&self.top_text), array_to_string(&self.alt_background), array_to_string(&self.alt_text), array_to_string(&self.alt_secondary))
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    //strip newline at the end
    let serialized = if serialized.ends_with("\n") { &serialized[..serialized.len() - 1] } else { serialized };
    let mut theme_info: ThemeInfo = Default::default();
    let arrays = serialized.split(":");
    //won't error or panic if less than 9... rest will just be black by default I guess
    for (a_i, a) in arrays.enumerate() {
      if a_i == 9 {
        return Err(());
      }
      let color = get_color(a)?;
      match a_i {
        0 => {
          theme_info.top = color;
        },
        1 => {
          theme_info.background = color;
        },
        2 => {
          theme_info.border_left_top = color;
        },
        3 => {
          theme_info.border_right_bottom = color;
        },
        4 => {
          theme_info.text = color;
        },
        5 => {
          theme_info.top_text = color;
        },
        6 => {
          theme_info.alt_background = color;
        },
        7 => {
          theme_info.alt_text = color;
        },
        8 => {
          theme_info.alt_secondary = color;
        },
        _ => {},
      };
      if a_i == 8 {
        return Ok(theme_info);
      }
    }
    Err(())
  }
}

#[test]
fn theme_info_serialize_deserialize() {
  use crate::themes::get_theme_info;
  let theme_info = get_theme_info(&Default::default()).unwrap();
  let serialized = theme_info.serialize();
  assert!(serialized == ThemeInfo::deserialize(&serialized).unwrap().serialize());
}

impl Serializable for WindowMessageResponse {
  fn serialize(&self) -> String {
    match self {
      WindowMessageResponse::JustRedraw => "JustRedraw".to_string(),
      WindowMessageResponse::DoNothing => "DoNothing".to_string(),
      WindowMessageResponse::Request(req) => {
        let req = match req {
          WindowManagerRequest::OpenWindow(name) => format!("OpenWindow/{}", name),
          WindowManagerRequest::ClipboardCopy(name) => format!("ClipboardCopy/{}", name),
          WindowManagerRequest::CloseStartMenu => "CloseStartMenu".to_string(),
          WindowManagerRequest::Unlock => "Unlock".to_string(),
          WindowManagerRequest::Lock => "Lock".to_string(),
          WindowManagerRequest::DoKeyChar(kc) => format!("DoKeyChar/{}", match kc {
            KeyChar::Press(c) => format!("Press/{}", c),
            KeyChar::Alt(c) => format!("Alt/{}", c),
            KeyChar::Ctrl(c) => format!("Ctrl/{}", c),
          }),
        };
        format!("Request/{}", req)
      },
    }
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    //strip newline at the end
    let serialized = if serialized.ends_with("\n") { &serialized[..serialized.len() - 1] } else { serialized };
    let mut parts = serialized.split("/");
    match parts.next().unwrap_or("Invalid") {
      "JustRedraw" => Ok(WindowMessageResponse::JustRedraw),
      "DoNothing" => Ok(WindowMessageResponse::DoNothing),
      "Request" => {
        let req = match parts.next().unwrap_or("Invalid") {
          //do get_rest_of_split instead of .next() because it is possible for window name or copy to have "/"
          "OpenWindow" => Some(WindowManagerRequest::OpenWindow(get_rest_of_split(&mut parts, Some("/")))),
          "ClipboardCopy" => Some(WindowManagerRequest::ClipboardCopy(get_rest_of_split(&mut parts, Some("/")))),
          "CloseStartMenu" => Some(WindowManagerRequest::CloseStartMenu),
          "Unlock" => Some(WindowManagerRequest::Unlock),
          "Lock" => Some(WindowManagerRequest::Lock),
          "DoKeyChar" => Some(WindowManagerRequest::DoKeyChar(
            match parts.next().unwrap_or("Invalid") {
              "Press" => KeyChar::Press(parts.next().unwrap_or("?").chars().next().unwrap()),
              "Alt" => KeyChar::Alt(parts.next().unwrap_or("?").chars().next().unwrap()),
              "Ctrl" => KeyChar::Ctrl(parts.next().unwrap_or("?").chars().next().unwrap()),
              _ => KeyChar::Press('?'), //yeah.
            }
          )),
          _ => None, //yeah...
        };
        if let Some(req) = req {
          Ok(WindowMessageResponse::Request(req))
        } else {
          Err(())
        }
      },
      _ => Err(()),
    }
  }
}

#[test]
fn window_message_response_serialize_deserialize() {
  let resp = WindowMessageResponse::JustRedraw;
  let serialized = resp.serialize();
  assert!(resp == WindowMessageResponse::deserialize(&serialized).unwrap());
  let resp = WindowMessageResponse::Request(WindowManagerRequest::OpenWindow("a".to_string()));
  let serialized = resp.serialize();
  assert!(resp == WindowMessageResponse::deserialize(&serialized).unwrap());
  let resp = WindowMessageResponse::Request(WindowManagerRequest::Unlock);
  let serialized = resp.serialize();
  assert!(resp == WindowMessageResponse::deserialize(&serialized).unwrap());
  let resp = WindowMessageResponse::Request(WindowManagerRequest::DoKeyChar(KeyChar::Alt('e')));
  let serialized = resp.serialize();
  assert!(resp == WindowMessageResponse::deserialize(&serialized).unwrap());
}

impl Serializable for DrawInstructions {
  fn serialize(&self) -> String {
    match self {
      //use \x1E (record separator) because it won't be in strings. it better fucking not be at least
      DrawInstructions::Rect(p, d, c) => format!("Rect/{}\x1E{}\x1E{}", array_to_string(p), array_to_string(d), array_to_string(c)),
      DrawInstructions::Text(p, vs, s, c1, c2, ou1, ou2) => format!("Text/{}\x1E{}\x1E{}\x1E{}\x1E{}\x1E{}\x1E{}", array_to_string(p), array_to_string(vs), s, array_to_string(c1), array_to_string(c2), option_to_string(ou1), option_to_string(ou2)),
      DrawInstructions::Gradient(p, d, c1, c2, u) => format!("Gradient/{}\x1E{}\x1E{}\x1E{}\x1E{}", array_to_string(p), array_to_string(d), array_to_string(c1), array_to_string(c2), u),
      DrawInstructions::Bmp(p, s, b) => format!("Bmp/{}\x1E{}\x1E{}", array_to_string(p), s, b),
      DrawInstructions::Circle(p, u, c) => format!("Circle/{}\x1E{}\x1E{}", array_to_string(p), u, array_to_string(c)),
    }
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    //no need to strip newlines cause the impl for Vec<DrawInstructions> does that for us
    let mut parts = serialized.split("/");
    match parts.next().unwrap_or("Invalid") {
      "Rect" => {
        let rest = get_rest_of_split(&mut parts, Some("/"));
        let mut args = rest.split("\x1E");
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let p = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let d = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let c = get_color(arg.unwrap())?;
        Ok(DrawInstructions::Rect(p, d, c))
      },
      "Text" => {
        let rest = get_rest_of_split(&mut parts, Some("/"));
        //(p, vs, s, c1, c2, ou1, ou2)
        let mut args = rest.split("\x1E");
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let p = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let mut vs = Vec::new();
        for s in arg.unwrap().split("\x1F") {
          vs.push(s.to_string());
        }
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let s = arg.unwrap();
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let c1 = get_color(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let c2 = get_color(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let arg = arg.unwrap();
        let o1 = match arg {
          "N" => None,
          _ => {
            if arg.len() > 1 {
              if let Ok(n) = arg[1..].parse() {
                Some(n)
              } else {
                None
              }
            } else {
              None
            }
          },
        };
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let arg = arg.unwrap();
        let o2 = match arg {
          "N" => None,
          _ => {
            if arg.len() > 1 {
              if let Ok(n) = arg[1..].parse() {
                Some(n)
              } else {
                None
              }
            } else {
              None
            }
          },
        };
        Ok(DrawInstructions::Text(p, vs, s.to_string(), c1, c2, o1, o2))
      },
      "Gradient" => {
        let rest = get_rest_of_split(&mut parts, Some("/"));
        //(p, d, c1, c2, u)
        let mut args = rest.split("\x1E");
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let p = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let d = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let c1 = get_color(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let c2 = get_color(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let u = arg.unwrap().parse();
        if u.is_err() {
          return Err(());
        }
        Ok(DrawInstructions::Gradient(p, d, c1, c2, u.unwrap()))
      },
      "Bmp" => {
        let rest = get_rest_of_split(&mut parts, Some("/"));
        //(p, s, b)
        let mut args = rest.split("\x1E");
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let p = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let s = arg.unwrap();
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let arg = arg.unwrap();
        if arg != "true" && arg != "false" {
          return Err(());
        }
        let b = arg == "true";
        Ok(DrawInstructions::Bmp(p, s.to_string(), b))
      },
      "Circle" => {
        let rest = get_rest_of_split(&mut parts, Some("/"));
        //(p, u, c)
        let mut args = rest.split("\x1E");
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let p = get_two_array(arg.unwrap())?;
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let u = arg.unwrap().parse();
        if u.is_err() {
          return Err(());
        }
        let arg = args.next();
        if arg.is_none() {
          return Err(());
        }
        let c = get_color(arg.unwrap())?;
        Ok(DrawInstructions::Circle(p, u.unwrap(), c))
      },
      _ => Err(()),
    }
  }
}

pub type DrawInstructionsVec = Vec<DrawInstructions>;

impl Serializable for DrawInstructionsVec {
  fn serialize(&self) -> String {
    if self.len() == 0 {
      return "empty".to_string();
    }
    let collected: Vec<_> = self.into_iter().map(|ins| ins.serialize()).collect();
    collected.join("\x1D")
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    //strip newline at the end
    let serialized = if serialized.ends_with("\n") { &serialized[..serialized.len() - 1] } else { serialized };
    if serialized == "empty" {
      return Ok(Vec::new());
    }
    let mut instructions = Vec::new();
    for ser_ins in serialized.split("\x1D") {
      if let Ok(ser_ins) = DrawInstructions::deserialize(ser_ins) {
        instructions.push(ser_ins);
      } else {
        return Err(());
      }
    }
    Ok(instructions)
  }
}

#[test]
fn draw_instructions_serialize_deserialize() {
  use std::vec;
  let instructions = vec![
    DrawInstructions::Rect([15, 24], [100, 320], [255, 0, 128]),
    DrawInstructions::Text([0, 158], vec!["nimbus-roman".to_string(), "shippori-mincho".to_string()], "Test test 1234 testing\nmictest / mictest is this thing\non?".to_string(), [12, 36, 108], [128, 128, 128], Some(1), None),
    DrawInstructions::Gradient([0, 500], [750, 125], [255, 255, 255], [0, 0, 0], 12),
    DrawInstructions::Text([123, 999], vec!["nimbus-romono".to_string()], "print!(\"{}\", variable_name);".to_string(), [12, 36, 108], [128, 128, 128], Some(44), Some(200)),
    DrawInstructions::Bmp([55, 98], "mingde".to_string(), true),
    DrawInstructions::Bmp([55, 98], "wooooo".to_string(), false),
    DrawInstructions::Circle([0, 1], 19, [128, 128, 128]),
  ];
  let serialized = instructions.serialize();
  assert!(serialized == DrawInstructionsVec::deserialize(&serialized).unwrap().serialize());
  let instructions = vec![
    DrawInstructions::Rect([0, 0], [410, 410], [0, 0, 0]),
    DrawInstructions::Text([4, 4], vec!["nimbus-romono".to_string()], "Mingde Terminal".to_string(), [255, 255, 255], [0, 0, 0], Some(0), Some(10)),
    DrawInstructions::Text([4, 34], vec!["nimbus-romono".to_string()], "$ a".to_string(), [255, 255, 255], [0, 0, 0], Some(0), Some(10)),
  ];
  let serialized = instructions.serialize() + "\n";
  assert!(serialized[..serialized.len() - 1] == DrawInstructionsVec::deserialize(&serialized).unwrap().serialize());
  let instructions = Vec::new();
  let serialized = instructions.serialize() + "\n";
  assert!(DrawInstructionsVec::deserialize(&serialized).unwrap().len() == 0);
}

impl Serializable for WindowLikeType {
  fn serialize(&self) -> String {
    match self {
      WindowLikeType::LockScreen => "LockScreen".to_string(),
      WindowLikeType::Window => "Window".to_string(),
      WindowLikeType::DesktopBackground => "DesktopBackground".to_string(),
      WindowLikeType::Taskbar => "Taskbar".to_string(),
      WindowLikeType::StartMenu => "StartMenu".to_string(),
      WindowLikeType::WorkspaceIndicator => "WorkspaceIndicator".to_string(),
      WindowLikeType::OnscreenKeyboard => "OnscreenKeyboard".to_string(),
    }
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    let serialized = if serialized.ends_with("\n") { &serialized[..serialized.len() - 1] } else { serialized };
    match serialized {
      "LockScreen" => Ok(WindowLikeType::LockScreen),
      "Window" => Ok(WindowLikeType::Window),
      "DesktopBackground" => Ok(WindowLikeType::DesktopBackground),
      "Taskbar" => Ok(WindowLikeType::Taskbar),
      "StartMenu" => Ok(WindowLikeType::StartMenu),
      "WorkspaceIndicator" => Ok(WindowLikeType::WorkspaceIndicator),
      "OnscreenKeyboard" => Ok(WindowLikeType::OnscreenKeyboard),
      _ => Err(()),
    }
  }
}

#[test]
fn window_like_type_serialize_deserialize() {
  let wl_type = WindowLikeType::Window;
  let serialized = wl_type.serialize();
  assert!(serialized == WindowLikeType::deserialize(&serialized).unwrap().serialize());
}

impl Serializable for Dimensions {
  fn serialize(&self) -> String {
    array_to_string(self)
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    //strip newline at the end
    let serialized = if serialized.ends_with("\n") { &serialized[..serialized.len() - 1] } else { serialized };
    let d = get_two_array(serialized)?;
    Ok(d)
  }
}

impl Serializable for WindowMessage {
  fn serialize(&self) -> String {
    match self {
      WindowMessage::Init(d) => format!("Init/{}", array_to_string(d)),
      WindowMessage::KeyPress(kp) => format!("KeyPress/{}", kp.key),
      WindowMessage::CtrlKeyPress(kp) => format!("CtrlKeyPress/{}", kp.key),
      WindowMessage::Shortcut(st) => format!("Shortcut/{}", match st {
        ShortcutType::StartMenu => "StartMenu".to_string(),
        ShortcutType::SwitchWorkspace(u) => format!("SwitchWorkspace/{}", u),
        ShortcutType::MoveWindowToWorkspace(u) => format!("MoveWindowToWorkspace/{}", u),
        ShortcutType::FocusPrevWindow => "FocusPrevWindow".to_string(),
        ShortcutType::FocusNextWindow => "FocusNextWindow".to_string(),
        ShortcutType::QuitWindow => "QuitWindow".to_string(),
        ShortcutType::MoveWindow(d) => format!("MoveWindow/{}", match d {
          Direction::Left => "Left",
          Direction::Down => "Down",
          Direction::Up => "Up",
          Direction::Right => "Right",
        }),
        ShortcutType::MoveWindowToEdge(d) => format!("MoveWindowToEdge/{}", match d {
          Direction::Left => "Left",
          Direction::Down => "Down",
          Direction::Up => "Up",
          Direction::Right => "Right",
        }),
        ShortcutType::ChangeWindowSize(d) => format!("ChangeWindowSize/{}", match d {
          Direction::Left => "Left",
          Direction::Down => "Down",
          Direction::Up => "Up",
          Direction::Right => "Right",
        }),
        ShortcutType::CenterWindow => "CenterWindow".to_string(),
        ShortcutType::FullscreenWindow => "FullscreenWindow".to_string(),
        ShortcutType::HalfWidthWindow => "HalfWidthWindow".to_string(),
        ShortcutType::ClipboardCopy => "ClipboardCopy".to_string(),
        ShortcutType::ClipboardPaste(s) => format!("ClipboardPaste/{}", s),
      }),
      WindowMessage::Info(i) => format!("Info/{}", match i {
        InfoType::WindowsInWorkspace(wv, u) => {
          let mut wv_string = String::new();
          for w in wv {
            wv_string += &format!("{}\x1F{}\x1F", w.0, w.1);
          }
          wv_string = wv_string[..wv_string.len() - 1].to_string();
          format!("WindowsInWorkspace/{}\x1E{}", wv_string, u)
        },
      }),
      WindowMessage::Focus => "Focus".to_string(),
      WindowMessage::Unfocus => "Unfocus".to_string(),
      WindowMessage::FocusClick => "FocusClick".to_string(),
      WindowMessage::ChangeDimensions(d) => format!("ChangeDimensions/{}", array_to_string(d)),
      WindowMessage::Touch(u1, u2) => format!("Touch/{}\x1E{}", u1, u2),
    }
  }
  fn deserialize(serialized: &str) -> Result<Self, ()> {
    let serialized = if serialized.ends_with("\n") { &serialized[..serialized.len() - 1] } else { serialized };
    let mut parts = serialized.split("/");
    match parts.next().unwrap_or("Invalid") {
      "Init" => {
        let arg = parts.next();
        if arg.is_none() {
          return Err(());
        }
        let d = get_two_array(arg.unwrap())?;
        Ok(WindowMessage::Init(d))
      },
      "KeyPress" => {
        let charg = get_rest_of_split(&mut parts, Some("/")).chars().next();
        if let Some(charg) = charg {
          Ok(WindowMessage::KeyPress(KeyPress { key: charg }))
        } else {
          Err(())
        }
      },
      "CtrlKeyPress" => {
        let charg = get_rest_of_split(&mut parts, Some("/")).chars().next();
        if let Some(charg) = charg {
          Ok(WindowMessage::CtrlKeyPress(KeyPress { key: charg }))
        } else {
          Err(())
        }
      },
      "Shortcut" => {
        let arg = parts.next();
        if arg.is_none() {
          return Err(());
        }
        let arg = arg.unwrap();
        let shortcut = match arg {
          "StartMenu" => Some(ShortcutType::StartMenu),
          "SwitchWorkspace" | "MoveWindowToWorkspace" => {
            let narg = parts.next();
            if narg.is_none() {
              None
            } else {
              let narg = narg.unwrap();
              if let Ok(n) = narg.parse() {
                if arg == "SwitchWorkspace" {
                  Some(ShortcutType::SwitchWorkspace(n))
                } else {
                  Some(ShortcutType::MoveWindowToWorkspace(n))
                }
              } else {
                None
              }
            }
          },
          "FocusPrevWindow" => Some(ShortcutType::FocusPrevWindow),
          "FocusNextWindow" => Some(ShortcutType::FocusNextWindow),
          "QuitWindow" => Some(ShortcutType::QuitWindow),
          "MoveWindow" | "MoveWindowToEdge" | "ChangeWindowSize" => {
            let darg = parts.next();
            if let Some(darg) = darg {
              let direction = match darg {
                "Left" => Some(Direction::Left),
                "Up" => Some(Direction::Up),
                "Down" => Some(Direction::Down),
                "Right" => Some(Direction::Right),
                _ => None,
              };
              if let Some(direction) = direction {
                if arg == "MoveWindow" {
                  Some(ShortcutType::MoveWindow(direction))
                } else if arg == "MoveWindowToEdge" {
                  Some(ShortcutType::MoveWindowToEdge(direction))
                } else {
                  Some(ShortcutType::ChangeWindowSize(direction))
                }
              } else {
                None
              }
            } else {
              None
            }
          },
          "CenterWindow" => Some(ShortcutType::CenterWindow),
          "FullscreenWindow" => Some(ShortcutType::FullscreenWindow),
          "HalfWidthWindow" => Some(ShortcutType::HalfWidthWindow),
          "ClipboardCopy" => Some(ShortcutType::ClipboardCopy),
          "ClipboardPaste" => Some(ShortcutType::ClipboardPaste(get_rest_of_split(&mut parts, Some("/")))),
          _ => None,
        };
        if let Some(shortcut) = shortcut {
          Ok(WindowMessage::Shortcut(shortcut))
        } else {
          Err(())
        }
      },
      "Info" => {
        //skip WindowsInWorkspace cause that's the only possible InfoType atm
        if parts.next().is_none() {
          return Err(());
        }
        let arg = parts.next();
        if arg.is_none() {
          return Err(());
        }
        let mut parts2 = arg.unwrap().split("\x1E");
        let arg2 = parts2.next();
        if arg2.is_none() {
          return Err(());
        }
        let mut w_tuple: (usize, String) = Default::default();
        let mut w_vec = Vec::new();
        for (i, a) in arg2.unwrap().split("\x1F").enumerate() {
          if i % 2 == 0 {
            if let Ok(n) = a.parse() {
              w_tuple.0 = n;
            }
          } else {
            w_tuple.1 = a.to_string();
            w_vec.push(w_tuple.clone());
          }
        }
        let arg2 = parts2.next();
        if arg2.is_none() {
          return Err(());
        }
        if let Ok(n) = arg2.unwrap().parse() {
          return Ok(WindowMessage::Info(InfoType::WindowsInWorkspace(w_vec, n)));
        } else {
          return Err(());
        }
      },
      "Focus" => Ok(WindowMessage::Focus),
      "Unfocus" => Ok(WindowMessage::Unfocus),
      "FocusClick" => Ok(WindowMessage::FocusClick),
      "ChangeDimensions" => {
        let arg = parts.next();
        if arg.is_none() {
          return Err(());
        }
        let d = get_two_array(arg.unwrap())?;
        Ok(WindowMessage::ChangeDimensions(d))
      },
      "Touch" => {
        let arg = parts.next();
        if arg.is_none() {
          return Err(());
        }
        let mut parts2 = arg.unwrap().split("\x1E");
        let arg2 = parts2.next();
        if arg2.is_none() {
          return Err(());
        }
        let u1 = arg2.unwrap().parse();
        let arg2 = parts2.next();
        if u1.is_err() || arg2.is_none() {
          return Err(());
        }
        let u2 = arg2.unwrap().parse();
        if u2.is_err() {
          return Err(());
        }
        Ok(WindowMessage::Touch(u1.unwrap(), u2.unwrap()))
      },
      _ => Err(()),
    }
  }
}

#[test]
fn window_message_serialize_deserialize() {
  for wm in [
    WindowMessage::Init([1000, 1001]),
    WindowMessage::KeyPress(KeyPress { key: 'a' }),
    WindowMessage::KeyPress(KeyPress { key: '/' }),
    WindowMessage::KeyPress(KeyPress { key: '𐘂' }),
    WindowMessage::CtrlKeyPress(KeyPress { key: ';' }),
    WindowMessage::Shortcut(ShortcutType::StartMenu),
    WindowMessage::Shortcut(ShortcutType::MoveWindowToWorkspace(7)),
    WindowMessage::Shortcut(ShortcutType::ClipboardPaste("105/20 Azumanga".to_string())),
    WindowMessage::Info(InfoType::WindowsInWorkspace(vec![(1, "Terminal".to_string()), (2, "Minesweeper".to_string()), (12, "Test Test".to_string())], 5)),
    WindowMessage::Focus,
    WindowMessage::Unfocus,
    WindowMessage::FocusClick,
    WindowMessage::ChangeDimensions([999, 250]),
    WindowMessage::Touch(12, 247),
  ] {
    let serialized = wm.serialize();
    assert!(serialized == WindowMessage::deserialize(&serialized).unwrap().serialize());
  }
}


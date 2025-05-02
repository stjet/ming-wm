use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::io::{ stdin, stdout, Write };
use std::process::exit;
use std::env;

use linux::fb::Framebuffer;
use linux::raw::RawStdout;
use linux::keys::{ RawStdin, Key };
use linux::input::{ Input, EventType };
use wm::framebuffer::{ FramebufferWriter, FramebufferInfo };
use wm::window_manager::WindowManager;

use ming_wm_lib::window_manager_types::KeyChar;
use ming_wm_lib::messages::*;

include!(concat!(env!("OUT_DIR"), "/password.rs"));

const CLEAR_ALL: &'static str = "\x1b[2J";
const HIDE_CURSOR: &'static str = "\x1b[?25l";
const SHOW_CURSOR: &'static str = "\x1b[?25h";

//use Linear A for escape, backspace, enter, arrow keys
//Linear A used only internally in onscreen keyboard: 𐘎 is alt, 𐘧 is switch board, 𐘾 is ctrl
fn key_to_char(key: Key) -> Option<KeyChar> {
  match key {
    Key::Char('\n') => Some(KeyChar::Press('𐘂')),
    Key::Char(c) => Some(KeyChar::Press(c)),
    Key::Alt(c) => Some(KeyChar::Alt(c)),
    Key::Ctrl(c) => Some(KeyChar::Ctrl(c)),
    Key::Backspace => Some(KeyChar::Press('𐘁')),
    Key::Esc => Some(KeyChar::Press('𐘃')),
    Key::ArrowUp => Some(KeyChar::Press('𐙘')),
    Key::ArrowDown => Some(KeyChar::Press('𐘞')),
    Key::ArrowLeft => Some(KeyChar::Press('𐙣')),
    Key::ArrowRight => Some(KeyChar::Press('𐙥')),
    _ => None,
  }
}

pub enum ThreadMessage {
  KeyChar(KeyChar),
  Touch(usize, usize),
  Clear,
  Exit,
}

fn init(framebuffer: Framebuffer, framebuffer_info: FramebufferInfo) {
  let args: Vec<_> = env::args().collect();

  let rotate = args.contains(&"rotate".to_string());

  let framebuffer_info = if rotate {
    FramebufferInfo {
      byte_len: framebuffer_info.byte_len,
      width: framebuffer_info.height,
      height: framebuffer_info.width,
      bytes_per_pixel: framebuffer_info.bytes_per_pixel,
      stride: framebuffer_info.height,
      old_stride: Some(framebuffer_info.stride),
    }
  } else {
    framebuffer_info
  };

  let dimensions = [framebuffer_info.width, framebuffer_info.height];
  
  let grayscale = args.contains(&"grayscale".to_string()) || args.contains(&"greyscale".to_string());

  let mut writer: FramebufferWriter = FramebufferWriter::new(grayscale);

  writer.init(framebuffer_info.clone());

  let mut wm: WindowManager = WindowManager::new(writer, framebuffer, dimensions, rotate, grayscale, PASSWORD_HASH);

  let mut stdout = RawStdout::new(stdout());
  stdout.enter_raw_mode().unwrap();

  write!(stdout.stdout, "{}", CLEAR_ALL).unwrap();

  write!(stdout.stdout, "{}", HIDE_CURSOR).unwrap();

  stdout.stdout.flush().unwrap();

  wm.draw(None, false);

  let (tx, rx) = mpsc::channel();

  let tx1 = tx.clone();

  //read key presses
  thread::spawn(move || {
    let stdin = RawStdin::new(stdin());
    for c in stdin {
      if let Some(kc) = key_to_char(c) {
        //do not allow exit when locked unless debugging
        //if kc == KeyChar::Alt('E') {
        if kc == KeyChar::Alt('E') {
          tx.send(ThreadMessage::Exit).unwrap();
        } else {
          tx.send(ThreadMessage::KeyChar(kc)).unwrap();
        }
      }
      thread::sleep(Duration::from_millis(1));
    }
  });

  let touch = args.contains(&"touch".to_string());

  //read touchscreen presses (hopefully)
  thread::spawn(move || {
    //spawn evtest, parse it for touch coords
    if touch {
      let mut events = Input::new("/dev/input/by-path/first-touchscreen").unwrap(); //panics in threads don't matter in this case
      let mut x: Option<usize> = None;
      let mut y: Option<usize> = None;
      loop {
        let event = events.next();
        if let Some(event) = event {
          //ABS_X = 0, ABS_Y = 1
          if event.type_ == EventType::EV_ABS && (event.code == 0 || event.code == 1) {
            if event.code == 0 {
              x = Some(event.value as usize); //event.value is u16 so this should be fine. unless usize is u8, lmao
            } else {
              y = Some(event.value as usize);
            }
            if x.is_some() && y.is_some() {
              let (x2, y2) = if rotate {
                (dimensions[0] - y.unwrap(), x.unwrap())
              } else {
                (x.unwrap(), y.unwrap())
              };
              //top right, clear
              //useful sometimes, I think.
              if x2 > dimensions[0] - 100 && y2 < 100 {
                tx1.send(ThreadMessage::Clear).unwrap();
              }
              println!(" "); //without any stdout, on my phone, for some reason the framebuffer doesn't get redrawn to the screen
              tx1.send(ThreadMessage::Touch(x2, y2)).unwrap();
              x = None;
              y = None;
            }
          }
        }
        thread::sleep(Duration::from_millis(1));
      }
    }
  });
  if touch {
    //opens osk
    wm.handle_message(WindowManagerMessage::Touch(1, 1));
  }
  
  for message in rx {
    match message {
      ThreadMessage::KeyChar(kc) => wm.handle_message(WindowManagerMessage::KeyChar(kc.clone())),
      ThreadMessage::Touch(x, y) => wm.handle_message(WindowManagerMessage::Touch(x, y)),
      ThreadMessage::Clear => {
        write!(stdout.stdout, "{}", CLEAR_ALL).unwrap();
        stdout.stdout.flush().unwrap();
      },
      ThreadMessage::Exit => {
        if !wm.locked {
          write!(stdout.stdout, "{}", SHOW_CURSOR).unwrap();
          stdout.exit_raw_mode().unwrap();
          exit(0);
        }
      },
    };
  }
}

fn main() {
  let fb = Framebuffer::open("/dev/fb0").unwrap();
  let bytes_per_pixel = (fb.var_screen_info.bits_per_pixel as usize) / 8;
  let fb_info = FramebufferInfo {
    byte_len: (fb.var_screen_info.yres_virtual * fb.fix_screen_info.line_length) as usize,
    width: fb.var_screen_info.xres_virtual as usize,
    height: fb.var_screen_info.yres_virtual as usize,
    bytes_per_pixel,
    stride: fb.fix_screen_info.line_length as usize / bytes_per_pixel,
    old_stride: None,
  };

  init(fb, fb_info);
}


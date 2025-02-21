use std::process::{ Command, Stdio };
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::io::{ stdin, stdout, BufReader, BufRead, Write };
use std::process::exit;
use std::env;

use linux_framebuffer::Framebuffer;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{ clear, cursor };

use ming_wm::framebuffer::{ FramebufferWriter, FramebufferInfo };
use ming_wm::window_manager::{ WindowManager, KeyChar };
use ming_wm::utils::key_to_char;
use ming_wm::messages::*;

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

  let mut wm: WindowManager = WindowManager::new(writer, framebuffer, dimensions, rotate, grayscale);

  let mut stdout = stdout().into_raw_mode().unwrap();

  write!(stdout, "{}", clear::All).unwrap();

  write!(stdout, "{}", cursor::Hide).unwrap();

  stdout.flush().unwrap();

  wm.draw(None, false);

  let (tx, rx) = mpsc::channel();

  let tx1 = tx.clone();

  //read key presses
  thread::spawn(move || {
    let stdin = stdin().lock();
    for c in stdin.keys() {
      if let Some(kc) = key_to_char(c.unwrap()) {
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
      let mut evtest = Command::new("evtest").arg("/dev/input/by-path/first-touchscreen").stdout(Stdio::piped()).spawn().unwrap();
      let reader = BufReader::new(evtest.stdout.as_mut().unwrap());
      let mut x: Option<usize> = None;
      let mut y: Option<usize> = None;
      for line in reader.lines() {
        let line = line.unwrap();
        if line.contains(&"ABS_X), value ") || line.contains(&"ABS_Y), value ") {
          let value: Vec<_> = line.split("), value ").collect();
          let value = value[value.len() - 1].parse::<usize>().unwrap();
          if line.contains(&"ABS_X") {
            x = Some(value);
          } else {
            y = Some(value);
          }
          if x.is_some() && y.is_some() {
            let (x2, y2) = if rotate {
              (y.unwrap(), dimensions[0] - x.unwrap())
            } else {
              (x.unwrap(), y.unwrap())
            };
            //top right, clear
            //useful sometimes, I think.
            if x2 > dimensions[0] - 100 && y2 < 100 {
              tx1.send(ThreadMessage::Clear).unwrap();
            }
            tx1.send(ThreadMessage::Touch(x2, y2)).unwrap();
            x = None;
            y = None;
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
        write!(stdout, "{}", clear::All).unwrap();
        stdout.flush().unwrap();
      },
      ThreadMessage::Exit => {
        if !wm.locked {
          write!(stdout, "{}", cursor::Show).unwrap();
          stdout.suspend_raw_mode().unwrap();
          exit(0);
        }
      },
    };
  }
}

fn main() {
  let fb = Framebuffer::new("/dev/fb0").unwrap();
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


use std::io::{ Read, Stdin };
use std::sync::mpsc::{ channel, Receiver };
use std::thread;

//includes a section on reading keys
//https://viewsourcecode.org/snaptoken/kilo/02.enteringRawMode.html

const ALPHABET: [char; 26] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];

pub enum Key {
  Char(char),
  Alt(char),
  Ctrl(char),
  Backspace,
  Esc,
  ArrowUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  Other(u8), //we don't get about anything else, lmao
}

pub struct RawStdin {
  //bytes: Peekable<Bytes<StdinLock<'a>>>,
  receiver: Receiver<u8>,
}

impl RawStdin {
  pub fn new(stdin: Stdin) -> Self {
    let (sender, receiver) = channel();
    thread::spawn(move || {
      let bytes = stdin.lock().bytes();
      for b in bytes {
        sender.send(b.unwrap()).unwrap();
      }
    });
    RawStdin {
      //bytes: stdin.lock().bytes().peekable(),
      receiver,
    }
  }
}

impl Iterator for RawStdin {
  type Item = Key;

  fn next(&mut self) -> Option<Self::Item> {
    let first = self.receiver.recv().unwrap();
    Some(match first {
      1..=26 => {
        //ctrl
        if first == 9 {
          Key::Char('\t')
        } else if first == 13 {
          //ctrl+m and enter give the same thing
          Key::Char('\n')
        } else {
          Key::Ctrl(ALPHABET[first as usize - 1])
        }
      },
      27 => {
        //escape sequence
        //not handling escape sequences that are 3+ bytes is probably going to come back to bite us
        let n = self.receiver.try_recv();
        if let Ok(b'[') = n {
          let n = self.receiver.recv().unwrap();
          match n {
            b'A' => Key::ArrowUp,
            b'B' => Key::ArrowDown,
            b'C' => Key::ArrowRight,
            b'D' => Key::ArrowLeft,
            _ => Key::Other(n),
          }
        } else if let Ok(n) = n {
          //Alt+<char> sends Esc+<char>
          Key::Alt(char::from(n))
        } else {
          Key::Esc
        }
      },
      127 => {
        Key::Backspace
      },
      _ => {
        Key::Char(char::from(first))
      },
    })
  }
}

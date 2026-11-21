use std::io::Stdout;
use std::mem::zeroed;
use std::os::fd::AsRawFd;

use libc::{ cfmakeraw, c_int, tcgetattr, tcsetattr, termios, TCSAFLUSH };

//https://viewsourcecode.org/snaptoken/kilo/02.enteringRawMode.html
//on TCSAFLUSH: "The TCSAFLUSH argument specifies when to apply the change: in this case, it waits for all pending output to be written to the terminal, and also discards any input that hasn't been read."

//https://www.man7.org/linux/man-pages/man3/termios.3.html
//(use cfmakeraw instead doing all those bitwise stuff manually)

//enter and exit tty raw mode

pub struct RawStdout {
  pub stdout: Stdout,
  old_termios: termios,
}

impl RawStdout {
  pub fn new(stdout: Stdout) -> Self {
    RawStdout {
      stdout,
      old_termios: unsafe { zeroed() },
    }
  }

  pub fn get_termios(raw_fd: c_int) -> Result<termios, ()> {
    let mut termios_struct: termios = unsafe { zeroed() };
    let result = unsafe {
      tcgetattr(raw_fd, &mut termios_struct)
    };
    if result != -1 {
      Ok(termios_struct)
    } else {
      Err(())
    }
  }

  pub fn enter_raw_mode(&mut self) -> Result<(), ()> {
    let raw_fd = self.stdout.as_raw_fd();
    let mut termios_struct = Self::get_termios(raw_fd)?;
    self.old_termios = termios_struct;
    let result = unsafe {
      cfmakeraw(&mut termios_struct);
      tcsetattr(raw_fd, TCSAFLUSH, &mut termios_struct)
    };
    if result != -1 {
      Ok(())
    } else {
      Err(())
    }
  }

  pub fn exit_raw_mode(&mut self) -> Result<(), ()> {
    let result = unsafe {
      tcsetattr(self.stdout.as_raw_fd(), TCSAFLUSH, &mut self.old_termios)
    };
    if result != -1 {
      Ok(())
    } else {
      Err(())
    }
  }
}

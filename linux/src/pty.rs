use std::ptr;
use std::process::{ Command, Child };
use std::fs::File;
use std::os::fd::{ OwnedFd, FromRawFd };

use libc::openpty;

//basically the master and slave are linked? slave behaves just like normal terminal
//I don't totally get it, I guess just attach the command's stdout and stderr to ptyslave for reading?

pub struct PtyMaster {
  pub file: File,
}

impl PtyMaster {
  pub fn new(fd: OwnedFd) -> Self {
    Self {
      file: File::from(fd),
    }
  }
}

pub struct PtySlave {
  pub file: File,
  fd: OwnedFd,
}

impl PtySlave {
  pub fn new(fd: OwnedFd) -> Self {
    Self {
      file: File::from(fd.try_clone().unwrap()),
      fd,
    }
  }

  //assume stdin is piped
  pub fn attach_and_spawn(&self, command: &mut Command) -> std::io::Result<Child> {
    command.stdout(self.fd.try_clone().unwrap());
    command.stderr(self.fd.try_clone().unwrap());
    command.spawn()
  }
}

pub fn open_pty() -> Result<(PtyMaster, PtySlave), ()> {
  let mut master_fd = 0;
  let mut slave_fd = 0;
  let result = unsafe { openpty(&mut master_fd, &mut slave_fd, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
  if result == -1 {
    Err(())
  } else {
    let master_fd = unsafe { OwnedFd::from_raw_fd(master_fd) };
    let slave_fd = unsafe { OwnedFd::from_raw_fd(slave_fd) };
    Ok((PtyMaster::new(master_fd), PtySlave::new(slave_fd)))
  }
}

use std::fs::{ File, OpenOptions };
use std::os::fd::AsRawFd;
use std::ptr;

use libc::{ ioctl, mmap, munmap, c_uint, c_int };

//man.openbsd.org/wsdisplay
//WIP

const WSDISPLAYIO_SMODE: c_uint = 76;
const WSDISPLAYIO_MODE_DUMBFB: c_uint = 2;

const WSDISPLAYIO_GINFO: c_uint = 65;
const WSDISPLAYIO_LINEBYTES: c_uint = 95;

#[derive(Default)]
#[repr(C)]
pub struct WSDISPLAY_FBINFO {
  pub height: c_uint,
  pub width: c_uint,
  pub depth: c_uint,
  pub cmsize: c_uint,
}

pub struct Framebuffer {
  pointer: *mut libc::c_void,
  pub info: WSDISPLAY_FBINFO,
  pub line_bytes: usize,
  size: usize,
}

impl Framebuffer {
  pub fn open(path: &str) -> Result<Self, &str> {
    let file = Framebuffer::open_file(path).expect("failed to open_file");
    //set mode
    let result = unsafe {
      ioctl(file.as_raw_fd(), WSDISPLAYIO_SMODE.try_into().unwrap(), WSDISPLAYIO_MODE_DUMBFB)
    };
    if result == -1 {
      return Err("Couldn't set mode to DUMBFB");
    }
    //get fb info and line bytes
    let info = Self::get_fbinfo(file.as_raw_fd()).expect("Couldn't get fb info");
    let line_bytes = Self::get_line_bytes(file.as_raw_fd()).expect("Couldn't get bytes per line") as usize;
    //then mmap or something
    let size = info.height as usize * line_bytes * (info.depth as usize / 8);
    let pointer = unsafe {
      mmap(ptr::null_mut(), size.try_into().unwrap(), libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, file.as_raw_fd(), 0)
    };
    if pointer == libc::MAP_FAILED {
      return Err("map failed");
    }
    Ok(Self {
      pointer,
      info,
      line_bytes,
      size,
    })
  }
  
  fn open_file(path: &str) -> Result<File, ()> {
    OpenOptions::new().read(true).write(true).open(path).map_err(|_| ())
  }
  
  fn get_fbinfo(raw_fd: c_int) -> Result<WSDISPLAY_FBINFO, ()> {
    let mut fi: WSDISPLAY_FBINFO = Default::default();
    //WSDISPLAYIO_GINFO
    let result = unsafe {
      ioctl(raw_fd, WSDISPLAYIO_GINFO.try_into().unwrap(), &mut fi)
    };
    if result != -1 {
      Ok(fi)
    } else {
      Err(())
    }
  }

  fn get_line_bytes(raw_fd: c_int) -> Result<c_uint, ()> {
    let mut lb: c_uint = Default::default();
    //now WSDISPLAYIO_LINEBYTES
    let result = unsafe {
      ioctl(raw_fd, WSDISPLAYIO_LINEBYTES.try_into().unwrap(), &mut lb)
    };
    if result != -1 {
      Ok(lb)
    } else {
      Err(())
    }
  }
  
  pub fn write_frame(&mut self, frame: &[u8]) {
    unsafe {
      ptr::copy_nonoverlapping(frame.as_ptr(), self.pointer as *mut u8, frame.len());
    };
  }
}

impl Drop for Framebuffer {
  fn drop(&mut self) {
    unsafe {
      munmap(self.pointer, self.size);
      //change mode back to text?
      //
    }
  }
}


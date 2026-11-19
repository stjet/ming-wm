use std::fs::{ File, OpenOptions };
use std::os::fd::AsRawFd;
use std::ptr;

use std::io::Write;

use libc::{ ioctl, mmap, munmap, c_ulong, c_uint, c_int, _IOW, _IOR };

//man.openbsd.org/wsdisplay
//https://github.com/openbsd/src/blob/master/sys/dev/wscons/wsconsio.h

const WSDISPLAYIO_SMODE: c_ulong = _IOW::<c_uint>('W' as c_ulong, 76);
const WSDISPLAYIO_MODE_EMUL: c_uint = 0;
const WSDISPLAYIO_MODE_DUMBFB: c_uint = 2;

#[derive(Default, Debug)]
#[repr(C)]
pub struct WSDISPLAY_FBINFO {
  pub height: c_uint, //in px
  pub width: c_uint, //in px
  pub depth: c_uint, //in bits
  pub stride: c_uint, //in bytes
  pub offset: c_uint, //in bytes
  pub cmsize: c_uint, //in # of entries
}

const WSDISPLAYIO_GINFO: c_ulong = _IOR::<WSDISPLAY_FBINFO>('W' as c_ulong, 65);
//const WSDISPLAYIO_LINEBYTES: c_ulong = _IOR::<c_uint>('W' as c_ulong, 95);

pub fn log(message: &str) {
  let _ = writeln!(OpenOptions::new().append(true).create(true).open("/home/user/.local/share/ming-wm/logs.txt").unwrap(), "{}", message);
}

pub struct Framebuffer {
  pointer: *mut libc::c_void,
  pub info: WSDISPLAY_FBINFO,
  //pub line_bytes: usize,
  size: usize,
  raw_fd: c_int,
}

impl Framebuffer {
  pub fn open(path: &str) -> Result<Self, &str> {
    let file = Framebuffer::open_file(path).expect("failed to open_file");
    //set mode
    let result = unsafe {
      ioctl(file.as_raw_fd(), WSDISPLAYIO_SMODE.try_into().unwrap(), &WSDISPLAYIO_MODE_DUMBFB)
    };
    if result == -1 {
      return Err("Couldn't set mode to DUMBFB");
    }
    //get fb info and line bytes
    let info = Self::get_fbinfo(file.as_raw_fd()).expect("Couldn't get fb info");
    //let line_bytes = Self::get_line_bytes(file.as_raw_fd()).expect("Couldn't get bytes per line") as usize;
    //then mmap or something
    let size = info.height as usize * info.stride as usize;
    let pointer = unsafe {
      mmap(ptr::null_mut(), size.try_into().unwrap(), libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, file.as_raw_fd(), 0)
    };
    if pointer == libc::MAP_FAILED {
      return Err("map failed");
    }
    Ok(Self {
      pointer,
      info,
      //line_bytes,
      size,
      raw_fd: file.as_raw_fd(),
    })
  }
  
  fn open_file(path: &str) -> Result<File, ()> {
    OpenOptions::new().read(true).write(true).open(path).map_err(|_| ())
  }
  
  fn get_fbinfo(raw_fd: c_int) -> Result<WSDISPLAY_FBINFO, ()> {
    let mut fi: WSDISPLAY_FBINFO = Default::default();
    let result = unsafe {
      ioctl(raw_fd, WSDISPLAYIO_GINFO.try_into().unwrap(), &mut fi)
    };
    if result != -1 {
      Ok(fi)
    } else {
      Err(())
    }
  }

  /*fn get_line_bytes(raw_fd: c_int) -> Result<c_uint, ()> {
    let mut lb: c_uint = Default::default();
    let result = unsafe {
      ioctl(raw_fd, WSDISPLAYIO_LINEBYTES.try_into().unwrap(), &mut lb)
    };
    if result != -1 {
      Ok(lb)
    } else {
      Err(())
    }
  }*/
  
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
      ioctl(self.raw_fd, WSDISPLAYIO_SMODE.try_into().unwrap(), &WSDISPLAYIO_MODE_EMUL);
    }
  }
}

use std::fs::{ File, OpenOptions };
use std::os::fd::AsRawFd;
use std::ptr;

use libc::{ ioctl, mmap, munmap, c_ulong, c_int };

//https://stackoverflow.com/a/75402838

//https://github.com/torvalds/linux/blob/master/include/uapi/linux/fb.h
pub const FBIOGET_VSCREENINFO: c_ulong = 0x4600;
pub const FBIOGET_FSCREENINFO: c_ulong = 0x4602;

//https://www.kernel.org/doc/html/latest/fb/api.html

#[derive(Default)]
#[repr(C)]
pub struct FB_BITFIELD {
  offset: u32,
  length: u32,
  msb_right: u32,
}

#[derive(Default)]
#[repr(C)]
pub struct FB_VAR_SCREENINFO {
  pub xres: u32,
  pub yres: u32,
  pub xres_virtual: u32,
  pub yres_virtual: u32,
  pub xoffset: u32,
  pub yoffset: u32,
  pub bits_per_pixel: u32,
  pub grayscale: u32,
  pub red: FB_BITFIELD,
  pub green: FB_BITFIELD,
  pub blue: FB_BITFIELD,
  pub transp: FB_BITFIELD,
  pub nonstd: u32,
  pub activate: u32,
  pub height: u32,
  pub width: u32,
  pub accel_flags: u32,
  pub pixclock: u32,
  pub left_margin: u32,
  pub right_margin: u32,
  pub upper_margin: u32,
  pub lower_margin: u32,
  pub hsync_len: u32,
  pub wsync_len: u32,
  pub sync: u32,
  pub vmode: u32,
  pub rotate: u32,
  pub colorspace: u32,
  pub reserved: [u32; 4],
}

#[derive(Default)]
#[repr(C)]
pub struct FB_FIX_SCREENINFO {
  pub id: [u8; 16],
  pub smem_start: usize,
  pub smem_len: u32,
  pub r#type: u32,
  pub type_aux: u32,
  pub visual: u32,
  pub xpanstep: u16,
  pub ypanstep: u16,
  pub ywrapstep: u16,
  pub line_length: u32,
  pub mmio_len: u32,
  pub accel: u32,
  pub capabilities: u16,
  pub reserved: [u16; 2],
}

pub struct Framebuffer {
  pointer: *mut libc::c_void,
  pub var_screen_info: FB_VAR_SCREENINFO,
  pub fix_screen_info: FB_FIX_SCREENINFO,
  size: usize,
}

impl Framebuffer {
  pub fn open(path: &str) -> Result<Self, ()> {
    let file = Framebuffer::open_file(path)?;
    let vi = Framebuffer::get_vscreeninfo(file.as_raw_fd())?;
    let fi = Framebuffer::get_fscreeninfo(file.as_raw_fd())?;
    //then mmap or something
    let size = vi.yres_virtual * fi.line_length * (vi.bits_per_pixel / 8);
    let pointer = unsafe {
      mmap(ptr::null_mut(), size.try_into().unwrap(), libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, file.as_raw_fd(), 0)
    };
    if pointer == libc::MAP_FAILED {
      return Err(());
    }
    Ok(Self {
      pointer,
      var_screen_info: vi,
      fix_screen_info: fi,
      size: size as usize,
    })
  }
  
  fn open_file(path: &str) -> Result<File, ()> {
    OpenOptions::new().read(true).write(true).open(path).map_err(|_| ())
  }
  
  fn get_vscreeninfo(raw_fd: c_int) -> Result<FB_VAR_SCREENINFO, ()> {
    let mut vi: FB_VAR_SCREENINFO = Default::default();
    let result = unsafe {
      ioctl(raw_fd, FBIOGET_VSCREENINFO, &mut vi)
    };
    if result != -1 {
      Ok(vi)
    } else {
      Err(())
    }
  }
  
  fn get_fscreeninfo(raw_fd: c_int) -> Result<FB_FIX_SCREENINFO, ()> {
    let mut fi: FB_FIX_SCREENINFO = Default::default();
    let result = unsafe {
      ioctl(raw_fd, FBIOGET_FSCREENINFO, &mut fi)
    };
    if result != -1 {
      Ok(fi)
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
    }
  }
}


use linux_framebuffer::Framebuffer;

mod framebuffer;
use framebuffer::FramebufferInfo;

mod window_manager;
use window_manager::init;

mod window_likes;

mod components;

mod themes;

mod keyboard;

mod messages;

mod fs;

mod utils;

fn main() {
  let mut fb = Framebuffer::new("/dev/fb0").unwrap();
  let bytes_per_pixel = (fb.var_screen_info.bits_per_pixel as usize) / 8;
  let fb_info = FramebufferInfo {
    byte_len: (fb.var_screen_info.yres_virtual * fb.fix_screen_info.line_length) as usize,
    width: fb.var_screen_info.xres_virtual as usize,
    height: fb.var_screen_info.yres_virtual as usize,
    bytes_per_pixel,
    stride: fb.fix_screen_info.line_length as usize / bytes_per_pixel,
  };

  init(fb, fb_info);
}


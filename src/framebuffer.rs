use std::vec::Vec;
//use core::ptr;

use bmp_rust::bmp::BMP;

use crate::fs::get_font_char_from_fonts;

pub type Point = [usize; 2]; //x, y
pub type Dimensions = [usize; 2]; //width, height
pub type RGBColor = [u8; 3]; //rgb

type FontChar = (char, Vec<Vec<u8>>, u8);

fn color_with_alpha(color: RGBColor, bg_color: RGBColor, alpha: u8) -> RGBColor {
  /*let factor: f32 = alpha as f32 / 255.0;
  [
    (bg_color[0] as f32 * (1.0 - factor)) as u8 + (color[0] as f32 * factor) as u8,
    (bg_color[1] as f32 * (1.0 - factor)) as u8 + (color[1] as f32 * factor) as u8,
    (bg_color[2] as f32 * (1.0 - factor)) as u8 + (color[2] as f32 * factor) as u8,
  ]*/
  //255 * 255 < max(u16)
  if alpha == 255 {
    color
  } else {
    let alpha = alpha as u16;
    [
      (bg_color[0] as u16 * (255 - alpha) / 255) as u8 + (color[0] as u16 * alpha / 255) as u8,
      (bg_color[1] as u16 * (255 - alpha) / 255) as u8 + (color[1] as u16 * alpha / 255) as u8,
      (bg_color[2] as u16 * (255 - alpha) / 255) as u8 + (color[2] as u16 * alpha / 255) as u8,
    ]
  }
}

fn color_to_grayscale(color: RGBColor) -> RGBColor {
  //0.3, 0.6, 0.1 weighting
  let gray = color[0] / 10 * 3 + color[1] / 10 * 6 + color[2] / 10;
  [gray; 3]
}

#[derive(Clone, Default)]
pub struct FramebufferInfo {
  pub byte_len: usize,
  pub width: usize,
  pub height: usize,
  pub bytes_per_pixel: usize,
  pub stride: usize,
  pub old_stride: Option<usize>, //used/set only when rotate is true
}

//currently doesn't check if writing onto next line accidentally
pub struct FramebufferWriter {
  info: FramebufferInfo,
  buffer: Vec<u8>,
  saved_buffer: Option<Vec<u8>>,
  rotate_buffer: Option<Vec<u8>>,
  grayscale: bool,
}

impl FramebufferWriter {
  pub fn new(grayscale: bool) -> Self {
    Self {
      info: Default::default(),
      buffer: Vec::new(),
      saved_buffer: None,
      rotate_buffer: None,
      grayscale,
    }
  }

  pub fn init(&mut self, info: FramebufferInfo) {
    self.info = info;
    self.buffer = vec![0; self.info.byte_len];
  }
  
  pub fn get_info(&self) -> FramebufferInfo {
    self.info.clone()
  }
  
  pub fn get_buffer(&mut self) -> &[u8] {
    &self.buffer
  }

  pub fn get_transposed_buffer(&mut self) -> &[u8] {
    let mut output_array = vec![255; self.info.byte_len];
    let row_bytes_len = self.info.stride * self.info.bytes_per_pixel;
    let row_bytes_len_transposed = self.info.old_stride.unwrap_or(self.info.height) * self.info.bytes_per_pixel;
    for y in 0..self.info.height {
      for x in 0..self.info.width {
        for i in 0..self.info.bytes_per_pixel {
          output_array[(self.info.width - x - 1) * row_bytes_len_transposed + y * self.info.bytes_per_pixel + i] = self.buffer[y * row_bytes_len + x * self.info.bytes_per_pixel + i];
        }
      }
    }
    self.rotate_buffer = Some(output_array);
    &self.rotate_buffer.as_ref().unwrap()
  }

  pub fn save_buffer(&mut self) {
    self.saved_buffer = Some(self.buffer.clone());
  }

  pub fn write_saved_buffer_to_raw(&mut self) {
    self.buffer[..]
      .copy_from_slice(&self.saved_buffer.as_ref().unwrap()[..]);
  }

  fn _draw_pixel(&mut self, start_pos: usize, color: RGBColor) {
    let color = [color[2], color[1], color[0]];
    let color = if self.grayscale { color_to_grayscale(color) } else { color };
    self.buffer[start_pos..(start_pos + 3)]
      .copy_from_slice(&color[..]);
  }

  fn _draw_line(&mut self, start_pos: usize, bytes: &[u8]) {
    self.buffer[start_pos..(start_pos + bytes.len())]
      .copy_from_slice(bytes);
  }

  pub fn draw_buffer(&mut self, top_left: Point, height: usize, bytes_per_line: usize, bytes: &[u8]) {
    //for our framebuffer
    let mut start_pos = (top_left[1] * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
    //of the buffer we want to draw on
    let mut start = 0;
    for _y in 0..height {
      self.buffer[start_pos..(start_pos + bytes_per_line)]
        .copy_from_slice(&bytes[start..(start + bytes_per_line)]);
      //let _ = unsafe { ptr::read_volatile(&self.buffer[start_pos]) };
      start += bytes_per_line;
      start_pos += self.info.stride * self.info.bytes_per_pixel;
    }
  }

  pub fn draw_char(&mut self, top_left: Point, char_info: &FontChar, color: RGBColor, bg_color: RGBColor) {
    let mut start_pos;
    for row in 0..char_info.1.len() {
      //char_info.2 is vertical offset
      start_pos = ((top_left[1] + row + char_info.2 as usize) * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
      for col in &char_info.1[row] {
        if col > &0 {
          if start_pos < self.info.byte_len {
            self._draw_pixel(start_pos, color_with_alpha(color, bg_color, *col));
          }
        }
        start_pos += self.info.bytes_per_pixel;
      }
    }
  }

  //dots

  pub fn draw_pixel(&mut self, point: Point, color: RGBColor) {
    let start_pos = (point[1] * self.info.stride + point[0]) * self.info.bytes_per_pixel;
    self._draw_pixel(start_pos, color);
  }
  
  //shapes

  pub fn draw_rect(&mut self, top_left: Point, dimensions: Dimensions, color: RGBColor) {
    let color = if self.grayscale { color_to_grayscale(color) } else { color };
    let line_bytes = if self.info.bytes_per_pixel > 3 {
      [color[2], color[1], color[0], 255].repeat(dimensions[0])
    } else {
      color.repeat(dimensions[0])
    };
    let mut start_pos = (top_left[1] * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
    for _row in 0..dimensions[1] {
      //use _draw_line instead for MUCH more efficiency
      self._draw_line(start_pos, &line_bytes[..]);
      start_pos += self.info.stride * self.info.bytes_per_pixel;
    }
  }

  //can optimise (?) by turning into lines and doing _draw_line instead?
  pub fn draw_circle(&mut self, centre: Point, radius: usize, color: RGBColor) {
    //x^2 + y^2 <= r^2
    for y in 0..radius {
      for x in 0..radius {
        if (x.pow(2) + y.pow(2)) <= radius.pow(2) {
          self.draw_pixel([centre[0] + x, centre[1] + y], color);
          self.draw_pixel([centre[0] - x, centre[1] + y], color);
          self.draw_pixel([centre[0] - x, centre[1] - y], color);
          self.draw_pixel([centre[0] + x, centre[1] - y], color);
        }
      }
    }
  }

  //direction is top to bottom
  pub fn draw_gradient(&mut self, top_left: Point, dimensions: Dimensions, start_color: RGBColor, end_color: RGBColor, steps: usize) {
    let delta_r = (end_color[0] as f32 - start_color[0] as f32) / steps as f32;
    let delta_g = (end_color[1] as f32 - start_color[1] as f32) / steps as f32;
    let delta_b = (end_color[2] as f32 - start_color[2] as f32) / steps as f32;
    let mut start_pos = (top_left[1] * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
    if steps <= dimensions[1] {
      //rounds down
      let mut y_per = dimensions[1] / steps;
      for s in 0..steps {
        let color;
        if s == steps - 1 {
          color = end_color;
          //the remaining lines are the last one
          y_per = dimensions[1] - (y_per * steps);
        } else {
          color = [(start_color[0] as f32 + (delta_r * s as f32)) as u8, (start_color[1] as f32 + (delta_g * s as f32)) as u8, (start_color[2] as f32 + (delta_b * s as f32)) as u8];
        };
        let color = if self.grayscale { color_to_grayscale(color) } else { color };
        let line_bytes = if self.info.bytes_per_pixel > 3 {
          [color[2], color[1], color[0], 255].repeat(dimensions[0])
        } else {
          color.repeat(dimensions[0])
        };
        for _y in 0..y_per {
          self._draw_line(start_pos, &line_bytes[..]);
          start_pos += self.info.stride * self.info.bytes_per_pixel;
        }
      }
    }
  }

  //text

  pub fn draw_text(&mut self, top_left: Point, fonts: Vec<String>, text: &str, color: RGBColor, bg_color: RGBColor, horiz_spacing: usize, mono_width: Option<u8>) {
    let mut top_left = top_left;
    //todo, config space
    for c in text.chars() {
      if c == ' ' {
        top_left[0] += mono_width.unwrap_or(5) as usize;
      } else {
        let char_info = get_font_char_from_fonts(&fonts, c);
        let char_width = char_info.1[0].len();
        let add_after: usize;
        if let Some(mono_width) = mono_width {
          let mono_width = mono_width as usize;
          if mono_width < char_width {
            add_after = mono_width;
          } else {
            let remainder = mono_width - char_width;
            top_left[0] += remainder / 2;
            add_after = remainder - remainder / 2 + char_width;
          };
        } else {
          add_after = char_width + horiz_spacing;
        }
        self.draw_char(top_left, &char_info, color, bg_color);
        top_left[0] += add_after;
      }
    }
  }

  //bmps

  //reverse is workaround for when my bmp lib returns rgba instead of bgra
  pub fn draw_bmp(&mut self, top_left: Point, path: String, reverse: bool) {
    let b = BMP::new_from_file(&path);
    let dib_header = b.get_dib_header().unwrap();
    let pixel_data = b.get_pixel_data().unwrap();
    let height = dib_header.height as usize;
    let width = dib_header.width as usize;
    let mut start_pos;
    for row in 0..height {
      start_pos = ((top_left[1] + row) * self.info.stride + top_left[0]) * self.info.bytes_per_pixel;
      for column in 0..width {
        let color = b.get_color_of_pixel_efficient(column, row, &dib_header, &pixel_data).unwrap();
        self._draw_pixel(start_pos, if reverse { [color[2], color[1], color[0]] } else { [color[0], color[1], color[2]] });
        start_pos += self.info.bytes_per_pixel;
      }
    }
  }
}


use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use crate::dirs;

#[derive(Clone)]
pub struct FontCharInfo {
  pub c: char,
  pub data: Vec<Vec<u8>>,
  pub top_offset: u8,
  pub height: usize,
  pub width: usize,
}

fn get_font_char(dir: &str, c: char) -> Option<FontCharInfo> {
  let c = if c == '/' { '𐘋' } else if c == '\\' { '𐚆' } else if c == '.' { '𐘅' } else { c };
  if let Ok(mut file) = File::open(dir.to_string() + "/" + &c.to_string() + ".alpha") {
    let mut ch: Vec<Vec<u8>> = Vec::new();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let lines: Vec<&str> = contents.split("\n").collect();
    for ln in lines.iter().skip(1) {
      //.unwrap_or(0) is important because zeroes are just empty
      ch.push(ln.replace(":", ",,,,").replace(";", ",,,").replace(".", ",,").split(",").map(|n| n.parse().unwrap_or(0)).collect());
    }
    return Some(FontCharInfo {
      c,
      top_offset: lines[0].parse().unwrap(),
      height: lines.len() - 1,
      width: ch[0].len(),
      data: ch,
     });
  }
  None
}

pub fn get_font_char_from_fonts(fonts: &[String], c: char) -> FontCharInfo {
  for font in fonts {
    let p = dirs::exe_dir(Some(&("ming_bmps/".to_string() + &font))).to_string_lossy().to_string();
    if let Some(font_char) = get_font_char(&p, c) {
      return font_char;
    }
  }
  let p = dirs::exe_dir(Some(&("ming_bmps/".to_string() + &fonts[0]))).to_string_lossy().to_string();
  //so a ? char should be in every font. otherwise will just return blank
  get_font_char(&p, '?').unwrap_or(FontCharInfo {
    c: '?',
    data: vec![vec![0]],
    top_offset: 0,
    height: 1,
    width: 1,
  })
}

pub struct MeasureInfo {
  pub height: usize,
  pub width: usize,
}

pub fn measure_text(fonts: &[String], text: &str, horiz_spacing: Option<usize>) -> MeasureInfo {
  let mut height = 0;
  let mut width = 0;
  for c in text.chars() {
    let i = get_font_char_from_fonts(fonts, c);
    let c_height = i.top_offset as usize + i.height;
    if c_height > height {
      height = c_height;
    }
    width += i.width + horiz_spacing.unwrap_or(1);
  }
  width -= horiz_spacing.unwrap_or(1);
  MeasureInfo {
    height,
    width,
  }
}

pub fn measure_text_with_cache(fc_getter: &mut CachedFontCharGetter, fonts: &[String], text: &str, horiz_spacing: Option<usize>) -> MeasureInfo {
  let mut height = 0;
  let mut width = 0;
  for c in text.chars() {
    let i = fc_getter.get(fonts, c);
    let c_height = i.top_offset as usize + i.height;
    if c_height > height {
      height = c_height;
    }
    width += i.width + horiz_spacing.unwrap_or(1);
  }
  width -= horiz_spacing.unwrap_or(1);
  MeasureInfo {
    height,
    width,
  }
}

#[derive(Default)]
pub struct CachedFontCharGetter {
  cache: HashMap<char, FontCharInfo>,
  cache_size: usize, //# of items cached
  pub max_cache_size: usize,
}

impl CachedFontCharGetter {
  pub fn new(max_cache_size: usize) -> Self {
    Self {
      max_cache_size,
      ..Default::default()
    }
  }

  pub fn get(&mut self, fonts: &[String], c: char) -> FontCharInfo {
    if let Some(cached) = self.cache.get(&c) {
      cached.clone()
    } else {
      let got = get_font_char_from_fonts(fonts, c);
      if self.cache_size == self.max_cache_size {
        self.cache_size = 0;
        self.cache = HashMap::new();
      }
      self.cache.insert(c, got.clone());
      self.cache_size += 1;
      got
    }
  }
}

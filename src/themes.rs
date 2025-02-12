use crate::framebuffer::RGBColor;

#[derive(PartialEq, Default)]
pub enum Themes {
  #[default]
  Standard,
  //
}

#[derive(Default)]
pub struct ThemeInfo {
  pub top: RGBColor,
  pub background: RGBColor,
  pub border_left_top: RGBColor,
  pub border_right_bottom: RGBColor,
  pub text: RGBColor,
  pub top_text: RGBColor,
  pub alt_background: RGBColor,
  pub alt_text: RGBColor,
  pub alt_secondary: RGBColor,
}

const THEME_INFOS: [(Themes, ThemeInfo); 1] = [
  (Themes::Standard, ThemeInfo {
    top: [0, 0, 128],
    background: [192, 192, 192],
    border_left_top: [255, 255, 255],
    border_right_bottom: [0, 0, 0],
    text: [0, 0, 0],
    top_text: [255, 255, 255],
    alt_background: [0, 0, 0],
    alt_text: [255, 255, 255],
    alt_secondary: [128, 128, 128],
    //
  }),
];

pub fn get_theme_info(theme: &Themes) -> Option<ThemeInfo> {
  for pair in THEME_INFOS {
    if &pair.0 == theme {
      return Some(pair.1);
    }
  }
  return None;
}


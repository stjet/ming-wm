use crate::framebuffer_types::RGBColor;

#[derive(PartialEq, Default)]
pub enum Themes {
  #[default]
  Standard,
  Night,
  Industrial,
  Forest,
  Royal,
  //Parchment,
}

impl std::str::FromStr for Themes {
  type Err = ();

  fn from_str(name: &str) -> Result<Self, Self::Err> {
    match name {
      "Standard" => Ok(Themes::Standard),
      "Night" => Ok(Themes::Night),
      "Industrial" => Ok(Themes::Industrial),
      "Forest" => Ok(Themes::Forest),
      "Royal" => Ok(Themes::Royal),
      _ => Err(()),
    }
  }
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

//besides standard, these themes aren't great, I know
const THEME_INFOS: [(Themes, ThemeInfo); 5] = [
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
  }),
  (Themes::Night, ThemeInfo {
    top: [0, 0, 0],
    background: [34, 34, 34],
    border_left_top: [239, 239, 239],
    border_right_bottom: [0, 0, 0],
    text: [239, 239, 239],
    top_text: [239, 239, 239],
    alt_background: [0, 0, 0],
    alt_text: [239, 239, 239],
    alt_secondary: [128, 128, 128],
  }),
  (Themes::Industrial, ThemeInfo {
    top: [40, 40, 40],
    background: [160, 160, 160],
    border_left_top: [255, 255, 255],
    border_right_bottom: [0, 0, 0],
    text: [0, 0, 0],
    top_text: [255, 255, 255],
    alt_background: [0, 0, 0],
    alt_text: [255, 255, 255],
    alt_secondary: [128, 128, 128],
  }),
  (Themes::Forest, ThemeInfo {
    top: [0, 128, 0],
    background: [192, 192, 192],
    border_left_top: [255, 255, 255],
    border_right_bottom: [0, 0, 0],
    text: [0, 0, 0],
    top_text: [255, 255, 255],
    alt_background: [0, 0, 0],
    alt_text: [255, 255, 255],
    alt_secondary: [128, 128, 128],
  }),
  (Themes::Royal, ThemeInfo {
    top: [128, 0, 128],
    background: [192, 192, 192],
    border_left_top: [255, 255, 255],
    border_right_bottom: [0, 0, 0],
    text: [0, 0, 0],
    top_text: [255, 255, 255],
    alt_background: [0, 0, 0],
    alt_text: [255, 255, 255],
    alt_secondary: [128, 128, 128],
  }),
  //
];

/// Window manager internal usage
pub fn get_theme_info(theme: &Themes) -> Option<ThemeInfo> {
  for pair in THEME_INFOS {
    if &pair.0 == theme {
      return Some(pair.1);
    }
  }
  None
}


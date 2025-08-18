use std::vec;
use std::vec::Vec;
use std::time::{ SystemTime, UNIX_EPOCH }; //for psuedo-randomness

use ming_wm_lib::framebuffer_types::Dimensions;
use ming_wm_lib::themes::ThemeInfo;
use ming_wm_lib::messages::{ WindowMessage, WindowMessageResponse, WindowManagerRequest };
use ming_wm_lib::window_manager_types::{ DrawInstructions, WindowLike, WindowLikeType };
use bitcoin_hashes::Sha512;

pub struct LockScreen {
  dimensions: Dimensions,
  input_password: String,
  password_hash: [u8; 64],
  lines: [String; 3],
}

impl WindowLike for LockScreen {
  fn handle_message(&mut self, message: WindowMessage) -> WindowMessageResponse {
    match message {
      WindowMessage::Init(dimensions) => {
        self.dimensions = dimensions;
        WindowMessageResponse::JustRedraw
      },
      WindowMessage::KeyPress(key_press) => {
        if key_press.is_enter() {
          //check password
          if Sha512::hash((self.input_password.clone() + "salt?sorrycryptographers!1!").as_bytes()).to_byte_array() == self.password_hash {
            WindowMessageResponse::Request(WindowManagerRequest::Unlock)
          } else {
            self.input_password = String::new();
            WindowMessageResponse::JustRedraw
          }
        } else if key_press.is_backspace() {
          let p_len = self.input_password.len();
          if p_len != 0 {
            self.input_password = self.input_password[..p_len - 1].to_string();
          }
          WindowMessageResponse::JustRedraw
        } else if key_press.is_regular() {
          self.input_password += &key_press.key.to_string();
          WindowMessageResponse::JustRedraw
        } else {
          WindowMessageResponse::DoNothing
        }
      },
      _ => WindowMessageResponse::DoNothing,
    }
  }

  fn draw(&self, _theme_info: &ThemeInfo) -> Vec<DrawInstructions> {
    vec![
      DrawInstructions::Rect([0, 0], self.dimensions, [0, 0, 0]),
      DrawInstructions::Text([4, 4], vec!["nimbus-roman".to_string()], self.lines[0].clone(), [255, 255, 255], [0, 0, 0], None, None),
      //He is my brother.
      DrawInstructions::Text([4, 4 + 16], vec!["nimbus-roman".to_string()], self.lines[1].clone(), [255, 255, 255], [0, 0, 0], None, None),
      //But I must kill him and keep strong to do it.
      DrawInstructions::Text([4, 4 + 16 * 2], vec!["nimbus-roman".to_string()], self.lines[2].clone(), [255, 255, 255], [0, 0, 0], None, None),
      DrawInstructions::Text([4, 4 + 16 * 3], vec!["nimbus-roman".to_string()], "Password: ".to_string(), [255, 255, 255], [0, 0, 0], None, None),
      DrawInstructions::Text([80, 4 + 16 * 3], vec!["nimbus-roman".to_string()], "*".repeat(self.input_password.len()), [255, 255, 255], [0, 0, 0], None, None),
    ]
  }
  
  //properties
  fn subtype(&self) -> WindowLikeType {
    WindowLikeType::LockScreen
  }

  fn ideal_dimensions(&self, dimensions: Dimensions) -> Dimensions {
    dimensions //fullscreen
  }
}

impl LockScreen {
  pub fn new(password_hash: [u8; 64]) -> Self {
    let possible_lines = [
      [
        "\"He took about forty pounds,\" the old man said aloud.".to_string(),
        "He took my harpoon too and all the rope, he thought, and now my fish bleeds again and there will be others.".to_string(),
        "He did not like to look at the fish anymore since it had been mutilated.".to_string()
      ],
      [
        "The bulldozer outside the kitchen window was quite a big one.".to_string(),
        "\"Yellow,\" he thought, and stomped off back to his bedroom to get dressed.".to_string(),
        "He stared at it.".to_string()
      ],
    ];
    let rand_index = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize % possible_lines.len();
    Self {
      dimensions: [0, 0],
      input_password: String::new(),
      password_hash,
      lines: possible_lines[rand_index].clone(),
    }
  }
}


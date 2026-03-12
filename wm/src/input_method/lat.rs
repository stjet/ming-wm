use ming_wm_lib::window_manager_types::InputMethod;
use ming_wm_lib::messages::KeyPress;

pub struct Lat {}

impl InputMethod for Lat {
  fn abbr(&self) -> [char; 3] {
    ['L', 'A', 'T']
  }

  fn internal_buffer(&self) -> String {
    String::new()
  }

  fn add_key_press(&mut self, kp: KeyPress) -> Option<char> {
    //everything goes
    Some(kp.key)
  }
}

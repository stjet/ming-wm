use std::collections::HashMap;

use ming_wm_lib::window_manager_types::InputMethod;
use ming_wm_lib::messages::KeyPress;

const TOK_CHARSET: [char; 14] = ['m', 'n', 'i', 'u', 'p', 't', 'k', 'e', 'o', 's', 'a', 'w', 'l', 'j'];

pub struct Tok {
  mapping: HashMap<String, char>,
  buffer: String,
}

impl InputMethod for Tok {
  fn abbr(&self) -> [char; 3] {
    ['T', 'O', 'K']
  }

  fn internal_buffer(&self) -> String {
    self.buffer.clone()
  }

  fn add_key_press(&mut self, kp: KeyPress) -> Option<char> {
    if kp.is_enter() || kp.is_backspace() {
      if self.buffer == String::new() {
        //^if no buffer, enter or backspace key should be passed through
        Some(kp.key)
      } else {
        if kp.is_enter() {
          if let Some(ret_c) = self.mapping.get(&self.buffer.to_lowercase()) {
            //^look up in table to see if buffer matches, if so, return
            self.buffer = String::new();
            return Some(*ret_c);
          }
        }
        //else, just clear buffer
        self.buffer = String::new();
        None
      }
    } else if TOK_CHARSET.contains(&kp.key) {
      self.buffer += &kp.key.to_string();
      None
    } else {
      //pass through stuff not in tp charset
      Some(kp.key)
    }
  }
}

impl Tok {
  pub fn new() -> Self {
    let mapping = HashMap::from([
      ("pali".to_string(), '󱥉'),
      ("weka".to_string(), '󱥶'),
      ("li".to_string(), '󱤧'),
      ("pona".to_string(), '󱥔'),
      ("tawa".to_string(), '󱥩'),
      ("mi".to_string(), '󱤴'),
      ("kulupu".to_string(), '󱤟'),
      ("pi".to_string(), '󱥍'),
      ("musi".to_string(), '󱤻'),
      ("sijelo".to_string(), '󱥛'),
      ("ni".to_string(), '󱥁'),
      ("palisa".to_string(), '󱥊'),
      ("a".to_string(), '󱤀'),
      ("akesi".to_string(), '󱤁'),
      ("ala".to_string(), '󱤂'),
      ("alasa".to_string(), '󱤃'),
      ("ali".to_string(), '󱤄'),
      ("ale".to_string(), '󱤄'),
      ("anpa".to_string(), '󱤅'),
      ("ante".to_string(), '󱤆'),
      ("anu".to_string(), '󱤇'),
      ("awen".to_string(), '󱤈'),
      ("e".to_string(), '󱤉'),
      ("en".to_string(), '󱤊'),
      ("esun".to_string(), '󱤋'),
      ("ijo".to_string(), '󱤌'),
      ("ike".to_string(), '󱤍'),
      ("ilo".to_string(), '󱤎'),
      ("insa".to_string(), '󱤏'),
      ("jaki".to_string(), '󱤐'),
      ("jan".to_string(), '󱤑'),
      ("jelo".to_string(), '󱤒'),
      ("jo".to_string(), '󱤓'),
      ("kala".to_string(), '󱤔'),
      ("kalama".to_string(), '󱤕'),
      ("kama".to_string(), '󱤖'),
      ("kasi".to_string(), '󱤗'),
      ("ken".to_string(), '󱤘'),
      ("kepeken".to_string(), '󱤙'),
      ("kili".to_string(), '󱤚'),
      ("kiwen".to_string(), '󱤛'),
      ("ko".to_string(), '󱤜'),
      ("kon".to_string(), '󱤝'),
      ("kule".to_string(), '󱤞'),
      ("kute".to_string(), '󱤠'),
      ("la".to_string(), '󱤡'),
      ("lape".to_string(), '󱤢'),
      ("laso".to_string(), '󱤣'),
      ("lawa".to_string(), '󱤤'),
      ("len".to_string(), '󱤥'),
      ("lete".to_string(), '󱤦'),
      ("lili".to_string(), '󱤨'),
      ("linja".to_string(), '󱤩'),
      ("lipu".to_string(), '󱤪'),
      ("loje".to_string(), '󱤫'),
      ("lon".to_string(), '󱤬'),
      ("luka".to_string(), '󱤭'),
      ("lukin".to_string(), '󱤮'),
      ("lupa".to_string(), '󱤯'),
      ("ma".to_string(), '󱤰'),
      ("mama".to_string(), '󱤱'),
      ("mani".to_string(), '󱤲'),
      ("meli".to_string(), '󱤳'),
      ("mije".to_string(), '󱤵'),
      ("moku".to_string(), '󱤶'),
      ("moli".to_string(), '󱤷'),
      ("monsi".to_string(), '󱤸'),
      ("mu".to_string(), '󱤹'),
      ("mun".to_string(), '󱤺'),
      ("mute".to_string(), '󱤼'),
      ("nanpa".to_string(), '󱤽'),
      ("nasa".to_string(), '󱤾'),
      ("nasin".to_string(), '󱤿'),
      ("nena".to_string(), '󱥀'),
      ("ni".to_string(), '󱥁'),
      ("nimi".to_string(), '󱥂'),
      ("noka".to_string(), '󱥃'),
      ("o".to_string(), '󱥄'),
      ("olin".to_string(), '󱥅'),
      ("ona".to_string(), '󱥆'),
      ("open".to_string(), '󱥇'),
      ("pakala".to_string(), '󱥈'),
      ("pan".to_string(), '󱥋'),
      ("pana".to_string(), '󱥌'),
      ("pilin".to_string(), '󱥎'),
      ("pimeja".to_string(), '󱥏'),
      ("pini".to_string(), '󱥐'),
      ("pipi".to_string(), '󱥑'),
      ("poka".to_string(), '󱥒'),
      ("poki".to_string(), '󱥓'),
      ("pu".to_string(), '󱥕'),
      ("sama".to_string(), '󱥖'),
      ("seli".to_string(), '󱥗'),
      ("selo".to_string(), '󱥘'),
      ("seme".to_string(), '󱥙'),
      ("sewi".to_string(), '󱥚'),
      ("sike".to_string(), '󱥜'),
      ("sin".to_string(), '󱥝'),
      ("sina".to_string(), '󱥞'),
      ("sinpin".to_string(), '󱥟'),
      ("sitelen".to_string(), '󱥠'),
      ("sona".to_string(), '󱥡'),
      ("soweli".to_string(), '󱥢'),
      ("suli".to_string(), '󱥣'),
      ("suno".to_string(), '󱥤'),
      ("supa".to_string(), '󱥥'),
      ("suwi".to_string(), '󱥦'),
      ("tan".to_string(), '󱥧'),
      ("taso".to_string(), '󱥨'),
      ("telo".to_string(), '󱥪'),
      ("tenpo".to_string(), '󱥫'),
      ("toki".to_string(), '󱥬'),
      ("tomo".to_string(), '󱥭'),
      ("tu".to_string(), '󱥮'),
      ("unpa".to_string(), '󱥯'),
      ("uta".to_string(), '󱥰'),
      ("utala".to_string(), '󱥱'),
      ("walo".to_string(), '󱥲'),
      ("wan".to_string(), '󱥳'),
      ("waso".to_string(), '󱥴'),
      ("wawa".to_string(), '󱥵'),
      ("wile".to_string(), '󱥷'),
      ("namako".to_string(), '󱥸'),
      ("kin".to_string(), '󱥹'),
      ("oko".to_string(), '󱥺'),
      ("kipisi".to_string(), '󱥻'),
      ("leko".to_string(), '󱥼'),
      ("monsuta".to_string(), '󱥽'),
      ("tonsi".to_string(), '󱥾'),
      ("kijetesantakalu".to_string(), '󱦀'),
      ("soko".to_string(), '󱦁'),
      ("meso".to_string(), '󱦂'),
      ("n".to_string(), '󱦆'),
      ("misikeke".to_string(), '󱦇'),
      //
    ]);
    Self {
      mapping,
      buffer: String::new(),
    }
  }
}

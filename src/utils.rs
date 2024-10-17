
//the tuple is first, line #, actual line
pub fn calc_actual_lines<'a>(lines: impl Iterator<Item = &'a String>, max_chars_per_line: usize) -> Vec<(bool, usize, String)> {
  let mut actual_lines = Vec::new();
  let mut line_num = 0;
  for real_line in lines {
    let mut line = real_line.to_string();
    let mut first = true;
    loop {
      if line.chars().count() <= max_chars_per_line {
        actual_lines.push((first, line_num, line));
        break;
      } else {
        let mut line_chars = line.chars();
        let mut push_string = String::new();
        for _i in 0..max_chars_per_line {
          push_string += &line_chars.next().unwrap().to_string();
        }
        actual_lines.push((first, line_num, push_string));
        line = line_chars.collect();
      }
      first = false;
    }
    line_num += 1;
  }
  actual_lines
}


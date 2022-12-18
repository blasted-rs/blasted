use {crate::util::char::CharExt, ropey::RopeSlice};

pub mod jumps;

pub fn cursor_left(
  rope: &RopeSlice,
  (line, pos): &(usize, usize),
) -> (usize, usize) {
  // we are at the beginning of the line
  if *pos == 0 {
    if *line == 0 {
      (*line, *pos)
    } else {
      let prev_line = rope.line_to_char(*line - 1);
      let prev_line_len = rope.line(*line - 1).len_chars();
      (line - 1, prev_line_len)
    }
    // we are somewhere in the middle of the line
  } else {
    (*line, *pos - 1)
  }
}

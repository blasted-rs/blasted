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

pub fn cursor_right(
  rope: &RopeSlice,
  (line, pos): &(usize, usize),
) -> (usize, usize) {
  // we are at the end of the line
  if *pos == rope.line(*line).len_chars() {
    if *line == rope.len_lines() - 1 {
      (*line, *pos)
    } else {
      (line + 1, 0)
    }
    // we are somewhere in the middle of the line
  } else {
    (*line, *pos + 1)
  }
}

// TODO: remember the pos when moving up
//       and when encountering a shorter line
//       move to the end of the shorter line but
//       when encountering a longer line move to
//       the remembered pos
pub fn cursor_up(
  rope: &RopeSlice,
  (line, pos): &(usize, usize),
) -> (usize, usize) {
  if *line == 0 {
    (*line, *pos)
  } else {
    let pos = if *pos > rope.line(*line - 1).len_chars() {
      rope.line(*line - 1).len_chars()
    } else {
      *pos
    };
    (*line - 1, pos)
  }
}

pub fn cursor_down(
  rope: &RopeSlice,
  (line, pos): &(usize, usize),
) -> (usize, usize) {
  if *line == rope.len_lines() - 1 {
    (*line, *pos)
  } else {
    let pos = if *pos > rope.line(*line + 1).len_chars() {
      rope.line(*line + 1).len_chars()
    } else {
      *pos
    };
    (*line + 1, pos)
  }
}

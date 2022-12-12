use ropey::RopeSlice;
use crate::util::char::CharExt;

pub fn next_word(
  r: &RopeSlice,
  (line, pos): &(usize, usize),
) -> (usize, usize) {
  // TODO: no unwraps
  let mut chars = r.get_chars_at(r.line_to_char(*line) + pos).unwrap();

  // "wo|rd  second"
  let mut whitespace = false;
  let mut pos_offset = *pos;
  let mut line_offset = *line;
  #[allow(clippy::while_let_on_iterator)]
  while let Some(c) = chars.next() {
    if c.is_whitespace() {
      whitespace = true;
      if c.is_line_ending() {
        line_offset += 1;
        pos_offset = 0;
        continue;
      }
    }

    if c.is_alphanumeric() && whitespace {
      break;
    }
    pos_offset += 1;
  }

  (line_offset, pos_offset)
}

#[test]
fn test_next_word() {
  use ropey::Rope;
  let buffer = Rope::from_str("one two three four       five\nsix seven");

  assert_eq!(next_word(&buffer.slice(..), &(0, 0)), (0, 4));
  assert_eq!(next_word(&buffer.slice(..), &(0, 4)), (0, 8));
  assert_eq!(next_word(&buffer.slice(..), &(0, 12)), (0, 14));

  // end of line, it should continue on the next line
  assert_eq!(next_word(&buffer.slice(..), &(0, 25)), (0, 30));
  assert_eq!(next_word(&buffer.slice(..), &(0, 30)), (0, 34));
  assert_eq!(next_word(&buffer.slice(..), &(0, 34)), (0, 39));

  // TODO: out of checking / removing the unwraps()

  dbg!(buffer.slice(39..));
}

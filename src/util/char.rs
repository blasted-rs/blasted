use super::line_endings::LineEnding;

pub trait CharExt {
  fn is_line_ending(&self) -> bool;
}

impl CharExt for char {
  fn is_line_ending(&self) -> bool {
    LineEnding::from_char(*self).is_some()
  }
}

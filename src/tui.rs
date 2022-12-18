use tui::layout::Rect;

/// tui Area extention trait
pub trait AreaExt {
  fn with_height(self, height: u16) -> Rect;
}

impl AreaExt for Rect {
  fn with_height(self, height: u16) -> Rect {
    // new height may make area > u16::max_value, so use new()
    Self::new(self.x, self.y, self.width, height)
  }
}

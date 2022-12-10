use slotmap::new_key_type;

new_key_type! { pub struct ViewId; }

#[derive(Default)]
pub struct View {
  // offset: (usize, usize),
}

impl View {}

use {crate::document::DocumentId, slotmap::new_key_type};

new_key_type! { pub struct ViewId; }

pub struct View {
  pub document_id: DocumentId,
  // offset: (usize, usize),
}

impl View {
  pub fn new(document_id: DocumentId) -> Self {
    Self { document_id }
  }
}

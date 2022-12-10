use {
  crate::{
    document::{Document, DocumentId},
    view::{View, ViewId},
  },
  slotmap::SlotMap,
};

#[derive(Default)]
pub struct Editor {
  views: SlotMap<ViewId, View>,
  documents: SlotMap<DocumentId, Document>,
}

impl Editor {
  pub fn create_view(&mut self, document: DocumentId) -> Option<ViewId> {
    let view = self.views.insert(View::default());

    let document = self.documents.get_mut(document)?;
    document.new_view(view);

    Some(view)
  }

  pub fn create_document(&mut self) -> DocumentId {
    self.documents.insert(Document::default())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_editor_document_and_view() {
    let mut editor = Editor::default();
    let document_id = editor.create_document();
    let view_id = editor.create_view(document_id).unwrap();

    // check if we created the cursor in the document
    let document = editor.documents.get(document_id).unwrap();
    assert_eq!(document.cursor[&view_id], (0, 0));
  }
}

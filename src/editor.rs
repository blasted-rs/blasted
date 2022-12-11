use crate::document::DocumentError;

use {
  crate::{
    document::{Document, DocumentId},
    view::{View, ViewId},
  },
  slotmap::SlotMap,
  thiserror::Error,
};

#[derive(Default)]
pub struct Editor {
  pub views: SlotMap<ViewId, View>,
  pub documents: SlotMap<DocumentId, Document>,
  pub active_view: Option<ViewId>,
}

#[derive(Debug, Error)]
pub enum EditorError {
  #[error("Trying to access a non-existent view")]
  ViewNotPresent,
  #[error("Trying to access a non-existent document")]
  DocumentNotPresent,
  #[error(transparent)]
  IoError(#[from] std::io::Error),
  #[error(transparent)]
  DocumentError(#[from] DocumentError),
}

type EditorResult<T> = Result<T, EditorError>;

impl Editor {
  pub fn create_view(&mut self, document: DocumentId) -> EditorResult<ViewId> {
    let view = self.views.insert(View::new(document));

    let document = self.documents.get_mut(document).ok_or(EditorError::DocumentNotPresent)?;
    document.new_view(view);

    // set active view if none is set
    if self.active_view.is_none() {
      self.active_view = Some(view);
    }

    Ok(view)
  }

  pub fn create_document(&mut self) -> DocumentId {
    self.documents.insert(Document::default())
  }

  pub fn open(
    &mut self,
    path: impl AsRef<std::path::Path>,
  ) -> EditorResult<( DocumentId, ViewId )> {
    // load path into a rope
    let document = Document::from_reader(path)?;

    // add to editor
    let document_id = self.documents.insert(document);
    let view_id = self.create_view(document_id)?;
    Ok((document_id, view_id))
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

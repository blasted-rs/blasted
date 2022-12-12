use {
  crate::{
    application::{Application, Plugin, PluginError, ProcessEvent},
    document::{Document, DocumentError, DocumentId},
    view::{View, ViewId},
  },
  crossterm::event::{Event, KeyCode},
  slotmap::SlotMap,
  thiserror::Error,
  tui::{buffer::Buffer as TuiBuffer, layout::Rect, style::Style},
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

    let document = self
      .documents
      .get_mut(document)
      .ok_or(EditorError::DocumentNotPresent)?;
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
  ) -> EditorResult<(DocumentId, ViewId)> {
    // load path into a rope
    let document = Document::from_reader(path)?;

    // add to editor
    let document_id = self.documents.insert(document);
    let view_id = self.create_view(document_id)?;
    Ok((document_id, view_id))
  }
}

impl Plugin for Editor {
  fn id(&self) -> Option<&'static str> {
    Some("editor")
  }

  fn init(&self, _app: &Application) -> Result<(), PluginError> {
    // TODO: load files from disk
    Ok(())
  }

  fn process_event(
    &self,
    app: &mut Application,
    event: &Event,
  ) -> Result<ProcessEvent, PluginError> {
    if let Event::Key(key) = event {
      if let KeyCode::Char('q') = key.code {
        println!("Quitting");
        if let Err(e) = app.quit() {
          tracing::error!("Failed to quit: {}", e);
        }
      }
    }

    Ok(ProcessEvent::Consumed)
  }

  fn render(
    &mut self,
    _app: &mut Application,
    _area: &Rect,
    frame: &mut TuiBuffer,
  ) {
    // render the active view and the command bar
    // or is the command bar a separate plugin?
    frame.set_string(0, 0, "Hello World", Style::default());
  }

  fn cursor(&self, _area: Rect) -> Option<(u16, u16)> {
    Some((5, 0))
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

use {
  crate::{
    application::{Application, Plugin, PluginError, ProcessEvent},
    document::{DocEvent, Document, DocumentError, DocumentId},
    view::{View, ViewId},
  },
  anyhow::Error as AnyError,
  crossterm::event::{Event as TuiEvent, KeyCode},
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
  pub fn create_view(
    &mut self,
    document_id: DocumentId,
  ) -> EditorResult<ViewId> {
    let view_id = self.views.insert(View::new(document_id));

    let document = self
      .documents
      .get_mut(document_id)
      .ok_or(EditorError::DocumentNotPresent)?;
    document.new_view(view_id);

    // set active view if none is set
    if self.active_view.is_none() {
      self.active_view = Some(view_id);
    }

    Ok(view_id)
  }

  pub fn active_view(&self) -> Option<(ViewId, DocumentId)> {
    self.active_view.map(|view_id| {
      let document_id = self
        .views
        .get(view_id)
        .expect("active view should exist")
        .document_id;
      (view_id, document_id)
    })
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
    &mut self,
    app: &mut Application,
    event: &TuiEvent,
  ) -> Result<ProcessEvent, PluginError> {
    if let TuiEvent::Key(key) = event {
      match key.code {
        KeyCode::Char('q') => {
          println!("Quitting");
          if let Err(e) = app.quit() {
            tracing::error!("Failed to quit: {}", e);
          }
        }
        KeyCode::Char('w') => {
          if let Some((view_id, document_id)) = self.active_view() {
            let document = self
              .documents
              .get_mut(document_id)
              .expect("document not present");

            document
              .process(&view_id, &DocEvent::MoveWord)
              .map_err(AnyError::from)?;
          }
        }
        _ => (),
      }
    }

    Ok(ProcessEvent::Consumed)
  }

  fn render(
    &mut self,
    _app: &mut Application,
    area: &Rect,
    frame: &mut TuiBuffer,
  ) {
    // render the active view and the command bar
    // or is the command bar a separate plugin?
    // frame.set_string(0, 0, "Hello World", Style::default());
    if let Some((view_id, document_id)) = self.active_view() {
      // TODO: get offset of view
      let document = self
        .documents
        .get_mut(document_id)
        .expect("document not present");

      for (line, text) in
        document.rope.lines().enumerate().take(area.height as usize)
      {
        frame.set_string(0, line as u16, text.to_string(), Style::default());
      }
    }
  }

  fn cursor(&self, _area: Rect) -> Option<(u16, u16)> {
    self.active_view().map(|(view_id, document_id)| {
      let document = self
        .documents
        .get(document_id)
        .expect("document not present");
      let (pos, line) =
        document.cursor.get(&view_id).expect("cursor not present");
      (*pos as u16, *line as u16)
    })
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

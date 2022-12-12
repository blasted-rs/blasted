use {
  crate::{movement, view::ViewId},
  ropey::Rope,
  slotmap::new_key_type,
  std::{collections::HashMap, convert::Infallible, str::FromStr},
  thiserror::Error,
};

new_key_type! { pub struct DocumentId; }

pub enum DocEvent {
  MoveWord,
}

#[derive(Error, Debug)]
pub enum DocumentError {
  #[error("Trying to access a non-existent view")]
  ViewNotPresent,
  #[error(transparent)]
  IoError(#[from] std::io::Error),
}

pub type DocumentResult<T> = Result<T, DocumentError>;

#[derive(Default)]
pub struct Document {
  pub rope: Rope,
  pub cursor: HashMap<ViewId, (usize, usize)>,
}

impl Document {
  pub fn new_view(&mut self, view: ViewId) {
    self.cursor.insert(view, Default::default());
  }

  pub fn from_reader(
    path: impl AsRef<std::path::Path>,
  ) -> DocumentResult<Self> {
    let rope = Rope::from_reader(std::fs::File::open(path)?)?;
    Ok(Self {
      rope,
      cursor: Default::default(),
    })
  }

  pub fn lines(&'_ self) -> impl Iterator<Item = String> + '_ {
    self.rope.lines().map(|line| line.to_string())
  }

  pub fn process(
    &mut self,
    view_id: &ViewId,
    event: &DocEvent,
  ) -> DocumentResult<()> {
    if !self.cursor.contains_key(view_id) {
      return Err(DocumentError::ViewNotPresent);
    };

    // TODO: better error type
    let rope = self.rope.slice(..);

    match event {
      DocEvent::MoveWord => {
        self.cursor.entry(*view_id).and_modify(|c| {
          *c = movement::jumps::next_word(&rope, c);
        });
      }
    }

    Ok(())
  }
}

impl FromStr for Document {
  type Err = Infallible;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self {
      rope: Rope::from_str(s),
      ..Default::default()
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_processing_of_events() {
    // create the document
    let mut document = Document::default();
    document.rope.insert(0, "one two three four");

    // create a view
    let view_id = ViewId::default();
    document.new_view(view_id);

    document.process(&view_id, &DocEvent::MoveWord).unwrap();
    assert_eq!(document.cursor.get(&view_id), Some(&(0, 4)));

    document.process(&view_id, &DocEvent::MoveWord).unwrap();
    assert_eq!(document.cursor.get(&view_id), Some(&(0, 8)));
  }

  #[test]
  fn test_from_str() {
    let doc = Document::from_str("Hello world!").unwrap();
    assert_eq!(doc.rope, "Hello world!");
  }
}

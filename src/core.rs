use {
  ropey::Rope,
  slotmap::{new_key_type, SlotMap},
  std::{collections::HashMap, convert::Infallible, str::FromStr},
  thiserror::Error,
};

// Terminal => KeyEvents => View => KeyMap => EditingModel => (DocumentEvent =>
// Document)
//
// EditModel (state, normal, insert, 2dw)
// events_in => KeyMap Events
// events_out => DocumentEvents
//
//  delete word
//
// 2dw: delete the next two words
//
// one two tree four
//
// KeyEvents:
// <ESC> => ModelEditing in NORMAL mode
// 2 => triggers a state in ModelEditing (no DocumentEvents are generated)
// d => ModelEditing
// w => ModelEditing
// DocumentEvents
//   Document
//      cursor
//      2x DocumentEvent::DeleteToNextWord
//
// Document
//   implement all the jumps possible in vim
//   we can implement plain editing and all other
//   etings
//
//  Higly customizable system
//
// INSERT

// Document
//   text
//   syntax => treesitter
//   language
//   cursors: Cursors
//   events() <= Event
//
// View
//   jumplist
//   offset
new_key_type! { pub struct ViewId; }
new_key_type! { pub struct DocumentId; }

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

#[derive(Default)]
pub struct View {
  offset: (usize, usize),
}

impl View {}

pub enum Event {
  MoveWord,
}

#[derive(Error, Debug)]
pub enum DocumentError {
  #[error("Noop")]
  Noop,
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

  pub fn process(
    &mut self,
    view_id: &ViewId,
    event: Event,
  ) -> DocumentResult<()> {
    // TODO: better error type
    let rope = self.rope.slice(..);

    match event {
      Event::MoveWord => {
        self.cursor.entry(*view_id).and_modify(|c| {
          *c = movement::jumps::next_word(&rope, c);
        });
      }
    }

    Ok(())
  }
}

#[test]
fn test_processing_of_events() {
  let mut editor = Editor::default();
  let document = editor.create_document();
  let view = editor.create_view(document).unwrap();

  let mut document = editor.documents.get_mut(document).unwrap();

  document.rope.insert(0, "one two three four");

  document.process(&view, Event::MoveWord).unwrap();
  assert_eq!(document.cursor.get(&view), Some(&(0, 4)));

  document.process(&view, Event::MoveWord).unwrap();
  assert_eq!(document.cursor.get(&view), Some(&(0, 8)));
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

pub mod movement {
  /// functional manipulation of a cursor
  pub mod jumps {
    use ropey::RopeSlice;

    pub fn next_word(
      r: &RopeSlice,
      (line, pos): &(usize, usize),
    ) -> (usize, usize) {
      // TODO: no unwraps
      let mut chars = r.get_chars_at(r.line_to_char(*line) + pos).unwrap();

      // "wo|rd  second"
      let mut whitespace = false;
      let mut offset = 0;
      #[allow(clippy::while_let_on_iterator)]
      while let Some(c) = chars.next() {
        if c.is_whitespace() {
          whitespace = true;
        }

        if c.is_alphanumeric() && whitespace {
          break;
        }
        offset += 1;
      }

      (*line, *pos + offset)
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
  }
}

#[cfg(test)]
mod test {
  use {
    super::{Document, *},
    ropey::Rope,
    std::str::FromStr,
  };

  #[test]
  fn test_creation_of_editor() {
    let mut editor = Editor::default();
    let document_id = editor.create_document();
    let view_id = editor.create_view(document_id).unwrap();

    // check if we created the cursor in the document
    let document = editor.documents.get(document_id).unwrap();
    assert_eq!(document.cursor[&view_id], (0, 0));
  }

  #[test]
  fn test_from_str() {
    let doc = Document::from_str("Hello world!").unwrap();
    assert_eq!(doc.rope, "Hello world!");
  }

  #[test]
  fn rope_test() {
    let mut rope = Rope::from_str("Hello world!");

    rope.remove(6..11);
    rope.insert(6, "Boy");

    assert_eq!(rope, "Hello Boy!");
  }
}

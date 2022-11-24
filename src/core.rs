use {
  ropey::{Rope, RopeSlice},
  slotmap::{new_key_type, SlotMap},
  std::{collections::HashMap, convert::Infallible, str::FromStr},
};

// Terminal => KeyEvents => Views => KeyMap => EditModel => (DocumentEvent =>
// Document)
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
  // offset: (usize, usize),
}

#[derive(Default)]
pub struct Document {
  pub rope: Rope,
  pub cursor: HashMap<ViewId, (usize, usize)>,
}

impl Document {
  #[allow(clippy::should_implement_trait)]
  pub fn from_str(s: &str) -> Self {
    Self {
      rope: Rope::from_str(s),
      ..Default::default()
    }
  }

  pub fn new_view(&mut self, view: ViewId) {
    self.cursor.insert(view, Default::default());
  }

  pub fn process(&mut self, view: ViewId, event: DocumentEvent) {
    match event {
      DocumentEvent::JumpNextWord => {
        self.cursor.entry(view).and_modify(mod_cursor(
          &self.rope.slice(..),
          &movement::jump::next_word,
        ));
      }
    }
  }
}

/// higher order function to convert from a functional move
/// function to a mutable function
fn mod_cursor<'a, 'b>(
  r: &'b RopeSlice,
  mfn: &'a impl Fn(&'a RopeSlice, &(usize, usize)) -> (usize, usize),
) -> impl FnMut(&mut (usize, usize)) + 'a
where
  'b: 'a,
{
  |p: &mut (usize, usize)| {
    let u = mfn(r, p);
    p.0 = u.0;
    p.1 = u.1;
  }
}

pub enum DocumentEvent {
  JumpNextWord,
}

pub mod movement {
  pub mod jump {
    use ropey::RopeSlice;
    pub fn next_word(
      t: &RopeSlice,
      (line, pos): &(usize, usize),
    ) -> (usize, usize) {
      let mut chars_to_end_of_line = t.get_line(*line).unwrap().chars_at(*pos);

      // always find whitespace and then hit the first
      // alphanumeric
      let mut white_space = false;
      let mut counter = 0;
      #[allow(clippy::while_let_on_iterator)]
      while let Some(c) = chars_to_end_of_line.next() {
        if c.is_whitespace() {
          white_space = true;
        }
        if c.is_alphanumeric() && white_space {
          break;
        }
        counter += 1;
      }

      (*line, pos + counter)
    }
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
mod test {
  use {
    super::{Document, *},
    ropey::Rope,
  };

  #[test]
  fn test_jump_word() {
    let mut doc = Document::from_str("Hello World");
    let view = ViewId::from(slotmap::KeyData::from_ffi(0));

    doc.new_view(view);
    doc.process(view, DocumentEvent::JumpNextWord);

    assert_eq!(doc.cursor[&view], (0, 6));
  }

  #[test]
  fn test_creation_of_editor() {
    let mut editor = Editor::default();
    let document_id = editor.create_document();
    let view_id = editor.create_view(document_id).unwrap();

    // document

    // check if we created the cursor in the document
    let document = editor.documents.get(document_id).unwrap();
    assert_eq!(document.cursor[&view_id], (0, 0));
  }

  #[test]
  fn test_from_str() {
    let doc = Document::from_str("Hello world!");
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

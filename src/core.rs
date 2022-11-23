use std::{str::FromStr, convert::Infallible, collections::HashMap};

use ropey::Rope;
use slotmap::{new_key_type, SlotMap};

// Terminal =>  KeyStores =>  View => EditingModel => Events => Document
//
// Event::JumpToNextWord(view_id)
// Event::NextBlock(view_id)
//
//

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
    documents: SlotMap<DocumentId, Document>
}

impl Editor {
    fn create_view(&mut self, document: DocumentId) -> Option<ViewId> {
        let view = self.views.insert(View::default());

        let document = self.documents.get_mut(document)?;
        document.new_view(view);

        Some(view)
    }

    fn create_document(&mut self) -> DocumentId {
        self.documents.insert(Document::default())
    }
}

#[derive(Default)]
pub struct View {
    offset: (usize, usize),
}

#[derive(Default)]
pub struct Document {
    pub rope: Rope,
    pub cursor: HashMap<ViewId, (usize, usize)>,
}

impl Document {
    fn new_view(&mut self, view: ViewId) {
        self.cursor.insert(view, Default::default());
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
    use super::Document;
    use ropey::Rope;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_creation_of_editor() {
        let mut editor = Editor::default();
        let document_id = editor.create_document();
        let view_id = editor.create_view(document_id).unwrap();

        // check if we created the cursor in the document
        let document = editor.documents.get(document_id).unwrap();
        assert_eq!(document.cursor[&view_id], (0,0));
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

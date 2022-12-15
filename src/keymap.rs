use {
  crate::document::DocEvent,
  crossterm::event::{Event as TuiEvent, KeyCode},
};

#[derive(Default)]
enum Mode {
  #[default]
  Normal,
  Insert,
  Visual,
}

#[derive(Default)]
pub struct ViKeymap {
  mode: Mode,
}

impl ViKeymap {
  fn process_normal_mode_event(
    &mut self,
    event: &TuiEvent,
  ) -> Option<Vec<DocEvent>> {
    match event {
      TuiEvent::Key(key_event) => {
        match key_event.code {
          KeyCode::Char('i') => {
            self.mode = Mode::Insert;
            None
          }
          KeyCode::Char('v') => {
            self.mode = Mode::Visual;
            None
          }
          KeyCode::Esc => {
            self.mode = Mode::Normal;
            None
          }
          KeyCode::Char('h') => Some(vec![DocEvent::MoveCursorLeft]),
          KeyCode::Char('j') => Some(vec![DocEvent::MoveCursorDown]),
          KeyCode::Char('k') => Some(vec![DocEvent::MoveCursorUp]),
          KeyCode::Char('l') => Some(vec![DocEvent::MoveCursorRight]),
          KeyCode::Char('w') => Some(vec![DocEvent::MoveWordForward]),
          KeyCode::Char('b') => Some(vec![DocEvent::MoveWordBackward]),
          KeyCode::Char('e') => Some(vec![DocEvent::MoveWordEnd]),
          KeyCode::Char('0') => Some(vec![DocEvent::MoveLineStart]),
          KeyCode::Char('$') => Some(vec![DocEvent::MoveLineEnd]),
          KeyCode::Char('G') => Some(vec![DocEvent::MoveDocumentEnd]),
          KeyCode::Char('g') => {
            // TODO: gg
            Some(vec![DocEvent::MoveDocumentStart])
          }
          KeyCode::Char('x') => Some(vec![DocEvent::DeleteChar]),
          _ => None,
        }
      }
      _ => None,
    }
  }

  fn process_insert_mode_event(
    &mut self,
    event: &TuiEvent,
  ) -> Option<Vec<DocEvent>> {
    if let TuiEvent::Key(key_event) = event {
        match key_event.code {
        KeyCode::Esc => {
          self.mode = Mode::Normal;
          None
        }

        _ => None,
      }
    } else {
        None
    }
  }

  fn process_visual_mode_event(
    &mut self,
    event: &TuiEvent,
  ) -> Option<Vec<DocEvent>> {
    if let TuiEvent::Key(key_event) = event {
        match key_event.code {
        KeyCode::Esc => {
          self.mode = Mode::Normal;
          None
        }

        _ => None,
      }
    } else {
        None
    }
  }

  pub fn process_event(&mut self, event: &TuiEvent) -> Option<Vec<DocEvent>> {
    match self.mode {
      Mode::Normal => self.process_normal_mode_event(event),
      Mode::Insert => self.process_insert_mode_event(event),
      Mode::Visual => self.process_visual_mode_event(event),
    }
  }
}

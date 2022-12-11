use blasted::{editor::Editor, view::ViewId};

use {
  anyhow::Result,
  blasted::Document,
  crossterm::{
    self,
    event::{
      DisableBracketedPaste,
      DisableFocusChange,
      DisableMouseCapture,
      EnableBracketedPaste,
      EnableFocusChange,
      EnableMouseCapture,
      Event,
      EventStream,
      KeyCode,
      KeyEvent,
    },
    execute,
    terminal::{
      self,
      disable_raw_mode,
      EnterAlternateScreen,
      LeaveAlternateScreen,
    },
  },
  futures::StreamExt,
  std::str::FromStr,
  tui::{
    backend::CrosstermBackend,
    buffer::Buffer as Surface,
    layout::Rect,
    style::Style,
    Terminal,
  },
};

pub trait AreaExt {
  fn with_height(self, height: u16) -> Rect;
}

impl AreaExt for Rect {
  fn with_height(self, height: u16) -> Rect {
    // new height may make area > u16::max_value, so use new()
    Self::new(self.x, self.y, self.width, height)
  }
}

struct Application {
  terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
  editor: Editor,
  active_view: Option<ViewId>,
  last_event: Option<Event>,
}

impl Application {
  fn new(terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Self {
    Self { terminal, editor: Editor::default(), last_event: None, active_view: None }
  }

  pub fn set_active_view(&mut self, view: ViewId) {
    self.active_view = Some(view);
  }


  pub fn render(&mut self, frame: usize) {
    let area = self.terminal.size().unwrap();
    let surface = self.terminal.current_buffer_mut();

    // draw frames right top
    surface.set_string(area.width - 4, 0, &format!("F{}", frame % 1000), Style::default());

    if let Some(view) = self.editor.active_view {
        let view = self.editor.views.get(view).unwrap();
        let document = self.editor.documents.get(view.document_id).unwrap();

        for (y,line) in document.lines().enumerate() {
            if y >= area.height as usize {
                break;
            }
            surface.set_string(0, y as u16, line, Style::default());
        }
    }

    // if let Some(document) = &self.document {
    //     for (y, line) in document.rope.lines().enumerate() {
    //         surface.set_string(0, y as u16, line.to_string(), Style::default());
    //     }
    // }

    // write to surface
    // Self::render_bufferline(area.with_height(1), surface);

    if let Some(event) = &self.last_event {
        surface.set_string(0, 10, format!("{:?}", event), Style::default());
    }

    // trigger render and render to screen
    self.terminal.draw(|f| f.set_cursor(0,0)).unwrap();
  }

  pub fn render_bufferline(viewport: Rect, surface: &mut Surface) {
    // surface.clear(viewport);
    surface
      .set_stringn(
        viewport.x,
        viewport.y,
        "bufferline.rs[+]",
        viewport.width as usize,
        Style::default(),
      );
  }

  pub async fn run(&mut self, events: &mut EventStream) {
    self.render(0);
    let mut frames = 0;
    while let Some(Ok(event)) = events.next().await {

      if let Event::Key(KeyEvent {
                code, modifiers, ..
              }) = event {
        if modifiers.is_empty() {
          if let KeyCode::Char('q') = code {
            break;
          }
        }
      }

      self.last_event = Some(event);
      frames += 1;
      self.render(frames);
    }
  }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {

  crossterm::terminal::enable_raw_mode().unwrap();

  let mut stdout = std::io::stdout();

  // Claim the terminal for our use
  execute!(
    stdout,
    EnterAlternateScreen,
    EnableBracketedPaste,
    EnableFocusChange,
    EnableMouseCapture
  )?;
  execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

  // setup applicatoin
  let backend = CrosstermBackend::new(stdout);
  let terminal = Terminal::new(backend)?;
  // let area = terminal.size().expect("couldn't get terminal size");

  let mut events = EventStream::new();

  let mut app = Application::new(terminal);

  // active_view is set when opening a file
  let ( document_id, view_id ) = app.editor.open("src/main.rs")?;

  // app.set_active_view(view_id);

  app.run(&mut events).await;

  // restore the terminal
  let mut stdout = std::io::stdout();
  execute!(
    stdout,
    DisableMouseCapture,
    DisableBracketedPaste,
    DisableFocusChange,
    LeaveAlternateScreen
  )?;

  terminal::disable_raw_mode()?;

  disable_raw_mode()?;

  Ok(())
}

// let mut reader = EventStream::new();
// loop {
//   match reader.next().await {
//     Some(Ok(event)) => {
//       println!("{:?}", event);
//       if let Event::Key(KeyEvent {
//         code: KeyCode::Char('q'),
//         ..
//       }) = event
//       {
//         break;
//       }
//     }
//     Some(Err(e)) => {
//       println!("Error: {:?}", e);
//       break;
//     }
//     None => break,
//   }
// }

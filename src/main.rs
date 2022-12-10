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
  last_event: Option<Event>,
}

impl Application {
  fn new(terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Self {
    Self { terminal, last_event: None }
  }

  pub fn render(&mut self) {
    let area = self.terminal.size().unwrap();
    let surface = self.terminal.current_buffer_mut();

    // write to surface
    Self::render_bufferline(area.with_height(1), surface);

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
    self.render();
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
      self.render();
    }
  }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  let _core = Document::from_str("Hello world!");

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

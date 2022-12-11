use {
  crossterm::{
    self,
    event::{
      DisableBracketedPaste,
      DisableFocusChange,
      DisableMouseCapture,
      EnableBracketedPaste,
      EnableFocusChange,
      EnableMouseCapture,
      EventStream,
    },
    execute,
    terminal::{
      self,
      disable_raw_mode,
      EnterAlternateScreen,
      LeaveAlternateScreen,
    },
  },
  tui::{backend::CrosstermBackend, Terminal},
};

pub fn claim_terminal() -> Result<(Terminal<CrosstermBackend<std::io::Stdout>>, EventStream), std::io::Error>  {
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

  let backend = CrosstermBackend::new(stdout);
  let terminal = Terminal::new(backend)?;

  let events = EventStream::new();

  Ok((terminal, events))
}

pub fn restore_terminal() -> Result<(), std::io::Error> {
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

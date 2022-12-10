use {
  anyhow::Result,
  blasted::Document,
  crossterm::{
    self,
    event::{
      DisableMouseCapture,
      EnableMouseCapture,
      Event,
      EventStream,
      KeyCode,
      KeyEvent,
    },
    execute,
    terminal::disable_raw_mode,
  },
  futures::StreamExt,
  std::str::FromStr,
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  let _core = Document::from_str("Hello world!");

  // use crossterm to set terminal into raw mode
  // and then restore it when the program exits
  // while capturing stdin and read on the EventStream
  // while printing the events to stdout

  crossterm::terminal::enable_raw_mode().unwrap();

  let mut stdout = std::io::stdout();
  execute!(stdout, EnableMouseCapture)?;

  let mut reader = EventStream::new();
  loop {
    match reader.next().await {
      Some(Ok(event)) => {
        println!("{:?}", event);
        if let Event::Key(KeyEvent {
          code: KeyCode::Char('q'),
          ..
        }) = event
        {
          break;
        }
      }
      Some(Err(e)) => {
        println!("Error: {:?}", e);
        break;
      }
      None => break,
    }
  }

  execute!(stdout, DisableMouseCapture)?;

  disable_raw_mode()?;

  Ok(())
}

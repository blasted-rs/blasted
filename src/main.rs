use {
  anyhow::Result,
  blasted::{application::Application, term},
};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
  let (terminal, mut event_stream) = term::claim_terminal()?;

  // run the main application loop for the terminal
  let mut app = Application::new(terminal);
  app.editor.as_mut().unwrap().open("src/main.rs")?;
  app.run(&mut event_stream).await?;

  term::restore_terminal()?;

  Ok(())
}

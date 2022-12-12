use {
  crate::editor::Editor,

  crossterm::event::{Event, EventStream},
  futures::StreamExt,
  std::collections::VecDeque,
  thiserror::Error,
  tokio::sync::mpsc::{error::SendError, UnboundedSender},
  tui::{
    backend::CrosstermBackend,
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    Terminal,
  },
};

#[derive(Debug, Error)]
pub enum PluginError {
  #[error("Could not initialize plugin {0}")]
  InitFailed(String),
}

pub enum ProcessEvent {
  Consumed,
  Ignored,
}

pub trait Plugin {
  fn id(&self) -> Option<&'static str>;
  fn type_name(&self) -> &'static str {
    std::any::type_name::<Self>()
  }
  fn init(&self, app: &Application) -> Result<(), PluginError>;
  fn process_event(
    &self,
    app: &mut Application,
    event: &Event,
  ) -> Result<ProcessEvent, PluginError>;

  /// Get cursor position and cursor kind.
  fn cursor(
    &self,
    _area: Rect,
  ) -> Option<(u16, u16)> {
    None
  }
  /// Render the plugin onto the provided surface.
  fn render(
    &mut self,
    app: &mut Application,
    area: &Rect,
    frame: &mut TuiBuffer,
  );
}

pub type TuiTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug, Error)]
pub enum ApplicationError {
  #[error(transparent)]
  PluginError(#[from] PluginError),
  #[error(transparent)]
  SendCommandError(#[from] SendError<Command>),
  #[error(transparent)]
  IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum Command {
  Quit,
}

pub struct Application {
  editor: Option<Editor>,
  plugins: Vec<Box<dyn Plugin>>,
  active_plugins: VecDeque<Box<dyn Plugin>>,
  terminal: Option<TuiTerminal>,
  cmd: Option<UnboundedSender<Command>>,
}

impl Application {
  pub fn new(terminal: TuiTerminal) -> Application {
    Self {
      terminal: Some(terminal),
      editor: Some(Editor::default()),
      plugins: Vec::new(),
      active_plugins: VecDeque::new(),
      cmd: None,
    }
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
    self.plugins.push(plugin);
  }

  pub fn quit(&mut self) -> Result<(), ApplicationError> {
    if let Some(cmd) = self.cmd.take() {
      cmd.send(Command::Quit)?;
    } else {
      tracing::warn!("Application::quit() called without a command channel");
    }

    Ok(())
  }

  /// run our application plugin system
  /// active plugins will be executed in reverse order of activation.
  pub async fn run(
    &mut self,
    events: &mut EventStream,
  ) -> Result<(), ApplicationError> {
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    self.cmd = Some(cmd_tx);

    let mut fused_events = events.fuse();
    loop {
      tokio::select! {
          cmd = cmd_rx.recv() => {
              match cmd {
                  Some(Command::Quit) => {
                      return Ok(());
                  }
                  _ => {}
              }
          }

          Ok(event) = fused_events.select_next_some() => {
              // first process the event with the active plugins, notice
              // we are moving the plugin out of the active_plugins list
              let mut consumed = false;
              let mut processed_plugins = VecDeque::new();
              while let Some(plugin) = self.active_plugins.pop_back() {
                  consumed =
                      matches!(plugin.process_event(self, &event)?, ProcessEvent::Consumed);
                  processed_plugins.push_front(plugin);
                  if consumed {
                      break;
                  }
              }
              // restoring the plugins
              self.active_plugins.append(&mut processed_plugins);

              // now process the event with on editor
              if !consumed {
                  if let Some(editor) = self.editor.take() {
                      editor.process_event(self, &event)?;
                      self.editor = Some(editor);
                  }
              }

              // now after we processed all the events
              // lets render the plugins
              if let Some(mut terminal) = self.terminal.take() {
                  let area = terminal.size()?;
                  let surface = terminal.current_buffer_mut();

                  // process the plugins
                  let mut processed_plugins = VecDeque::new();
                  while let Some(mut plugin) = self.active_plugins.pop_front() {
                      plugin.render(self, &area, surface);
                      processed_plugins.push_back(plugin);
                  }
                  self.active_plugins.append(&mut processed_plugins);

                  // the last plugin to render is our editor plugin
                  if let Some(mut editor) = self.editor.take() {
                      editor.render(self, &area, surface);
                      self.editor = Some(editor);
                  }

                  // set the cursor position
                  let mut cursor = self.active_plugins
                      .iter()
                      .rev()
                      .find_map(|p| p.cursor(area));

                  // when no cursor set yet, try the editor
                  if cursor.is_none() {
                      if let Some(editor) = &self.editor {
                          cursor = editor.cursor(area);
                      }
                  }

                  // set the cursor
                  let (x,y) = cursor.unwrap_or((0,0));
                  terminal.draw(|f| f.set_cursor(x, y))?;
              }
          }
      }
    }
  }
}

pub enum ApplicationEvent {
  ActivatePlugin(String),
  DeactivatePlugin(String),
  Quit,
}
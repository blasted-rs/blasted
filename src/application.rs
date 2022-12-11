use {
  crossterm::event::{Event, EventStream},
  futures::StreamExt,
  std::collections::VecDeque,
  thiserror::Error,
  tui::{backend::CrosstermBackend, Terminal},
};

#[derive(Debug, Error)]
pub enum PluginError {
  #[error("Could not initialize plugin {0}")]
  InitFailed(String),
}

pub trait Plugin {
  fn name(&self) -> &str;
  fn init(&self, app: &Application) -> Result<(), PluginError>;
  fn process_event(
    &self,
    app: &mut Application,
    event: Event,
  ) -> Result<ApplicationEvent, PluginError>;
}

pub type TuiTerminal = Terminal<CrosstermBackend<std::io::Stdout>>;

#[derive(Debug, Error)]
pub enum ApplicationError {
  #[error(transparent)]
  PluginError(#[from] PluginError),
}

pub struct Application {
  plugins: Vec<Box<dyn Plugin>>,
  active_plugins: VecDeque<Box<dyn Plugin>>,
  terminal: TuiTerminal,
}

impl Application {
  pub fn new(terminal: TuiTerminal) -> Application {
    Self {
      terminal,
      plugins: Vec::new(),
      active_plugins: VecDeque::new(),
    }
  }

  pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
    self.plugins.push(plugin);
  }

  // run our application plugin system
  pub async fn run(
    &mut self,
    events: &mut EventStream,
  ) -> Result<(), ApplicationError> {
    while let Some(Ok(event)) = events.next().await {
      let mut processed_plugins = VecDeque::new();
      while let Some(plugin) = self.active_plugins.pop_front() {
        plugin.process_event(self, event.clone())?;
        processed_plugins.push_back(plugin);
      }
      self.active_plugins.append(&mut processed_plugins);
    }
    Ok(())
  }
}

pub enum ApplicationEvent {
  ActivatePlugin(String),
  DeactivatePlugin(String),
  Quit,
}

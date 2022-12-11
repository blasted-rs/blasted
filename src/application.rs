use {
  crossterm::event::{Event, EventStream},
  futures::StreamExt,
  std::collections::VecDeque,
  tui::{backend::CrosstermBackend, Terminal},
};

// Cargo as plugins
//
// Plugin Trait
//
// Everyhing is a plugin.
//
// Application
//    plugins: Vec<Plugin>,
//    active_plugins: Vec<Plugin>,
//
// fn main() {
//    ... commandline params
//    let app = Application::new();
//    app.run();
//       // claim the terminal
//       // setup plugins
//       // run eventloop and take dispatching
// }
//

// thiserror PLuginError

#[derive(Debug, thiserror::Error)]
enum PluginError {
  #[error("Could not initialize plugin {0}")]
  InitFailed(String),
}

trait Plugin {
  fn name(&self) -> &str;
  fn init(&self, app: &Application) -> Result<(), PluginError>;
  fn process_event(
    &self,
    app: &mut Application,
    event: Event,
  ) -> Result<ApplicationEvent, PluginError>;
}

type TerminalT = Terminal<CrosstermBackend<std::io::Stdout>>;

struct Application {
  plugins: Vec<Box<dyn Plugin>>,
  active_plugins: VecDeque<Box<dyn Plugin>>,
  terminal: TerminalT,
}

impl Application {
  fn new(terminal: TerminalT) -> Application {
    Self {
      terminal,
      plugins: Vec::new(),
      active_plugins: VecDeque::new(),
    }
  }

  fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
    plugin.init(self);
    self.plugins.push(plugin);
  }

  // run our application plugin system
  async fn run(&mut self, events: &mut EventStream) {
    while let Some(Ok(event)) = events.next().await {
      let mut processed_plugins = VecDeque::new();
      while let Some(plugin) = self.active_plugins.pop_front() {
        plugin.process_event(self, event.clone());
        processed_plugins.push_back(plugin);
      }
      self.active_plugins.append(&mut processed_plugins);
    }
  }
}

enum ApplicationEvent {
  ActivatePlugin(String),
  DeactivatePlugin(String),
  Quit,
}

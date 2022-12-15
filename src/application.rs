use as_any::{AsAny, Downcast};

use {
  crate::editor::Editor,
  crossterm::event::{Event as TuiEvent, EventStream},
  futures::StreamExt,
  std::{any::Any, collections::VecDeque},
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
  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

pub enum ProcessEvent {
  Consumed,
  Ignored,
}

pub trait Plugin: AsAny {
  fn id(&self) -> Option<&'static str>;
  fn type_name(&self) -> &'static str {
    std::any::type_name::<Self>()
  }
  fn init(&self, app: &Application) -> Result<(), PluginError>;
  fn process_event(
    &mut self,
    app: &mut Application,
    event: &TuiEvent,
  ) -> Result<ProcessEvent, PluginError>;

  /// Get cursor position and cursor kind.
  fn cursor(&self, _area: Rect) -> Option<(u16, u16)> {
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
  plugins: Vec<Box<dyn Plugin>>,
  active_plugins: VecDeque<Box<dyn Plugin>>,
  terminal: Option<TuiTerminal>,
  cmd: Option<UnboundedSender<Command>>,
}

impl Application {
  pub fn new(terminal: TuiTerminal) -> Application {
    Self {
      terminal: Some(terminal),
      plugins: Vec::new(),
      active_plugins: VecDeque::from_iter(vec![
        Box::<Editor>::default() as Box<dyn Plugin>
      ]),
      cmd: None,
    }
  }

  // returns the first editor it can find walking first through the active plugins
  // and then through the inactive plugins
  pub fn editor_safe(&mut self) -> Option<&mut Editor> {
    self
      .active_plugins
      .iter_mut().chain(self.plugins.iter_mut())
      .find_map(|p| p.as_mut().downcast_mut::<Editor>())
  }

  pub fn editor(&mut self) -> &mut Editor {
      self.editor_safe().expect("editor plugin not found")
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
          #[allow(clippy::single_match)]
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
          let mut processed_plugins = VecDeque::new();
          while let Some(mut plugin) = self.active_plugins.pop_back() {
            let consumed =
              matches!(plugin.process_event(self, &event)?, ProcessEvent::Consumed);
            processed_plugins.push_front(plugin);
            if consumed {
              break;
            }
          }
          // restoring the plugins
          self.active_plugins.append(&mut processed_plugins);

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

            // set the cursor position, first one wins
            let cursor = self.active_plugins
              .iter()
              .rev()
              .find_map(|p| p.cursor(area));

            // set the cursor
            let (line,pos) = cursor.unwrap_or((0,0));

            surface.set_stringn(area.width-5, area.height-1,
                                format!("{}:{}", line, pos),
                                20,
                                tui::style::Style::default());

            terminal.draw(|f| f.set_cursor(pos, line))?;

            self.terminal = Some(terminal);
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

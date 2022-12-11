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




// Application (View)
//   - manages the editor and the terminal
//   - rendering the editor to the terminal
//     - navigate the file system
//     - open files
//     - save files
//   - for handling events
//
//   the application is the view of the editor, it is the thing that the user
//   interacts with. This should be highly composable, so we can for example
//   provide crates which extend the application with new functionality.
//
//   crates can think of for exmaple:
//     - a file explorer,
//     - a git integration
//
//   for this the application object should be highly composable,
//   so that we can add new functionality to it. These are essentially
//   little applications that can be added to the main application.
//
//   the UX system is fairly simpel, there can be only one plugin UX active
//   at a time. Plugins insert themselves into the layer system.
//
//   Helix has something like a compositor, which is responsible for
//   rendering the different layers to the screen.
//
//   A plugin can place itself next to an editor view, or in new layer
//   on top of the editor view.
//
//   We can say the following, an editor view is on layer-0, which has
//   a layout. If we add something to the layout at layer-0 all editor
//   views, splits and whatever will be compressed into something smaller.
//
//   UX
//     Layers
//       0: editor view
//       1; plugin overlay
//
//
//   So how do we register a plugin?
//      we include the crate, and in our main function we forward our main
//      to the applicatoin, which will parse all the command line arguments.
//
//      before this we register our plugin.
//
//      the main loop of the application will ofcourse redner the layer-0
//      rendering of the editor view.
//
//      after this it will render all the plugins, in order they were registered.
//      if the render function returns a result, this result will be used to
//      inject the plugin in the render output. This output will then be renderedn
//
//  So how do we activate a plugin?
//      the state of the plugin will be triggered by some events, we can for example
//      have a plugin file explorer, we press <space>f and the plugin will be activated.
//
//      <space>f generates an event, which should be forwarded to the plugin. The plugin
//      activates itself, gives an output to the layer system, an now captures all events.
//
//      since pressing an key should not work in the editor view.
//
//      EventStream => Application => EditorView => View
//                                 => Plugin
//
//      If a plugin is active, the application will not forward events to the editor view.
//      This requires a dispatcher, which first queries the active plugins and if one claims
//      a class of events, the events will forward to the plugin. Or lets say, the active plugin.
//
//      This means our EditorView is in essense also a plugin.
//
//      Can we way that the last activated plugin is the active plugin, and only one
//      plugin rendering can be active. The EditorView plugin is always the last. Has the most
//      weight.
//
//      So we get a keystroke, who is responsible for handling this event?
//      The application, based on an ordered list of plugins.
//
//      Application
//         active_plugins: Vec<Plugin> => place in the list deterines the layer
//         plugins: Vec<Plugin>
//
//      ApplicationEvent::ActivatePlugin(Plugin)
//        // moves a plugin from plugins to active_plugins
//        // and does a push_front
//
//      ApplicationEvent::DeactivatePlugin(Plugin)
//        // moves a plugin from active plugin to plugins
//        // not receiving any events anymore, and no rendering
//
//      Plugin
//          id: String,
//          render() -> Result<RenderOutput>
//          process_event(event: event) -> Result<ApplicationEvent>
//
//      EditorPlugin: Plugin
//          id: editor
//          config() -> PluginConfig
//          render() -> Result<RenderOutput>
//          process_event(event: event) -> Result<ApplicationEvent>
//
//   So how do we map specific events relevant for an speicic plugin?
//
//   Once we have the plugin into our cargo.toml, our plugin will have
//   some custom events. Like in the file explorer I want map certain
//   keys to certain actions inside the plugin. Like v to view a file.
//
//   Each plugin will have its own keymap object which can be configured.
//   A keymap is a mapping for an input event to an action. The most
//   abstract is an EventMap, which maps an event to an (Plugin) action.
//
//
//   Imagine a file explorer, it needs to be able to index the file system,
//   present a view of the files system, and in the end should be able
//   to open a file. How can we do this?
//     - how can we create a view inside the terminal UX? Which works
//       with the rest of the application?
//       - one option would be to have some limitations on what can be done
//         the plugin for example should be able to claim the ux, as in
//         the focus of the ux is now on the plugin.
//       - one could say there is only one focus at a time, and that is
//
//   Editor
//   responsible for managing the documents
//     - loading documents
//     - saving documents
//     - creating new documents (buffers)
//
//   Document (Model)
//   responsible for managing the rope and provides methods to
//   update the rope
//
//   View
//   represents a view into a document

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

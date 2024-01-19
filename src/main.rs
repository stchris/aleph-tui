#![deny(clippy::unwrap_used)]

pub mod app;
pub mod event;
pub mod models;
pub mod tui;
pub mod ui;
pub mod update;

use app::App;

use color_eyre::Result;
use event::{Event, EventHandler};
use ratatui::prelude::{CrosstermBackend, Terminal};
use tui::Tui;
fn main() -> Result<()> {
    let mut app = App::new();
    app.update_status()
        .unwrap_or_else(|e| app.error_message = e.to_string());

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => update::fetch(&mut app),
            Event::Key(key_event) => update::update(&mut app, key_event),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };
    }

    tui.exit()?;
    Ok(())
}

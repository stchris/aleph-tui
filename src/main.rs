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

#[tokio::main]
async fn main() -> Result<()> {
    human_panic::setup_panic!();
    let mut app = App::new();
    let first_arg = std::env::args().nth(1);
    let quit = match first_arg {
        Some(arg) => match arg.as_str() {
            "--version" => {
                app.print_version();
                true
            }
            "--help" => {
                app.print_help();
                true
            }
            _ => {
                app.set_profile(arg)?;
                false
            }
        },
        None => false,
    };
    if quit {
        std::process::exit(0);
    };

    app.fetch()
        .await
        .unwrap_or_else(|e| app.error_message = e.to_string());

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(50);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    while !app.should_quit {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => update::fetch(&mut app).await,
            Event::Key(key_event) => update::update(&mut app, key_event).await,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };
    }

    tui.exit()?;
    Ok(())
}

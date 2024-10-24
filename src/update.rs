use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, CurrentView};

pub async fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        }
        KeyCode::Char('p') => app.toggle_profile_selector(),
        KeyCode::Up | KeyCode::Char('k') => match app.show_profile_selector() {
            true => app.profile_up().await,
            false => app.collection_up().await,
        },
        KeyCode::Down | KeyCode::Char('j') => match app.show_profile_selector() {
            true => app.profile_down().await,
            false => app.collection_down().await,
        },
        KeyCode::Enter => {
            if app.current_view == CurrentView::ProfileSwitcher {
                app.toggle_profile_selector();
            }
        }
        _ => {}
    };
}

pub(crate) async fn fetch(app: &'static mut App) {
    let elapsed = Local::now() - app.last_fetch;
    if elapsed.num_seconds() > app.config.fetch_interval {
        tokio::spawn(async move {
            app.fetch().await;
            app.last_fetch = Local::now();
        });
    }
}

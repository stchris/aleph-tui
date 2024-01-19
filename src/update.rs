use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, CurrentView};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        }
        KeyCode::Char('p') => app.toggle_profile_selector(),
        KeyCode::Up | KeyCode::Char('k') => match app.show_profile_selector() {
            true => app.profile_up(),
            false => app.collection_up(),
        },
        KeyCode::Down | KeyCode::Char('j') => match app.show_profile_selector() {
            true => app.profile_down(),
            false => app.collection_down(),
        },
        KeyCode::Enter => {
            if app.current_view == CurrentView::ProfileSwitcher {
                app.toggle_profile_selector();
            }
        }
        _ => {}
    };
}

pub(crate) fn fetch(app: &mut App) {
    let elapsed = Local::now() - app.last_fetch;
    if elapsed.num_seconds() > app.config.fetch_interval {
        match app.update_status() {
            Ok(()) => {
                app.last_fetch = Local::now();
                app.error_message = String::default();
            }
            Err(e) => app.error_message = e.to_string(),
        }
    }
}

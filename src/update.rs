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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::{create_test_app, create_test_app_with_collections};

    #[test]
    fn test_quit_on_q() {
        let mut app = create_test_app();
        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        );
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_on_escape() {
        let mut app = create_test_app();
        update(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let mut app = create_test_app();
        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        );
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_on_ctrl_shift_c() {
        let mut app = create_test_app();
        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL),
        );
        assert!(app.should_quit);
    }

    #[test]
    fn test_c_without_ctrl_does_not_quit() {
        let mut app = create_test_app();
        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
        );
        assert!(!app.should_quit);
    }

    #[test]
    fn test_toggle_profile_selector() {
        let mut app = create_test_app();
        assert_eq!(app.current_view, CurrentView::Main);

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
        );
        assert_eq!(app.current_view, CurrentView::ProfileSwitcher);

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
        );
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_navigation_down_in_main_view() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(0));

        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_navigation_up_in_main_view() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(1));

        update(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.collection_tablestate.selected(), Some(0));
    }

    #[test]
    fn test_navigation_in_profile_selector() {
        let mut app = create_test_app();
        app.toggle_profile_selector();
        assert_eq!(app.current_profile, 0);

        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.current_profile, 1);

        update(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.current_profile, 0);
    }

    #[test]
    fn test_vim_key_j_down() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(0));

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        );
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_vim_key_k_up() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(1));

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
        );
        assert_eq!(app.collection_tablestate.selected(), Some(0));
    }

    #[test]
    fn test_enter_closes_profile_selector() {
        let mut app = create_test_app();
        app.toggle_profile_selector();
        assert_eq!(app.current_view, CurrentView::ProfileSwitcher);

        update(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_enter_in_main_view_no_effect() {
        let mut app = create_test_app();
        assert_eq!(app.current_view, CurrentView::Main);

        update(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        // Should still be in main view
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_unhandled_key_no_effect() {
        let mut app = create_test_app();
        let initial_view = app.current_view.clone();
        let initial_quit = app.should_quit;

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );

        // State should be unchanged
        assert_eq!(app.current_view, initial_view);
        assert_eq!(app.should_quit, initial_quit);
    }
}

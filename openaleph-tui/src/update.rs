use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, CurrentView, Tab};

pub fn update(app: &mut App, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Esc if app.show_help() => app.toggle_help(),
        KeyCode::Char('?')
            if !key_event
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER) =>
        {
            app.toggle_help()
        }
        KeyCode::Char('c') | KeyCode::Char('C')
            if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.quit()
        }
        _ if app.show_help() => {}
        KeyCode::Esc => app.quit(),
        KeyCode::Char('q')
            if !matches!(app.active_tab, Tab::Search | Tab::Investigations)
                || app.show_profile_selector() =>
        {
            app.quit()
        }
        KeyCode::Char('p') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_profile_selector()
        }
        KeyCode::Char('p')
            if !matches!(app.active_tab, Tab::Search | Tab::Investigations)
                || app.show_profile_selector() =>
        {
            app.toggle_profile_selector()
        }
        KeyCode::Tab if !app.show_profile_selector() => app.next_tab(),
        KeyCode::BackTab if !app.show_profile_selector() => app.previous_tab(),
        KeyCode::Up | KeyCode::Char('k') => match app.show_profile_selector() {
            true => app.profile_up(),
            false if app.active_tab == Tab::Search && key_event.code == KeyCode::Up => {
                app.search_result_up()
            }
            false if app.active_tab == Tab::Investigations && key_event.code == KeyCode::Up => {
                app.investigation_up()
            }
            false if app.active_tab == Tab::Status => app.collection_up(),
            false
                if app.active_tab == Tab::Search
                    && !key_event.modifiers.intersects(
                        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                    ) =>
            {
                app.push_search_char('k')
            }
            false
                if app.active_tab == Tab::Investigations
                    && !key_event.modifiers.intersects(
                        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                    ) =>
            {
                app.push_investigations_search_char('k')
            }
            false => {}
        },
        KeyCode::Down | KeyCode::Char('j') => match app.show_profile_selector() {
            true => app.profile_down(),
            false if app.active_tab == Tab::Search && key_event.code == KeyCode::Down => {
                app.search_result_down()
            }
            false if app.active_tab == Tab::Investigations && key_event.code == KeyCode::Down => {
                app.investigation_down()
            }
            false if app.active_tab == Tab::Status => app.collection_down(),
            false
                if app.active_tab == Tab::Search
                    && !key_event.modifiers.intersects(
                        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                    ) =>
            {
                app.push_search_char('j')
            }
            false
                if app.active_tab == Tab::Investigations
                    && !key_event.modifiers.intersects(
                        KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                    ) =>
            {
                app.push_investigations_search_char('j')
            }
            false => {}
        },
        KeyCode::Enter if app.current_view == CurrentView::ProfileSwitcher => {
            app.toggle_profile_selector();
        }
        KeyCode::Enter if app.active_tab == Tab::Search => app.start_search(),
        KeyCode::Enter if app.active_tab == Tab::Investigations => {
            app.start_investigations_search()
        }
        KeyCode::Backspace if app.active_tab == Tab::Search => app.pop_search_char(),
        KeyCode::Backspace if app.active_tab == Tab::Investigations => {
            app.pop_investigations_search_char()
        }
        KeyCode::Char(character)
            if app.active_tab == Tab::Search
                && !app.show_profile_selector()
                && !key_event.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                ) =>
        {
            app.push_search_char(character);
        }
        KeyCode::Char(character)
            if app.active_tab == Tab::Investigations
                && !app.show_profile_selector()
                && !key_event.modifiers.intersects(
                    KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER,
                ) =>
        {
            app.push_investigations_search_char(character);
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
        app.active_tab = Tab::Status;
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
        assert_eq!(app.search_query, "c");
    }

    #[test]
    fn test_toggle_profile_selector() {
        let mut app = create_test_app();
        assert_eq!(app.current_view, CurrentView::Main);

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL),
        );
        assert_eq!(app.current_view, CurrentView::ProfileSwitcher);

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE),
        );
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_question_mark_toggles_help() {
        let mut app = create_test_app();

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT),
        );
        assert_eq!(app.current_view, CurrentView::Help);

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT),
        );
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_escape_closes_help_without_quitting() {
        let mut app = create_test_app();
        app.toggle_help();

        update(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));

        assert_eq!(app.current_view, CurrentView::Main);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_help_blocks_other_input() {
        let mut app = create_test_app();
        app.toggle_help();

        update(
            &mut app,
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
        );

        assert!(app.search_query.is_empty());
        assert_eq!(app.current_view, CurrentView::Help);
    }

    #[test]
    fn test_navigation_down_in_main_view() {
        let mut app = create_test_app_with_collections();
        app.active_tab = Tab::Status;
        app.collection_tablestate.select(Some(0));

        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_navigation_up_in_main_view() {
        let mut app = create_test_app_with_collections();
        app.active_tab = Tab::Status;
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
        app.active_tab = Tab::Status;
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
        app.active_tab = Tab::Status;
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

    #[tokio::test]
    async fn test_enter_searches_investigations() {
        let mut app = create_test_app();
        app.active_tab = Tab::Investigations;

        update(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.is_searching_investigations);
    }

    #[tokio::test]
    async fn test_enter_submits_empty_search() {
        let mut app = create_test_app();
        assert!(app.search_query.is_empty());

        update(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.is_searching);
    }

    #[test]
    fn test_investigations_query_editing() {
        let mut app = create_test_app();
        app.active_tab = Tab::Investigations;

        for character in ['q', 'u', 'i', 'p'] {
            update(
                &mut app,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
        }
        update(
            &mut app,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );

        assert_eq!(app.investigations_query, "qui");
        assert!(!app.should_quit);
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_search_query_editing() {
        let mut app = create_test_app();

        for character in ['q', 'u', 'i', 'p'] {
            update(
                &mut app,
                KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            );
        }
        update(
            &mut app,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        );

        assert_eq!(app.search_query, "qui");
        assert!(!app.should_quit);
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_search_result_navigation() {
        let mut app = create_test_app();
        app.search_response.results = vec![Default::default(), Default::default()];
        app.search_list_state.select(Some(0));

        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.search_list_state.selected(), Some(1));
        update(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.search_list_state.selected(), Some(0));
    }

    #[test]
    fn test_investigation_navigation() {
        let mut app = create_test_app();
        app.active_tab = Tab::Investigations;
        app.investigations_response.results = vec![Default::default(), Default::default()];
        app.investigations_list_state.select(Some(0));

        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(app.investigations_list_state.selected(), Some(1));
        update(&mut app, KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(app.investigations_list_state.selected(), Some(0));
    }

    #[test]
    fn test_tab_cycles_forward() {
        let mut app = create_test_app();
        assert_eq!(app.active_tab, Tab::Search);

        update(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_tab, Tab::Investigations);
        update(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_tab, Tab::Status);
        update(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.active_tab, Tab::Search);
    }

    #[test]
    fn test_backtab_wraps_to_status() {
        let mut app = create_test_app();

        update(
            &mut app,
            KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
        );

        assert_eq!(app.active_tab, Tab::Status);
    }

    #[test]
    fn test_tab_does_not_change_while_profile_selector_is_open() {
        let mut app = create_test_app();
        app.toggle_profile_selector();

        update(&mut app, KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

        assert_eq!(app.active_tab, Tab::Search);
    }

    #[test]
    fn test_collection_navigation_is_limited_to_status_tab() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(0));

        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));

        assert_eq!(app.collection_tablestate.selected(), Some(0));
    }
}

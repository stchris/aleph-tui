use chrono::{NaiveDateTime, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Frame,
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::app::App;

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn render(app: &mut App, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.size());
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default());

    let text = vec![
        Line::from(match &app.metadata.app.title {
            Some(title) => format!(
                "{} ({}): {} jobs running",
                title,
                app.current_profile().name,
                app.status.total
            ),
            None => format!(
                "({}): {} jobs running",
                app.current_profile().name,
                app.status.total
            ),
        }),
        Line::from(
            match (&app.metadata.app.version, &app.metadata.app.ftm_version) {
                (Some(aleph), Some(ftm)) => format!("version: {}, followthemoney: {}", aleph, ftm),
                (None, Some(ftm)) => format!("followthemoney: {}", ftm),
                (Some(aleph), None) => format!("version: {}", aleph),
                (None, None) => String::default(),
            },
        ),
    ];
    let title = Paragraph::new(text)
        .style(Style::default().fg(Color::Green))
        .block(title_block);
    f.render_widget(title, chunks[0]);

    let mut rows = Vec::new();
    let now = Utc::now().naive_utc();
    for result in &app.status.results {
        let last_update = match result.last_update.clone() {
            Some(t) => {
                let last_update = NaiveDateTime::parse_from_str(&t, "%Y-%m-%dT%H:%M:%S.%f")
                    .expect("Failed to parse last_update timestamp");
                let last_update = last_update - now;
                HumanTime::from(last_update).to_text_en(Accuracy::Precise, Tense::Present)
            }
            None => "".to_string(),
        };

        rows.push(Row::new(vec![
            result.collection.id.to_string(),
            result.collection.label.to_string(),
            result.finished.to_string(),
            result.running.to_string(),
            result.pending.to_string(),
            last_update,
        ]))
    }
    let widths = [
        Constraint::Length(5),
        Constraint::Min(30),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(10),
        Constraint::Length(25),
    ];
    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                "ID",
                "Label",
                "Finished",
                "Running",
                "Pending",
                "Last update",
            ])
            .style(
                Style::new()
                    .bold()
                    .blue()
                    .underline_color(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED),
            )
            .bottom_margin(1),
        )
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    f.render_stateful_widget(table, chunks[1], &mut app.collection_tablestate);

    f.render_widget(
        Paragraph::new(app.error_message.to_string()).style(Style::new().red()),
        chunks[2],
    );

    let status_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25),
            Constraint::Length(50),
            Constraint::Min(1),
        ])
        .split(chunks[3]);
    f.render_widget(
        Block::default().title(format!("aleph-tui version {}", app.version)),
        status_bar_chunks[0],
    );
    let fetching_icon = match app.is_fetching {
        true => "📥",
        false => "",
    };
    let last_fetch_text = format!(
        "last fetch: {} {}",
        HumanTime::from(app.last_fetch),
        fetching_icon
    );
    f.render_widget(
        Block::default()
            .title(last_fetch_text)
            .title_alignment(Alignment::Right),
        status_bar_chunks[1],
    );
    f.render_widget(
        Block::default()
            .title("Shortcuts: `q`, `^C`, `Esc` - quit, `p` - select profile")
            .title_alignment(Alignment::Right),
        status_bar_chunks[2],
    );

    if app.show_profile_selector() {
        let popup_block = Block::default()
            .title("Select profile")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue));

        let area = centered_rect(40, 25, f.size());
        f.render_widget(popup_block.clone(), area);

        let mut rows = Vec::new();
        for (idx, profile) in app.config.profiles.clone().into_iter().enumerate() {
            rows.push(Row::new([profile.name.to_string()]));
            if app.current_profile == profile.index {
                app.profile_tablestate.select(Some(idx))
            }
        }
        let profile_table = Table::new(rows, [Constraint::Min(15)])
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");
        f.render_stateful_widget(
            profile_table,
            popup_block.inner(area),
            &mut app.profile_tablestate,
        );
    }
}

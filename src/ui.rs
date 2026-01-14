use chrono::Local;
use humanize_duration::prelude::DurationExt;
use humanize_duration::Truncate;
use num_format::{Locale, ToFormattedString};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    prelude::Frame,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Padding, Paragraph, Row, Table},
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
    let chunks = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(1),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Start)
        .split(f.area());
    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Rounded);

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
                (Some(aleph), Some(ftm)) => format!("version: {aleph}, followthemoney: {ftm}"),
                (None, Some(ftm)) => format!("followthemoney: {ftm}"),
                (Some(aleph), None) => format!("version: {aleph}"),
                (None, None) => String::default(),
            },
        ),
    ];
    let title = Paragraph::new(text).block(title_block);
    f.render_widget(title, chunks[0]);

    let mut rows = Vec::new();
    for result in &app.status.results {
        let remaining = match result.remaining_time {
            Some(t) => format!("{}", t.human(Truncate::Second)),
            None => "not sure. soon?".to_string(),
        };

        let collection_id = match &result.collection {
            Some(c) => c.id.to_string(),
            None => "-".to_string(),
        };
        let collection_label = match &result.collection {
            Some(c) => c.label.to_string(),
            None => result.name.clone(),
        };
        rows.push(Row::new(vec![
            collection_id,
            collection_label,
            result.finished.to_formatted_string(&Locale::en),
            result.doing.to_formatted_string(&Locale::en),
            result.todo.to_formatted_string(&Locale::en),
            remaining,
        ]))
    }
    let widths = [
        Constraint::Length(5),
        Constraint::Min(20),
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
                "Remaining",
            ])
            .bottom_margin(1),
        )
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    f.render_stateful_widget(table, chunks[1], &mut app.collection_tablestate);

    if let Some(index) = app.collection_tablestate.selected() {
        let result = &app.status.results[index];
        let mut task_rows = vec![];
        for batch in result.batches.clone() {
            for queue in batch.queues {
                for task in queue.tasks {
                    task_rows.push(Row::new(vec![
                        task.name.clone(),
                        task.total.to_formatted_string(&Locale::en),
                        task.active.to_formatted_string(&Locale::en),
                        task.finished.to_formatted_string(&Locale::en),
                        task.todo.to_formatted_string(&Locale::en),
                        task.doing.to_formatted_string(&Locale::en),
                        task.succeeded.to_formatted_string(&Locale::en),
                        task.failed.to_formatted_string(&Locale::en),
                    ]));
                }
            }
        }
        let title = match &result.collection {
            Some(col) => format!("Collection {} <{}>", col.collection_id, col.label),
            None => "Details".to_string(),
        };
        let info_block = Block::default()
            .title(title)
            .padding(Padding::new(1, 1, 1, 1))
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded);
        let task_widths = [
            Constraint::Min(15),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(8),
        ];
        let task_table = Table::new(task_rows, task_widths)
            .header(
                Row::new(vec![
                    "Task Name",
                    "Total",
                    "Active",
                    "Finished",
                    "Todo",
                    "Doing",
                    "Succeeded",
                    "Failed",
                ])
                .bottom_margin(1),
            )
            .block(info_block);
        f.render_widget(task_table, chunks[2]);
    }

    f.render_widget(
        Paragraph::new(app.error_message.to_string()).style(Style::new().red()),
        chunks[3],
    );

    let status_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Min(1), Constraint::Min(25)])
        .split(chunks[4]);
    f.render_widget(
        Block::default().title(format!("aleph-tui version {}", app.version)),
        status_bar_chunks[0],
    );
    let fetching_icon = match app.is_fetching {
        true => "🔄",
        false => "",
    };
    let last_fetch = Local::now() - app.last_fetch;
    let last_fetch = last_fetch.human(Truncate::Second);
    let last_fetch_text = format!(
        "{} fetching every {}s - last fetch {} ago",
        fetching_icon, app.config.fetch_interval, last_fetch,
    );
    f.render_widget(
        Block::default()
            .title(last_fetch_text)
            .title_alignment(Alignment::Left),
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
            .borders(Borders::ALL);

        let area = centered_rect(40, 25, f.area());
        f.render_widget(popup_block.clone(), area);

        let mut rows = Vec::new();
        for (idx, profile) in app.config.profiles.clone().into_iter().enumerate() {
            rows.push(Row::new([profile.name.to_string()]));
            if app.current_profile == profile.index {
                app.profile_tablestate.select(Some(idx))
            }
        }
        let profile_table = Table::new(rows, [Constraint::Min(15)])
            .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">>");
        f.render_stateful_widget(
            profile_table,
            popup_block.inner(area),
            &mut app.profile_tablestate,
        );
    }
}

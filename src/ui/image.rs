use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::enums::image::ImageRow;
use crate::models::app::App;
use crate::models::generics::GroupHeader;
use bollard::models::ImageSummary;

pub fn render_images_tab(app: &mut App, frame: &mut Frame, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let visible_rows = app.images_tab.get_visible_rows();

    let rows: Vec<Row> = visible_rows
        .iter()
        .map(|row_item| match row_item {
            ImageRow::GroupHeader(GroupHeader {
                name,
                total_count,
                is_expanded,
                ..
            }) => {
                let icon = if *is_expanded { "▼ 📁" } else { "▶ 📁" };
                let group_name_cell = Cell::from(format!("{} {}", icon, name)).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );
                let info_cell = Cell::from(format!("({} images)", total_count))
                    .style(Style::default().fg(Color::DarkGray));
                let status_cell = Cell::from("◈ compose group").cyan();

                Row::new(vec![group_name_cell, info_cell, status_cell])
                    .style(Style::default().bg(Color::Reset))
            }
            ImageRow::ChildImage {
                image,
                is_last_in_group,
            } => {
                let tag = format_tag(image);
                let branch = if *is_last_in_group {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                let name_cell = Cell::from(format!("{}{}", branch, tag));
                let id_cell = Cell::from(format_id(&image.id));
                let size_cell = Cell::from(format_size(image.size));

                Row::new(vec![name_cell, id_cell, size_cell])
            }
            ImageRow::StandaloneImage { image } => {
                let tag = format_tag(image);

                let name_cell = Cell::from(format!("◈ {}", tag)).bold();
                let id_cell = Cell::from(format_id(&image.id));
                let size_cell = Cell::from(format_size(image.size));

                Row::new(vec![name_cell, id_cell, size_cell])
            }
        })
        .collect();

    let selected_index = app.images_tab.table_state.selected();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["REPOSITORY:TAG", "IMAGE ID", "SIZE"])
            .style(Style::default().fg(Color::Yellow)),
    )
    .block(Block::default().borders(Borders::ALL).title(" Images "))
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(table, main_chunks[0], &mut app.images_tab.table_state);

    let inspector_rows = app.images_tab.get_visible_rows();
    let selected_info = selected_index.and_then(|idx| inspector_rows.get(idx));

    match selected_info {
        Some(ImageRow::ChildImage { image, .. }) | Some(ImageRow::StandaloneImage { image }) => {
            render_image_inspector(frame, main_chunks[1], image);
        }
        _ => {
            let placeholder = Paragraph::new("Select an image to view details.").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 🖼️ Image Inspector "),
            );
            frame.render_widget(placeholder, main_chunks[1]);
        }
    }
}

fn render_image_inspector(frame: &mut Frame, area: Rect, image: &ImageSummary) {
    let mut details = vec![
        Line::from(vec![
            Span::styled(
                "ID: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(image.id.clone()),
        ]),
        Line::from(vec![
            Span::styled(
                "Repository Tags: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(if image.repo_tags.is_empty() {
                "<none>".to_string()
            } else {
                image.repo_tags.join(", ")
            }),
        ]),
        Line::from(vec![
            Span::styled(
                "Size: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format_size(image.size)),
        ]),
        Line::from(vec![
            Span::styled(
                "Containers Using: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(image.containers.to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "Created: ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format_timestamp(image.created)),
        ]),
    ];

    if !image.labels.is_empty() {
        details.push(Line::from(""));
        details.push(Line::from(Span::styled(
            "Labels:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        for (k, v) in &image.labels {
            details.push(Line::from(vec![
                Span::styled(format!("  {}: ", k), Style::default().fg(Color::DarkGray)),
                Span::raw(v.clone()),
            ]));
        }
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 🖼️ Image Inspector ");
    let paragraph = Paragraph::new(details).block(block);
    frame.render_widget(paragraph, area);
}

fn format_tag(image: &ImageSummary) -> String {
    if let Some(tag) = image.repo_tags.first() {
        if tag != "<none>:<none>" {
            return tag.clone();
        }
    }
    if let Some(digest) = image.repo_digests.first() {
        if let Some((repo, _)) = digest.split_once('@') {
            return format!("{} (<none>)", repo);
        }
    }
    "<none>:<none>".to_string()
}

fn format_id(id: &str) -> String {
    let clean = id.strip_prefix("sha256:").unwrap_or(id);
    if clean.len() > 12 {
        clean[..12].to_string()
    } else {
        clean.to_string()
    }
}

fn format_size(bytes: i64) -> String {
    let result:String;

    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let size = bytes as f64;
    
    if size >= GB {
        result = format!("{:.2} GB", size / GB);
    } else if size >= MB {
        result = format!("{:.1} MB", size / MB);
    } else if size >= KB {
        result = format!("{:.1} KB", size / KB);
    } else {
        result = format!("{} B", bytes);
    }

    result
}

fn format_timestamp(timestamp: i64) -> String {
    let result:String;
    if timestamp == 0 {
        return "N/A".to_string();
    }
    use std::time::{Duration, UNIX_EPOCH};
    let d = UNIX_EPOCH + Duration::from_secs(timestamp as u64);
    result = format!("{:?}", d);
    result
}

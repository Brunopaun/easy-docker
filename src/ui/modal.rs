use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::models::app::{App, DeleteTarget};

pub fn render_confirm_delete_modal(app: &App, frame: &mut Frame, area: Rect) {
    let target = match &app.delete_target {
        Some(t) => t,
        None => return,
    };

    let popup_area = centered_rect(60, 35, area);

    frame.render_widget(Clear, popup_area);

    let (item_type, item_name) = match target {
        DeleteTarget::Volume(name) => ("Volume", name.clone()),
        DeleteTarget::Container(id) => {
            let short_id = id.chars().take(12).collect::<String>();
            ("Container", short_id)
        }
        DeleteTarget::Image(id) => {
            let clean_id = id.strip_prefix("sha256:").unwrap_or(id);
            let short_id = clean_id.chars().take(12).collect::<String>();
            ("Image", short_id)
        }
        DeleteTarget::Network(id) => {
            let short_id = id.chars().take(12).collect::<String>();
            ("Network", short_id)
        }
        DeleteTarget::GroupContainers(group_name, ids) => (
            "Compose Group",
            format!("{} ({} containers)", group_name, ids.len()),
        ),
        DeleteTarget::GroupVolumes(group_name, names) => (
            "Volume Group",
            format!("{} ({} volumes)", group_name, names.len()),
        ),
        DeleteTarget::GroupImages(group_name, ids) => (
            "Image Group",
            format!("{} ({} images)", group_name, ids.len()),
        ),
        DeleteTarget::GroupNetworks(group_name, ids) => (
            "Network Group",
            format!("{} ({} networks)", group_name, ids.len()),
        ),
    };

    let warning_msg = match target {
        DeleteTarget::Volume(_) => "This action permanently removes the volume data.",
        DeleteTarget::Container(_) => "This action permanently deletes the container.",
        DeleteTarget::Image(_) => "This action permanently removes the image from Docker.",
        DeleteTarget::Network(_) => "This action permanently removes the network from Docker.",
        DeleteTarget::GroupContainers(_, _) => "This action permanently deletes ALL containers in this compose group.",
        DeleteTarget::GroupVolumes(_, _) => "This action permanently deletes ALL volumes in this compose group.",
        DeleteTarget::GroupImages(_, _) => "This action permanently deletes ALL images in this compose group.",
        DeleteTarget::GroupNetworks(_, _) => "This action permanently deletes ALL networks in this compose group.",
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Are you sure you want to delete this ", Style::default().fg(Color::White)),
            Span::styled(item_type.to_lowercase(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("?"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Target: ", Style::default().fg(Color::Gray)),
            Span::styled(item_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("⚠️ Warning: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(warning_msg, Style::default().fg(Color::LightRed)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" [y / Enter] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("Confirm Delete    "),
            Span::styled(" [n / Esc] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Cancel"),
        ]),
    ];

    let block = Block::default()
        .title(Span::styled(
            format!(" ⚠️ Confirm Delete {} ", item_type),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

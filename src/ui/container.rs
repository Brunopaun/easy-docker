use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::components::container::ContainerInspectorView;
use crate::enums::container::ContainerRow;
use crate::models::app::App;
use crate::models::generics::GroupHeader;

pub fn render_containers_tab(app: &mut App, frame: &mut Frame, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let visible_rows = app.containers_tab.get_visible_rows();

    let rows: Vec<Row> = visible_rows
        .iter()
        .map(|row_item| match row_item {
            ContainerRow::GroupHeader(GroupHeader {
                name,
                total_count,
                running_count,
                is_expanded,
            }) => {
                let icon = if *is_expanded { "▼ 📁" } else { "▶ 📁" };
                let group_name_cell = Cell::from(format!("{} {}", icon, name))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
                let info_cell = Cell::from(format!("({}/{} running)", running_count, total_count))
                    .style(Style::default().fg(Color::DarkGray));
                let status_cell = if *running_count > 0 {
                    Cell::from("🟢 active group").green()
                } else {
                    Cell::from("🔴 inactive group").red()
                };

                Row::new(vec![group_name_cell, info_cell, status_cell])
                    .style(Style::default().bg(Color::Reset))
            }
            ContainerRow::ChildContainer {
                container,
                is_last_in_group,
            } => {
                let raw_name = container
                    .names
                    .as_ref()
                    .and_then(|n| n.first())
                    .map(|s| s.as_str())
                    .unwrap_or("N/A");
                let clean_name = raw_name.strip_prefix('/').unwrap_or(raw_name);
                let branch = if *is_last_in_group {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                let name_cell = Cell::from(format!("{}{}", branch, clean_name));
                let image = container.image.as_deref().unwrap_or("N/A").to_string();
                let state = container
                    .state
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or("N/A");

                let state_cell = match state {
                    "running" => Cell::from("🟢 running").green(),
                    "exited" => Cell::from("🔴 stopped").red(),
                    _ => Cell::from(state.to_string()).yellow(),
                };

                Row::new(vec![name_cell, Cell::from(image), state_cell])
            }
            ContainerRow::StandaloneContainer { container } => {
                let raw_name = container
                    .names
                    .as_ref()
                    .and_then(|n| n.first())
                    .map(|s| s.as_str())
                    .unwrap_or("N/A");
                let clean_name = raw_name.strip_prefix('/').unwrap_or(raw_name);

                let name_cell = Cell::from(format!("◼ {}", clean_name)).bold();
                let image = container.image.as_deref().unwrap_or("N/A").to_string();
                let state = container
                    .state
                    .as_ref()
                    .map(|s| s.as_ref())
                    .unwrap_or("N/A");

                let state_cell = match state {
                    "running" => Cell::from("🟢 running").green(),
                    "exited" => Cell::from("🔴 stopped").red(),
                    _ => Cell::from(state.to_string()).yellow(),
                };

                Row::new(vec![name_cell, Cell::from(image), state_cell])
            }
        })
        .collect();

    drop(visible_rows);

    let selected_index = app.containers_tab.table_state.selected();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(45),
            Constraint::Percentage(35),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec!["NAME", "IMAGE / INFO", "STATUS"])
            .style(Style::default().fg(Color::Yellow)),
    )
    .block(Block::default().borders(Borders::ALL).title(" Containers "))
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(
        table,
        main_chunks[0],
        &mut app.containers_tab.table_state,
    );

    let inspector_rows = app.containers_tab.get_visible_rows();
    let selected_info = selected_index.and_then(|idx| inspector_rows.get(idx));

    match selected_info {
        Some(ContainerRow::ChildContainer { container, .. })
        | Some(ContainerRow::StandaloneContainer { container }) => {
            let raw_name = container
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|s| s.as_str())
                .unwrap_or("N/A");
            let name = raw_name.strip_prefix('/').unwrap_or(raw_name);

            if app.containers_tab.inspector_view == ContainerInspectorView::Logs {
                let auto_scroll_status = if app.containers_tab.auto_scroll {
                    "Auto-Scroll: [ON]"
                } else {
                    "Auto-Scroll: [OFF]"
                };

                let title_text = format!(
                    " 📋 Logs: {} │ {} │ lines: {} [d: Details] ",
                    name,
                    auto_scroll_status,
                    app.containers_tab.logs.len()
                );

                let content_lines: Vec<Line> = if app.containers_tab.is_loading_logs {
                    vec![Line::from(Span::styled(
                        "⏳ Loading logs...",
                        Style::default().fg(Color::Yellow),
                    ))]
                } else if app.containers_tab.logs.is_empty() {
                    vec![Line::from(Span::styled(
                        "No log output available.",
                        Style::default().fg(Color::DarkGray),
                    ))]
                } else {
                    app.containers_tab
                        .logs
                        .iter()
                        .map(|l| Line::from(Span::raw(l.clone())))
                        .collect()
                };

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        title_text,
                        Style::default().fg(Color::Cyan).bold(),
                    ));

                let scroll_offset = if app.containers_tab.auto_scroll && !app.containers_tab.logs.is_empty() {
                    app.containers_tab.logs.len().saturating_sub(1) as u16
                } else {
                    app.containers_tab.logs_scroll_offset as u16
                };

                let logs_widget = Paragraph::new(content_lines)
                    .block(block)
                    .scroll((scroll_offset, 0));

                frame.render_widget(logs_widget, main_chunks[1]);
            } else {
                let id_short = container
                    .id
                    .as_deref()
                    .map(|id| id.chars().take(12).collect::<String>())
                    .unwrap_or_else(|| "N/A".into());
                let image = container.image.as_deref().unwrap_or("N/A");
                let state = container
                    .state
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "N/A".into());
                let status = container.status.as_deref().unwrap_or("N/A");
                let command = container.command.as_deref().unwrap_or("N/A");

                let ports_text = container
                    .ports
                    .as_ref()
                    .map(|ports| {
                        let formatted: Vec<String> = ports
                            .iter()
                            .map(|p| {
                                format!(
                                    "{}:{}->{}/{}",
                                    p.ip.as_deref().unwrap_or(""),
                                    p.public_port.unwrap_or(0),
                                    p.private_port,
                                    p.typ
                                        .as_ref()
                                        .map(|t| t.to_string())
                                        .unwrap_or_default()
                                )
                            })
                            .collect();
                        if formatted.is_empty() {
                            "None".into()
                        } else {
                            formatted.join(", ")
                        }
                    })
                    .unwrap_or_else(|| "None".into());

                let text = vec![
                    Line::from(vec![
                        Span::raw("Name: "),
                        Span::styled(name, Style::default().fg(Color::Cyan).bold()),
                    ]),
                    Line::from(vec![
                        Span::raw("ID: "),
                        Span::styled(id_short, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![Span::raw("Image: "), Span::raw(image)]),
                    Line::from(vec![
                        Span::raw("State: "),
                        Span::styled(
                            state.clone(),
                            if state == "running" {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default().fg(Color::Red)
                            },
                        ),
                    ]),
                    Line::from(vec![Span::raw("Status: "), Span::raw(status)]),
                    Line::from(vec![Span::raw("Command: "), Span::raw(command)]),
                    Line::from(vec![Span::raw("Ports: "), Span::raw(ports_text)]),
                ];

                let inspector = Paragraph::new(text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" Container Details ({}) [l: Logs] ", name)),
                );
                frame.render_widget(inspector, main_chunks[1]);
            }
        }
        Some(ContainerRow::GroupHeader(GroupHeader {
            name,
            total_count,
            running_count,
            ..
        })) => {
            let text = vec![
                Line::from(vec![
                    Span::raw("Group: "),
                    Span::styled(*name, Style::default().fg(Color::Cyan).bold()),
                ]),
                Line::from(vec![
                    Span::raw("Total Containers: "),
                    Span::raw(total_count.to_string()),
                ]),
                Line::from(vec![
                    Span::raw("Running Containers: "),
                    Span::raw(running_count.to_string()),
                ]),
            ];
            let inspector = Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Compose Project ({}) ", name)),
            );
            frame.render_widget(inspector, main_chunks[1]);
        }
        None => {
            let inspector_block = Paragraph::new("Select a container to inspect").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Container Stats "),
            );
            frame.render_widget(inspector_block, main_chunks[1]);
        }
    }
}
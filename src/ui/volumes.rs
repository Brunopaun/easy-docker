use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::models::generics::GroupHeader;
use crate::components::volumes::VolumeRow;
use crate::models::app::App;

pub fn render_volumes_tab(app: &mut App, frame: &mut Frame, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let visible_rows = app.volumes_tab.get_visible_rows();

    let rows: Vec<Row> = visible_rows
        .iter()
        .map(|row_item| match row_item {
            VolumeRow::GroupHeader(GroupHeader {
                name,
                total_count,
                is_expanded,
                ..
            }) => {
                let icon = if *is_expanded { "▼ 📁" } else { "▶ 📁" };
                let group_name_cell = Cell::from(format!("{} {}", icon, name))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
                let info_cell = Cell::from(format!("({} volumes)", total_count))
                    .style(Style::default().fg(Color::DarkGray));
                let status_cell = Cell::from("📁 compose group").cyan();

                Row::new(vec![group_name_cell, info_cell, status_cell])
                    .style(Style::default().bg(Color::Reset))
            }
            VolumeRow::ChildVolume {
                volume,
                is_last_in_group,
            } => {
                let raw_name = &volume.name;
                let branch = if *is_last_in_group {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                let name_cell = Cell::from(format!("{}{}", branch, raw_name));
                let driver_cell = Cell::from(volume.driver.clone());
                let mountpoint_cell = Cell::from(volume.mountpoint.clone());

                Row::new(vec![name_cell, driver_cell, mountpoint_cell])
            }
            VolumeRow::StandaloneVolume { volume } => {
                let raw_name = &volume.name;

                let name_cell = Cell::from(format!("📁 {}", raw_name)).bold();
                let driver_cell = Cell::from(volume.driver.clone());
                let mountpoint_cell = Cell::from(volume.mountpoint.clone());

                Row::new(vec![name_cell, driver_cell, mountpoint_cell])
            }
        })
        .collect();

    let selected_index = app.volumes_tab.table_state.selected();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ],
    )
    .header(
        Row::new(vec!["NAME", "DRIVER", "MOUNTPOINT"])
            .style(Style::default().fg(Color::Yellow)),
    )
    .block(Block::default().borders(Borders::ALL).title(" Volumes "))
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(
        table,
        main_chunks[0],
        &mut app.volumes_tab.table_state,
    );

    let inspector_rows = app.volumes_tab.get_visible_rows();
    let selected_info = selected_index.and_then(|idx| inspector_rows.get(idx));

    match selected_info {
        Some(VolumeRow::ChildVolume { volume, .. })
        | Some(VolumeRow::StandaloneVolume { volume }) => {
            let name = &volume.name;
            let driver = &volume.driver;
            let mountpoint = &volume.mountpoint;
            let created_at = volume.created_at.as_deref().unwrap_or("N/A");
            let scope = volume
                .scope
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "local".into());
            let labels_count = volume.labels.len();
            let options_count = volume.options.len();

            let text = vec![
                Line::from(vec![
                    Span::raw("Name: "),
                    Span::styled(name.as_str(), Style::default().fg(Color::Cyan).bold()),
                ]),
                Line::from(vec![
                    Span::raw("Driver: "),
                    Span::styled(driver.as_str(), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("Scope: "),
                    Span::raw(scope),
                ]),
                Line::from(vec![
                    Span::raw("Created At: "),
                    Span::raw(created_at),
                ]),
                Line::from(vec![
                    Span::raw("Mountpoint: "),
                    Span::raw(mountpoint.as_str()),
                ]),
                Line::from(vec![
                    Span::raw("Labels: "),
                    Span::raw(format!("{} label(s)", labels_count)),
                ]),
                Line::from(vec![
                    Span::raw("Options: "),
                    Span::raw(format!("{} option(s)", options_count)),
                ]),
            ];

            let inspector = Paragraph::new(text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Volume Details ({}) ", name)),
            );
            frame.render_widget(inspector, main_chunks[1]);
        }
        Some(VolumeRow::GroupHeader(GroupHeader {
            name,
            total_count,
            ..
        })) => {
            let text = vec![
                Line::from(vec![
                    Span::raw("Group: "),
                    Span::styled(*name, Style::default().fg(Color::Cyan).bold()),
                ]),
                Line::from(vec![
                    Span::raw("Total Volumes: "),
                    Span::raw(total_count.to_string()),
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
            let inspector_block = Paragraph::new("Select a volume to inspect").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Volume Info "),
            );
            frame.render_widget(inspector_block, main_chunks[1]);
        }
    }
}
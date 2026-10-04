use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::enums::network::NetworkRow;
use crate::models::app::App;
use crate::models::generics::GroupHeader;
use bollard::models::Network;

pub fn render_networks_tab(app: &mut App, frame: &mut Frame, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    let visible_rows = app.network_tab.get_visible_rows();
    let rows = build_rows(&visible_rows);
    let selected_index = app.network_tab.table_state.selected();
    drop(visible_rows);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["NAME", "DRIVER", "SCOPE", "SUBNET"])
            .style(Style::default().fg(Color::Yellow)),
    )
    .block(Block::default().borders(Borders::ALL).title(" Networks "))
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_stateful_widget(table, main_chunks[0], &mut app.network_tab.table_state);

    let inspector_rows = app.network_tab.get_visible_rows();
    let selected_info = selected_index.and_then(|idx| inspector_rows.get(idx));

    match selected_info {
        Some(NetworkRow::ChildNetwork { network, .. })
        | Some(NetworkRow::StandaloneNetwork { network }) => {
            render_network_inspector(frame, main_chunks[1], network);
        }
        Some(NetworkRow::GroupHeader(GroupHeader {
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
                    Span::raw("Total Networks: "),
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
            let placeholder = Paragraph::new("Select a network to inspect").block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Network Details "),
            );
            frame.render_widget(placeholder, main_chunks[1]);
        }
    }
}

fn build_rows(visible_rows: &[NetworkRow<'_>]) -> Vec<Row<'static>> {
    visible_rows
        .iter()
        .map(|row_item| match row_item {
            NetworkRow::GroupHeader(GroupHeader {
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
                let info_cell = Cell::from(format!("({} networks)", total_count))
                    .style(Style::default().fg(Color::DarkGray));
                let status_cell = Cell::from("⚡ compose group").cyan();

                Row::new(vec![group_name_cell, info_cell, status_cell])
                    .style(Style::default().bg(Color::Reset))
            }
            NetworkRow::ChildNetwork {
                network,
                is_last_in_group,
            } => {
                let name = network.name.as_deref().unwrap_or("unnamed");
                let branch = if *is_last_in_group {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                let name_cell = Cell::from(format!("{}{}", branch, name));
                let driver_cell = Cell::from(network.driver.as_deref().unwrap_or("-").to_string());
                let scope_cell = Cell::from(network.scope.as_deref().unwrap_or("local").to_string());
                let subnet_cell = Cell::from(get_subnet(network));

                Row::new(vec![name_cell, driver_cell, scope_cell, subnet_cell])
            }
            NetworkRow::StandaloneNetwork { network } => {
                let name = network.name.as_deref().unwrap_or("unnamed");

                let name_cell = Cell::from(format!("⚡ {}", name)).bold();
                let driver_cell = Cell::from(network.driver.as_deref().unwrap_or("-").to_string());
                let scope_cell = Cell::from(network.scope.as_deref().unwrap_or("local").to_string());
                let subnet_cell = Cell::from(get_subnet(network));

                Row::new(vec![name_cell, driver_cell, scope_cell, subnet_cell])
            }
        })
        .collect()
}

fn get_subnet(network: &Network) -> String {
    network
        .ipam
        .as_ref()
        .and_then(|ipam| ipam.config.as_ref())
        .and_then(|configs| configs.first())
        .and_then(|cfg| cfg.subnet.as_deref())
        .unwrap_or("-")
        .to_string()
}

fn get_gateway(network: &Network) -> String {
    network
        .ipam
        .as_ref()
        .and_then(|ipam| ipam.config.as_ref())
        .and_then(|configs| configs.first())
        .and_then(|cfg| cfg.gateway.as_deref())
        .unwrap_or("-")
        .to_string()
}

fn format_id(id: &Option<String>) -> String {
    id.as_ref()
        .map(|s| {
            if s.len() >= 12 {
                s[..12].to_string()
            } else {
                s.clone()
            }
        })
        .unwrap_or_else(|| "N/A".to_string())
}

fn render_network_inspector(frame: &mut Frame, area: Rect, network: &Network) {
    let name = network.name.as_deref().unwrap_or("unnamed");
    let id = format_id(&network.id);
    let driver = network.driver.as_deref().unwrap_or("N/A");
    let scope = network.scope.as_deref().unwrap_or("local");
    let subnet = get_subnet(network);
    let gateway = get_gateway(network);
    let internal = if network.internal.unwrap_or(false) { "Yes" } else { "No" };
    let ipv6 = if network.enable_ipv6.unwrap_or(false) { "Yes" } else { "No" };
    let attachable = if network.attachable.unwrap_or(false) { "Yes" } else { "No" };
    let ingress = if network.ingress.unwrap_or(false) { "Yes" } else { "No" };
    let created_at = network
        .created
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "N/A".to_string());
    let labels_count = network.labels.as_ref().map(|l| l.len()).unwrap_or(0);
    let options_count = network.options.as_ref().map(|o| o.len()).unwrap_or(0);

    let text = vec![
        Line::from(vec![
            Span::raw("Name: "),
            Span::styled(name, Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::raw("ID: "),
            Span::styled(id, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("Driver: "),
            Span::styled(driver, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("Scope: "),
            Span::raw(scope),
        ]),
        Line::from(vec![
            Span::raw("Subnet: "),
            Span::raw(subnet),
        ]),
        Line::from(vec![
            Span::raw("Gateway: "),
            Span::raw(gateway),
        ]),
        Line::from(vec![
            Span::raw("Internal: "),
            Span::raw(internal),
        ]),
        Line::from(vec![
            Span::raw("IPv6 Enabled: "),
            Span::raw(ipv6),
        ]),
        Line::from(vec![
            Span::raw("Attachable: "),
            Span::raw(attachable),
        ]),
        Line::from(vec![
            Span::raw("Ingress: "),
            Span::raw(ingress),
        ]),
        Line::from(vec![
            Span::raw("Created At: "),
            Span::raw(created_at),
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
            .title(format!(" Network Details ({}) ", name)),
    );

    frame.render_widget(inspector, area);
}

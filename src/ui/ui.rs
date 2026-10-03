use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::models::app::{ActiveTab, App, ViewMode};
use crate::ui::container::render_containers_tab;
use crate::ui::image::render_images_tab;
use crate::ui::modal::render_confirm_delete_modal;
use crate::ui::network::render_networks_tab;
use crate::ui::volume::render_volumes_tab;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    render_header(app, frame, chunks[0]);
    render_main_content(app, frame, chunks[1]);
    render_footer(app, frame, chunks[2]);

    if app.view_mode == ViewMode::ConfirmDeleteModal {
        render_confirm_delete_modal(app, frame, area);
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let titles = vec![
        "[1] Containers",
        "[2] Images",
        "[3] Volumes",
        "[4] Networks",
        "[5] System",
    ];
    let selected_index = match app.active_tab {
        ActiveTab::Containers => 0,
        ActiveTab::Images => 1,
        ActiveTab::Volumes => 2,
        ActiveTab::Networks => 3,
        ActiveTab::System => 4,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" 🐳 Easy Docker "))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .select(selected_index);

    frame.render_widget(tabs, area);
}

fn render_main_content(app: &mut App, frame: &mut Frame, area: Rect) {
    match app.active_tab {
        ActiveTab::Containers => render_containers_tab(app, frame, area),
        ActiveTab::Images => render_images_tab(app, frame, area),
        ActiveTab::Volumes => render_volumes_tab(app, frame, area),
        ActiveTab::Networks => render_networks_tab(app, frame, area),
        _ => {
            let placeholder = Paragraph::new("Tab coming soon...")
                .block(Block::default().borders(Borders::ALL).title(" Tab "));
            frame.render_widget(placeholder, area);
        }
    }
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let (title_text, title_style) = if let Some(toast) = &app.toast_message {
        (
            format!(" ⌨ Controls & Help │ 🔔 {} ", toast),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            " ⌨ Controls & Help ".to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    };

    let text = vec![
        Line::from(vec![
            Span::styled(" [↑/↓ / j/k] ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Navigate  │ "),
            Span::styled("[Click] ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Select  │ "),
            Span::styled("[Space/Enter] ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Toggle Group  │ "),
            Span::styled("[1-5 / Tab] ", Style::default().fg(Color::Yellow).bold()),
            Span::raw("Switch Tab"),
        ]),
        Line::from(vec![
            Span::styled(" [s] ", Style::default().fg(Color::Green).bold()),
            Span::raw("Start  │ "),
            Span::styled("[x] ", Style::default().fg(Color::Red).bold()),
            Span::raw("Stop  │ "),
            Span::styled("[r] ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("Restart  │ "),
            Span::styled("[d] ", Style::default().fg(Color::Magenta).bold()),
            Span::raw(match app.active_tab {
                ActiveTab::Containers => "Delete Container  │ ",
                ActiveTab::Images => "Delete Image  │ ",
                ActiveTab::Volumes => "Delete Volume  │ ",
                ActiveTab::Networks => "Delete Network  │ ",
                _ => "Delete  │ ",
            }),
            Span::styled("[q] ", Style::default().fg(Color::Gray).bold()),
            Span::raw("Quit"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title_text, title_style));

    let footer = Paragraph::new(text).block(block);
    frame.render_widget(footer, area);
}

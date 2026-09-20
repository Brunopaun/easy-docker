mod components;
mod event;
mod models;
mod services;
mod ui;

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
};
use crossterm::execute;
use futures_util::StreamExt;
use std::io::stdout;
use tokio::sync::mpsc;

use models::app::App;

pub enum AppEvent {
    Input(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Tick,
    DockerInitialized(bollard::Docker),
    ContainersUpdated(Vec<bollard::models::ContainerSummary>),
    VolumesUpdated(Vec<bollard::models::Volume>),
    ContainerDeleted {
        id: String,
        result: Result<(), String>,
    },
    ContainerActionDone {
        id: String,
        action: String,
        result: Result<(), String>,
    },
    ClearToast,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();
    let _ = execute!(stdout(), EnableMouseCapture);

    let mut app = App::new();

    let (tx, mut rx) = mpsc::channel::<AppEvent>(100);

    let tx_docker = tx.clone();
    tokio::spawn(async move {
        if let Ok(docker) = services::docker::get_docker_client().await {
            let _ = tx_docker.send(AppEvent::DockerInitialized(docker)).await;
        }
    });

    let tx_events = tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let _ = tx_events.send(AppEvent::Tick).await;
                }
                maybe_event = reader.next() => {
                    match maybe_event {
                        Some(Ok(Event::Key(key))) => {
                            if key.kind == KeyEventKind::Press {
                                let _ = tx_events.send(AppEvent::Input(key)).await;
                            }
                        }
                        Some(Ok(Event::Mouse(mouse))) => {
                            let _ = tx_events.send(AppEvent::Mouse(mouse)).await;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let tx_docker = tx.clone();
    tokio::spawn(async move {
        if let Ok(docker) = services::docker::get_docker_client().await {
            loop {
                if let Ok(containers) = services::docker::list_containers(&docker).await {
                    if tx_docker
                        .send(AppEvent::ContainersUpdated(containers))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    });

    let tx_docker = tx.clone();
    tokio::spawn(async move {
        if let Ok(docker) = services::docker::get_docker_client().await {
            loop {
                if let Ok(volumes) = services::docker::list_volumes(&docker).await {
                    if tx_docker.send(AppEvent::VolumesUpdated(volumes)).await.is_err() {
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    });

    while !app.should_quit {
        let tx_docker = tx.clone();
        terminal.draw(|frame| ui::render(&mut app, frame))?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Input(key) => match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Down | KeyCode::Char('j') => app.containers_tab.next_container(),
                    KeyCode::Up | KeyCode::Char('k') => app.containers_tab.previous_container(),
                    KeyCode::Char(' ') | KeyCode::Enter | KeyCode::Char('o') => {
                        app.containers_tab.toggle_group();
                    }
                    KeyCode::Char('1') => app.active_tab = models::app::ActiveTab::Containers,
                    KeyCode::Char('2') => app.active_tab = models::app::ActiveTab::Images,
                    KeyCode::Char('3') => app.active_tab = models::app::ActiveTab::Volumes,
                    KeyCode::Char('4') => app.active_tab = models::app::ActiveTab::Networks,
                    KeyCode::Char('5') => app.active_tab = models::app::ActiveTab::System,
                    KeyCode::Char('s') => {
                        app.containers_tab
                            .start_container(&app.client, tx_docker, &mut app.toast_message);
                    }
                    KeyCode::Char('x') => {
                        app.containers_tab
                            .stop_container(&app.client, tx_docker, &mut app.toast_message);
                    }
                    KeyCode::Char('r') => {
                        app.containers_tab
                            .restart_container(&app.client, tx_docker, &mut app.toast_message);
                    }
                    KeyCode::Char('d') => {
                        app.containers_tab
                            .delete_container(&app.client, tx_docker, &mut app.toast_message);
                    }
                    KeyCode::Tab => {
                        app.active_tab = match app.active_tab {
                            models::app::ActiveTab::Containers => models::app::ActiveTab::Images,
                            models::app::ActiveTab::Images => models::app::ActiveTab::Volumes,
                            models::app::ActiveTab::Volumes => models::app::ActiveTab::Networks,
                            models::app::ActiveTab::Networks => models::app::ActiveTab::System,
                            models::app::ActiveTab::System => models::app::ActiveTab::Containers,
                        };
                    }
                    _ => {}
                },
                AppEvent::Mouse(mouse) => {
                    app.containers_tab.handle_mouse_click(mouse);
                }
                AppEvent::Tick => {
                    // Periodic UI animation / tick updates if needed
                }
                AppEvent::ContainersUpdated(containers) => {
                    app.containers_tab.containers = containers;
                    app.containers_tab.is_loading = false;
                    if app.containers_tab.table_state.selected().is_none()
                        && !app.containers_tab.get_visible_rows().is_empty()
                    {
                        app.containers_tab.table_state.select(Some(0));
                    }
                }
                AppEvent::DockerInitialized(docker) => {
                    app.client = Some(docker);
                }
                AppEvent::ContainerDeleted { id, result } => {
                    match result {
                        Ok(_) => {
                            let short_id = id.chars().take(12).collect::<String>();
                            app.toast_message =
                                Some(format!("✅ Container {} deleted!", short_id));
                            app.containers_tab
                                .containers
                                .retain(|c| c.id.as_deref() != Some(id.as_str()));
                        }
                        Err(err) => {
                            let short_id = id.chars().take(12).collect::<String>();
                            app.toast_message =
                                Some(format!("❌ Error deleting {}: {}", short_id, err));
                        }
                    }
                }
                AppEvent::ContainerActionDone { id, action, result } => {
                    let short_id = id.chars().take(12).collect::<String>();
                    match result {
                        Ok(_) => {
                            app.toast_message =
                                Some(format!("✅ Container {} {}!", short_id, action));
                        }
                        Err(err) => {
                            app.toast_message =
                                Some(format!("❌ Error {}: {}", action, err));
                        }
                    }
                }
                AppEvent::VolumesUpdated(volumes) => {
                    app.volumes_tab.volumes = volumes;
                },
                AppEvent::ClearToast => {
                    app.toast_message = None;
                }
            }
        }
    }

    let _ = execute!(stdout(), DisableMouseCapture);
    ratatui::restore();
    Ok(())
}

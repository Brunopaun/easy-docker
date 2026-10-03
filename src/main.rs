mod components;
mod event;
mod models;
mod services;
mod ui;
mod enums;

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
    ContainerDeleted {
        id: String,
        result: Result<(), String>,
    },
    ContainerActionDone {
        id: String,
        action: String,
        result: Result<(), String>,
    },
    ContainerLogsLoaded {
        id: String,
        name: String,
        logs: Result<Vec<String>, String>,
    },
    VolumesUpdated(Vec<bollard::models::Volume>),
    VolumeActionDone {
        name:String,
        action: String,
        result: Result<(), String>,
    },
    ImagesUpdated(Vec<bollard::models::ImageSummary>),
    ImageActionDone {
        id: String,
        action: String,
        result: Result<(), String>,
    },
    NetworksUpdated(Vec<bollard::models::Network>),
    NetworkActionDone {
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
                if let Ok(images) = services::docker::list_images(&docker).await {
                    if tx_docker
                        .send(AppEvent::ImagesUpdated(images))
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

    let tx_docker = tx.clone();
    tokio::spawn(async move {
        if let Ok(docker) = services::docker::get_docker_client().await {
            loop {
                if let Ok(networks) = services::docker::list_networks(&docker).await {
                    if tx_docker.send(AppEvent::NetworksUpdated(networks)).await.is_err() {
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
                AppEvent::Input(key) => match app.view_mode {
                    models::app::ViewMode::ConfirmDeleteModal => match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            if let Some(target) = app.delete_target.take() {
                                match target {
                                    models::app::DeleteTarget::Container(_) => {
                                        app.containers_tab.delete_container(
                                            &app.client,
                                            tx_docker,
                                            &mut app.toast_message,
                                        );
                                    }
                                    models::app::DeleteTarget::Volume(_) => {
                                        app.volumes_tab.delete_volume(
                                            &app.client,
                                            tx_docker,
                                            &mut app.toast_message,
                                        );
                                    }
                                    models::app::DeleteTarget::Image(_) => {
                                        app.images_tab.delete_image(
                                            &app.client,
                                            tx_docker,
                                            &mut app.toast_message,
                                        );
                                    }
                                    models::app::DeleteTarget::Network(_) => {
                                        app.network_tab.delete_network(
                                            &app.client,
                                            tx_docker,
                                            &mut app.toast_message,
                                        );
                                    }
                                }
                            }
                            app.view_mode = models::app::ViewMode::Normal;
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            app.delete_target = None;
                            app.view_mode = models::app::ViewMode::Normal;
                        }
                        _ => {}
                    },
                    models::app::ViewMode::Normal => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Down | KeyCode::Char('j') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab.next_container();
                                if app.containers_tab.inspector_view == components::container::ContainerInspectorView::Logs {
                                    app.containers_tab.fetch_logs(&app.client, tx_docker);
                                }
                            }
                            models::app::ActiveTab::Images => app.images_tab.next_image(),
                            models::app::ActiveTab::Volumes => app.volumes_tab.next_volume(),
                            models::app::ActiveTab::Networks => app.network_tab.next_network(),
                            _ => {}
                        },
                        KeyCode::Up | KeyCode::Char('k') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab.previous_container();
                                if app.containers_tab.inspector_view == components::container::ContainerInspectorView::Logs {
                                    app.containers_tab.fetch_logs(&app.client, tx_docker);
                                }
                            }
                            models::app::ActiveTab::Images => app.images_tab.previous_image(),
                            models::app::ActiveTab::Volumes => app.volumes_tab.previous_volume(),
                            models::app::ActiveTab::Networks => app.network_tab.previous_network(),
                            _ => {}
                        },
                        KeyCode::Char(' ') | KeyCode::Enter | KeyCode::Char('o') => match app.active_tab {
                            models::app::ActiveTab::Containers => app.containers_tab.toggle_group(),
                            models::app::ActiveTab::Images => app.images_tab.toggle_group(),
                            models::app::ActiveTab::Volumes => app.volumes_tab.toggle_group(),
                            models::app::ActiveTab::Networks => app.network_tab.toggle_group(),
                            _ => {}
                        },
                        KeyCode::Char('1') => app.active_tab = models::app::ActiveTab::Containers,
                        KeyCode::Char('2') => app.active_tab = models::app::ActiveTab::Images,
                        KeyCode::Char('3') => app.active_tab = models::app::ActiveTab::Volumes,
                        KeyCode::Char('4') => app.active_tab = models::app::ActiveTab::Networks,
                        KeyCode::Char('5') => app.active_tab = models::app::ActiveTab::System,
                        KeyCode::Char('s') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab
                                    .start_container(&app.client, tx_docker, &mut app.toast_message);
                            }
                            _ => {}
                        },
                        KeyCode::Char('x') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab
                                    .stop_container(&app.client, tx_docker, &mut app.toast_message);
                            }
                            _ => {}
                        },
                        KeyCode::Char('r') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab
                                    .restart_container(&app.client, tx_docker, &mut app.toast_message);
                            }
                            _ => {}
                        },
                        KeyCode::Char('d') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab.inspector_view = components::container::ContainerInspectorView::Details;
                            }
                            _ => {}
                        },
                        KeyCode::Char('l') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                app.containers_tab.inspector_view = components::container::ContainerInspectorView::Logs;
                                app.containers_tab.fetch_logs(&app.client, tx_docker);
                            }
                            _ => {}
                        },
                        KeyCode::Char('f') => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                if app.containers_tab.inspector_view == components::container::ContainerInspectorView::Logs {
                                    app.containers_tab.toggle_auto_scroll();
                                }
                            }
                            _ => {}
                        },
                        KeyCode::PageDown => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                if app.containers_tab.inspector_view == components::container::ContainerInspectorView::Logs {
                                    app.containers_tab.scroll_logs_down();
                                }
                            }
                            _ => {}
                        },
                        KeyCode::PageUp => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                if app.containers_tab.inspector_view == components::container::ContainerInspectorView::Logs {
                                    app.containers_tab.scroll_logs_up();
                                }
                            }
                            _ => {}
                        },
                        KeyCode::Delete => match app.active_tab {
                            models::app::ActiveTab::Containers => {
                                if let Some(id) = app.containers_tab.get_selected_container_id() {
                                    app.delete_target = Some(models::app::DeleteTarget::Container(id));
                                    app.view_mode = models::app::ViewMode::ConfirmDeleteModal;
                                }
                            }
                            models::app::ActiveTab::Volumes => {
                                if let Some(name) = app.volumes_tab.get_selected_volume() {
                                    app.delete_target = Some(models::app::DeleteTarget::Volume(name));
                                    app.view_mode = models::app::ViewMode::ConfirmDeleteModal;
                                }
                            }
                            models::app::ActiveTab::Images => {
                                if let Some(id) = app.images_tab.get_selected_image_id() {
                                    app.delete_target = Some(models::app::DeleteTarget::Image(id));
                                    app.view_mode = models::app::ViewMode::ConfirmDeleteModal;
                                }
                            }
                            models::app::ActiveTab::Networks => {
                                if let Some(id) = app.network_tab.get_selected_network_id() {
                                    app.delete_target = Some(models::app::DeleteTarget::Network(id));
                                    app.view_mode = models::app::ViewMode::ConfirmDeleteModal;
                                }
                            }
                            _ => {}
                        },
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
                    _ => {}
                },
                AppEvent::Mouse(mouse) => match app.active_tab {
                    models::app::ActiveTab::Containers => app.containers_tab.handle_mouse_click(mouse),
                    models::app::ActiveTab::Images => app.images_tab.handle_mouse_click(mouse),
                    models::app::ActiveTab::Volumes => app.volumes_tab.handle_mouse_click(mouse),
                    models::app::ActiveTab::Networks => app.network_tab.handle_mouse_click(mouse),
                    _ => {}
                },
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
                },
                AppEvent::ContainerLogsLoaded { id, name: _, logs } => {
                    if app.containers_tab.active_log_container_id.as_deref() == Some(id.as_str()) {
                        app.containers_tab.is_loading_logs = false;
                        match logs {
                            Ok(lines) => {
                                app.containers_tab.logs = lines;
                            }
                            Err(err) => {
                                app.containers_tab.logs = vec![format!("❌ Error fetching logs: {}", err)];
                            }
                        }
                    }
                },
                AppEvent::VolumeActionDone { name, action, result } => {
                    match result {
                        Ok(_) => {
                            app.toast_message =
                                Some(format!("✅ Volume {} {}!", name, action));
                        }
                        Err(err) => {
                            app.toast_message =
                                Some(format!("❌ Error {}: {}", action, err));
                        }
                    }
                },
                AppEvent::ImageActionDone { id, action, result } => {
                    let clean_id = id.strip_prefix("sha256:").unwrap_or(&id);
                    let short_id = clean_id.chars().take(12).collect::<String>();
                    match result {
                        Ok(_) => {
                            app.toast_message =
                                Some(format!("✅ Image {} {}!", short_id, action));
                        }
                        Err(err) => {
                            app.toast_message =
                                Some(format!("❌ Error {}: {}", action, err));
                        }
                    }
                },
                AppEvent::NetworkActionDone { id, action, result } => {
                    let short_id = id.chars().take(12).collect::<String>();
                    match result {
                        Ok(_) => {
                            app.toast_message =
                                Some(format!("✅ Network {} {}!", short_id, action));
                        }
                        Err(err) => {
                            app.toast_message =
                                Some(format!("❌ Error {}: {}", action, err));
                        }
                    }
                },
                AppEvent::VolumesUpdated(volumes) => {
                    app.volumes_tab.volumes = volumes;
                    app.volumes_tab.is_loading = false;
                    if app.volumes_tab.table_state.selected().is_none()
                        && !app.volumes_tab.get_visible_rows().is_empty()
                    {
                        app.volumes_tab.table_state.select(Some(0));
                    }
                },
                AppEvent::ImagesUpdated(images) => {
                    app.images_tab.images = images;
                    app.images_tab.is_loading = false;
                    if app.images_tab.table_state.selected().is_none()
                        && !app.images_tab.get_visible_rows().is_empty()
                    {
                        app.images_tab.table_state.select(Some(0));
                    }
                },
                AppEvent::NetworksUpdated(networks) => {
                    app.network_tab.networks = networks;
                    app.network_tab.is_loading = false;
                    if app.network_tab.table_state.selected().is_none()
                        && !app.network_tab.get_visible_rows().is_empty()
                    {
                        app.network_tab.table_state.select(Some(0));
                    }
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

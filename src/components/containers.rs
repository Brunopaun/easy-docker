use std::collections::{BTreeMap, HashSet};
use bollard::{models::ContainerSummary, Docker};
use ratatui::widgets::TableState;
use tokio::sync::mpsc::Sender;

use crate::services::docker::{
    remove_container, restart_container, start_container, stop_container,
};
use crate::AppEvent;
use crate::models::generics::GroupHeader;


pub enum ContainerRow<'a> {
    GroupHeader(GroupHeader<'a>),
    ChildContainer {
        container: &'a ContainerSummary,
        is_last_in_group: bool,
    },
    StandaloneContainer {
        container: &'a ContainerSummary,
    },
}

pub struct ContainersTab {
    pub containers: Vec<ContainerSummary>,
    pub table_state: TableState,
    pub collapsed_groups: HashSet<String>,
    pub is_loading: bool,
}

impl ContainersTab {
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
            table_state: TableState::default(),
            collapsed_groups: HashSet::new(),
            is_loading: true,
        }
    }

    pub fn get_visible_rows(&self) -> Vec<ContainerRow<'_>> {
        let mut grouped: BTreeMap<&str, Vec<&ContainerSummary>> = BTreeMap::new();
        let mut standalone: Vec<&ContainerSummary> = Vec::new();

        for c in &self.containers {
            if let Some(project) = c
                .labels
                .as_ref()
                .and_then(|l| l.get("com.docker.compose.project"))
            {
                grouped.entry(project.as_str()).or_default().push(c);
            } else {
                standalone.push(c);
            }
        }

        let mut rows = Vec::new();

        for (project_name, group_containers) in &grouped {
            let total_count = group_containers.len();
            let running_count = group_containers
                .iter()
                .filter(|c| c.state.as_ref().map(|s| s.as_ref()) == Some("running"))
                .count();
            let is_expanded = !self.collapsed_groups.contains(*project_name);

            rows.push(ContainerRow::GroupHeader(GroupHeader {
                name: project_name,
                total_count,
                running_count,
                is_expanded,
            }));

            if is_expanded {
                let len = group_containers.len();
                for (idx, c) in group_containers.iter().enumerate() {
                    rows.push(ContainerRow::ChildContainer {
                        container: c,
                        is_last_in_group: idx == len - 1,
                    });
                }
            }
        }

        for c in standalone {
            rows.push(ContainerRow::StandaloneContainer { container: c });
        }

        rows
    }

    pub fn toggle_group(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let rows = self.get_visible_rows();
            if let Some(ContainerRow::GroupHeader(GroupHeader {
                name, is_expanded, ..
            })) = rows.get(selected)
            {
                let group_name = name.to_string();
                let is_exp = *is_expanded;
                drop(rows);

                if is_exp {
                    self.collapsed_groups.insert(group_name);
                } else {
                    self.collapsed_groups.remove(&group_name);
                }

                let new_count = self.get_visible_rows().len();
                if new_count == 0 {
                    self.table_state.select(None);
                } else if selected >= new_count {
                    self.table_state.select(Some(new_count - 1));
                }
            }
        }
    }

    pub fn next_container(&mut self) {
        let count = self.get_visible_rows().len();
        if count == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + 1) % count,
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous_container(&mut self) {
        let count = self.get_visible_rows().len();
        if count == 0 {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn handle_mouse_click(&mut self, mouse: crossterm::event::MouseEvent) {
        if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) {
            if mouse.row >= 5 {
                let clicked_row = (mouse.row - 5) as usize;
                let rows_count = self.get_visible_rows().len();
                if clicked_row < rows_count {
                    self.table_state.select(Some(clicked_row));
                    self.toggle_group();
                }
            }
        }
    }

    pub fn get_selected_container_id(&self) -> Option<String> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        match rows.get(selected_index)? {
            ContainerRow::ChildContainer { container, .. }
            | ContainerRow::StandaloneContainer { container } => container.id.clone(),
            _ => None,
        }
    }

    pub fn start_container(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_container_id()) {
            let short_id = id.chars().take(12).collect::<String>();
            *toast = Some(format!("⏳ Starting {}...", short_id));

            tokio::spawn(async move {
                let res = start_container(&client, &id).await.map_err(|e| e.to_string());
                let _ = tx
                    .send(AppEvent::ContainerActionDone {
                        id,
                        action: "started".into(),
                        result: res,
                    })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }

    pub fn stop_container(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_container_id()) {
            let short_id = id.chars().take(12).collect::<String>();
            *toast = Some(format!("⏳ Stopping {}...", short_id));

            tokio::spawn(async move {
                let res = stop_container(&client, &id).await.map_err(|e| e.to_string());
                let _ = tx
                    .send(AppEvent::ContainerActionDone {
                        id,
                        action: "stopped".into(),
                        result: res,
                    })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }

    pub fn restart_container(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_container_id()) {
            let short_id = id.chars().take(12).collect::<String>();
            *toast = Some(format!("⏳ Restarting {}...", short_id));

            tokio::spawn(async move {
                let res = restart_container(&client, &id).await.map_err(|e| e.to_string());
                let _ = tx
                    .send(AppEvent::ContainerActionDone {
                        id,
                        action: "restarted".into(),
                        result: res,
                    })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }

    pub fn delete_container(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_container_id()) {
            let short_id = id.chars().take(12).collect::<String>();
            *toast = Some(format!("⏳ Deleting {}...", short_id));

            tokio::spawn(async move {
                let res = remove_container(&client, &id).await.map_err(|e| e.to_string());

                let _ = tx
                    .send(AppEvent::ContainerDeleted { id, result: res })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }
}

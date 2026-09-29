use std::collections::{BTreeMap, HashSet};
use bollard::models::Volume;
use bollard::Docker;
use ratatui::widgets::TableState;
use tokio::sync::mpsc::Sender;

use crate::models::generics::GroupHeader;
use crate::services::docker::remove_volume;
use crate::AppEvent;

pub enum VolumeRow<'a> {
    GroupHeader(GroupHeader<'a>),
    ChildVolume {
        volume: &'a Volume,
        is_last_in_group: bool,
    },
    StandaloneVolume {
        volume: &'a Volume,
    },
}

pub struct VolumesTab {
    pub volumes: Vec<Volume>,
    pub table_state: TableState,
    pub collapsed_groups: HashSet<String>,
    pub is_loading: bool,
}

impl VolumesTab {
    pub fn new() -> Self {
        Self {
            volumes: Vec::new(),
            table_state: TableState::default(),
            collapsed_groups: HashSet::new(),
            is_loading: false,
        }
    }

    pub fn get_visible_rows(&self) -> Vec<VolumeRow<'_>> {
        let mut grouped: BTreeMap<&str, Vec<&Volume>> = BTreeMap::new();
        let mut standalone: Vec<&Volume> = Vec::new();

        for c in &self.volumes {
            if let Some(project) = c.labels.get("com.docker.compose.project") {
                grouped.entry(project.as_str()).or_default().push(c);
            } else {
                standalone.push(c);
            }
        }

        let mut rows = Vec::new();

        for (project_name, group_volumes) in &grouped {
            let total_count = group_volumes.len();
            let running_count = total_count;
            let is_expanded = !self.collapsed_groups.contains(*project_name);

            rows.push(VolumeRow::GroupHeader(GroupHeader {
                name: project_name,
                total_count,
                running_count,
                is_expanded,
            }));

            if is_expanded {
                let len = group_volumes.len();
                for (idx, c) in group_volumes.iter().enumerate() {
                    rows.push(VolumeRow::ChildVolume {
                        volume: c,
                        is_last_in_group: idx == len - 1,
                    });
                }
            }
        }

        for c in standalone {
            rows.push(VolumeRow::StandaloneVolume { volume: c });
        }

        rows
    }

    pub fn toggle_group(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let rows = self.get_visible_rows();
            if let Some(VolumeRow::GroupHeader(GroupHeader {
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

    pub fn next_volume(&mut self) {
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

    pub fn previous_volume(&mut self) {
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

    pub fn get_selected_volume(&self) -> Option<String> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        match rows.get(selected_index)? {
            VolumeRow::ChildVolume { volume, .. }
            | VolumeRow::StandaloneVolume { volume } => Some(volume.name.clone()),
            _ => None,
        }
    }

    pub fn delete_volume(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(name)) = (client.clone(), self.get_selected_volume()) {
            *toast = Some(format!("⏳ Deleting volume {}...", name));

            tokio::spawn(async move {
                let res = remove_volume(&client, &name).await.map_err(|e| e.to_string());
                let _ = tx
                    .send(AppEvent::VolumeActionDone {
                        name,
                        action: "deleted".into(),
                        result: res,
                    })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }
}



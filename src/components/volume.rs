use std::ops::{Deref, DerefMut};
use bollard::models::Volume;
use bollard::Docker;
use tokio::sync::mpsc::Sender;

use crate::components::tab::Tab;
use crate::models::generics::GroupHeader;
use crate::services::docker::remove_volume;
use crate::AppEvent;
use crate::enums::volume::VolumeRow;

pub struct VolumesTab {
    pub tab: Tab,
    pub volumes: Vec<Volume>,
}

impl Deref for VolumesTab {
    type Target = Tab;
    fn deref(&self) -> &Self::Target {
        &self.tab
    }
}

impl DerefMut for VolumesTab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tab
    }
}

impl VolumesTab {
    pub fn new() -> Self {
        Self {
            tab: Tab::new(),
            volumes: Vec::new(),
        }
    }

    pub fn get_visible_rows(&self) -> Vec<VolumeRow<'_>> {
        self.tab.build_grouped_rows(
            &self.volumes,
            |c| c.labels.get("com.docker.compose.project").map(|s| s.as_str()),
            |_| true,
            VolumeRow::GroupHeader,
            |c, is_last| VolumeRow::ChildVolume {
                volume: c,
                is_last_in_group: is_last,
            },
            |c| VolumeRow::StandaloneVolume { volume: c },
        )
    }

    pub fn toggle_group(&mut self) {
        let target = {
            let rows = self.get_visible_rows();
            self.tab.get_selected_group_header(&rows)
        };

        if let Some((group_name, is_expanded)) = target {
            self.tab.toggle_group_name(&group_name, is_expanded);
            let new_count = self.get_visible_rows().len();
            self.tab.adjust_selection(new_count);
        }
    }

    pub fn next_volume(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_next(count);
    }

    pub fn previous_volume(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_previous(count);
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

    pub fn get_selected_group_volumes(&self) -> Option<(String, Vec<String>)> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        if let Some(VolumeRow::GroupHeader(GroupHeader { name, .. })) = rows.get(selected_index) {
            let group_name = name.to_string();
            let mut volume_names = Vec::new();
            for v in &self.volumes {
                if let Some(proj) = v.labels.get("com.docker.compose.project") {
                    if proj == &group_name {
                        volume_names.push(v.name.clone());
                    }
                }
            }
            if !volume_names.is_empty() {
                return Some((group_name, volume_names));
            }
        }
        None
    }

    pub fn delete_volume_group(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
        group_name: &str,
        names: Vec<String>,
    ) {
        if let Some(client) = client.clone() {
            let count = names.len();
            *toast = Some(format!("⏳ Deleting {} volumes in group {}...", count, group_name));

            tokio::spawn(async move {
                for name in names {
                    let res = remove_volume(&client, &name).await.map_err(|e| e.to_string());
                    let _ = tx
                        .send(AppEvent::VolumeActionDone {
                            name,
                            action: "deleted".into(),
                            result: res,
                        })
                        .await;
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }
}



use bollard::models::Network;
use bollard::Docker;
use ratatui::widgets::TableState;
use std::collections::{BTreeMap, HashSet};
use tokio::sync::mpsc::Sender;

use crate::enums::network::NetworkRow;
use crate::models::generics::GroupHeader;
use crate::services::docker::remove_network;
use crate::AppEvent;

pub struct NetworkTab {
    pub networks: Vec<Network>,
    pub table_state: TableState,
    pub collapsed_groups: HashSet<String>,
    pub is_loading: bool,
}

impl NetworkTab {
    pub fn new() -> Self {
        Self {
            networks: Vec::new(),
            table_state: TableState::default(),
            collapsed_groups: HashSet::new(),
            is_loading: true,
        }
    }

    pub fn get_visible_rows(&self) -> Vec<NetworkRow<'_>> {
        let mut grouped: BTreeMap<&str, Vec<&Network>> = BTreeMap::new();
        let mut standalone: Vec<&Network> = Vec::new();

        for c in &self.networks {
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

        for (project_name, group_networks) in &grouped {
            let total_count = group_networks.len();
            let running_count = total_count;
            let is_expanded = !self.collapsed_groups.contains(*project_name);

            rows.push(NetworkRow::GroupHeader(GroupHeader {
                name: project_name,
                total_count,
                running_count,
                is_expanded,
            }));

            if is_expanded {
                let len = group_networks.len();
                for (idx, c) in group_networks.iter().enumerate() {
                    rows.push(NetworkRow::ChildNetwork {
                        network: c,
                        is_last_in_group: idx == len - 1,
                    });
                }
            }
        }

        for c in standalone {
            rows.push(NetworkRow::StandaloneNetwork { network: c });
        }

        rows
    }

    pub fn toggle_group(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            let rows = self.get_visible_rows();
            if let Some(NetworkRow::GroupHeader(GroupHeader {
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

    pub fn next_network(&mut self) {
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

    pub fn previous_network(&mut self) {
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

    pub fn get_selected_network_id(&self) -> Option<String> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        match rows.get(selected_index)? {
            NetworkRow::ChildNetwork { network, .. }
            | NetworkRow::StandaloneNetwork { network } => {
                network.id.clone().or_else(|| network.name.clone())
            }
            _ => None,
        }
    }

    pub fn delete_network(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_network_id()) {
            let short_id = id.chars().take(12).collect::<String>();
            *toast = Some(format!("⏳ Deleting network {}...", short_id));

            tokio::spawn(async move {
                let res = remove_network(&client, &id).await;
                let result_str = res.map_err(|e| e.to_string());
                let _ = tx
                    .send(AppEvent::NetworkActionDone {
                        id,
                        action: "deleted".to_string(),
                        result: result_str,
                    })
                    .await;
            });
        }
    }

    pub fn get_selected_group_networks(&self) -> Option<(String, Vec<String>)> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        if let Some(NetworkRow::GroupHeader(GroupHeader { name, .. })) = rows.get(selected_index) {
            let group_name = name.to_string();
            let mut network_ids = Vec::new();
            for net in &self.networks {
                if let Some(proj) = net.labels.as_ref().and_then(|l| l.get("com.docker.compose.project")) {
                    if proj == &group_name {
                        if let Some(id) = net.id.clone().or_else(|| net.name.clone()) {
                            network_ids.push(id);
                        }
                    }
                }
            }
            if !network_ids.is_empty() {
                return Some((group_name, network_ids));
            }
        }
        None
    }

    pub fn delete_network_group(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
        group_name: &str,
        ids: Vec<String>,
    ) {
        if let Some(client) = client.clone() {
            let count = ids.len();
            *toast = Some(format!("⏳ Deleting {} networks in group {}...", count, group_name));

            tokio::spawn(async move {
                for id in ids {
                    let res = remove_network(&client, &id).await;
                    let result_str = res.map_err(|e| e.to_string());
                    let _ = tx
                        .send(AppEvent::NetworkActionDone {
                            id,
                            action: "deleted".to_string(),
                            result: result_str,
                        })
                        .await;
                }
            });
        }
    }
}
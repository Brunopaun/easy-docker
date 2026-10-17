use std::ops::{Deref, DerefMut};
use bollard::models::Network;
use bollard::Docker;
use tokio::sync::mpsc::Sender;

use crate::components::tab::Tab;
use crate::enums::network::NetworkRow;
use crate::models::generics::GroupHeader;
use crate::services::docker::remove_network;
use crate::AppEvent;

pub struct NetworkTab {
    pub tab: Tab,
    pub networks: Vec<Network>,
}

impl Deref for NetworkTab {
    type Target = Tab;
    fn deref(&self) -> &Self::Target {
        &self.tab
    }
}

impl DerefMut for NetworkTab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tab
    }
}

impl NetworkTab {
    pub fn new() -> Self {
        Self {
            tab: Tab::new(),
            networks: Vec::new(),
        }
    }

    pub fn get_visible_rows(&self) -> Vec<NetworkRow<'_>> {
        self.tab.build_grouped_rows(
            &self.networks,
            |c| {
                c.labels
                    .as_ref()
                    .and_then(|l| l.get("com.docker.compose.project").map(|s| s.as_str()))
            },
            |_| true,
            NetworkRow::GroupHeader,
            |c, is_last| NetworkRow::ChildNetwork {
                network: c,
                is_last_in_group: is_last,
            },
            |c| NetworkRow::StandaloneNetwork { network: c },
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

    pub fn next_network(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_next(count);
    }

    pub fn previous_network(&mut self) {
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
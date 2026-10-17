use bollard::{Docker, models::ContainerSummary, query_parameters::LogsOptions};
use futures_util::StreamExt;
use std::ops::{Deref, DerefMut};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::AppEvent;
use crate::components::tab::Tab;
pub use crate::enums::container::ContainerInspectorView;
use crate::enums::container::ContainerRow;
use crate::models::generics::GroupHeader;
use crate::services::docker::{
    remove_container, restart_container, start_container, stop_container,
};

pub struct ContainersTab {
    pub tab: Tab,
    pub containers: Vec<ContainerSummary>,
    pub inspector_view: ContainerInspectorView,
    pub logs: Vec<String>,
    pub logs_scroll_offset: usize,
    pub auto_scroll: bool,
    pub is_loading_logs: bool,
    pub active_log_container_id: Option<String>,
    pub active_log_container_name: Option<String>,
    pub log_task: Option<JoinHandle<()>>,
    pub visible_height: usize,
}

impl Deref for ContainersTab {
    type Target = Tab;
    fn deref(&self) -> &Self::Target {
        &self.tab
    }
}

impl DerefMut for ContainersTab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tab
    }
}

impl ContainersTab {
    pub fn new() -> Self {
        Self {
            tab: Tab::new(),
            containers: Vec::new(),
            inspector_view: ContainerInspectorView::Details,
            logs: Vec::new(),
            logs_scroll_offset: 0,
            auto_scroll: true,
            is_loading_logs: false,
            active_log_container_id: None,
            active_log_container_name: None,
            log_task: None,
            visible_height: 20,
        }
    }

    pub fn get_visible_rows(&self) -> Vec<ContainerRow<'_>> {
        self.tab.build_grouped_rows(
            &self.containers,
            |c| {
                c.labels
                    .as_ref()
                    .and_then(|l| l.get("com.docker.compose.project").map(|s| s.as_str()))
            },
            |c| c.state.as_ref().map(|s| s.as_ref()) == Some("running"),
            ContainerRow::GroupHeader,
            |c, is_last| ContainerRow::ChildContainer {
                container: c,
                is_last_in_group: is_last,
            },
            |c| ContainerRow::StandaloneContainer { container: c },
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

    pub fn next_container(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_next(count);
    }

    pub fn previous_container(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_previous(count);
    }

    pub fn handle_mouse_click(&mut self, mouse: crossterm::event::MouseEvent) {
        let count = self.get_visible_rows().len();
        if self.tab.handle_mouse_click(mouse, count) {
            self.toggle_group();
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
                let res = start_container(&client, &id)
                    .await
                    .map_err(|e| e.to_string());
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
                let res = stop_container(&client, &id)
                    .await
                    .map_err(|e| e.to_string());
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
                let res = restart_container(&client, &id)
                    .await
                    .map_err(|e| e.to_string());
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
                let res = remove_container(&client, &id)
                    .await
                    .map_err(|e| e.to_string());

                let _ = tx
                    .send(AppEvent::ContainerDeleted { id, result: res })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }

    pub fn get_selected_group_containers(&self) -> Option<(String, Vec<String>)> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        if let Some(ContainerRow::GroupHeader(GroupHeader { name, .. })) = rows.get(selected_index)
        {
            let group_name = name.to_string();
            let mut container_ids = Vec::new();
            for c in &self.containers {
                if let Some(proj) = c
                    .labels
                    .as_ref()
                    .and_then(|l| l.get("com.docker.compose.project"))
                {
                    if proj == &group_name {
                        if let Some(id) = &c.id {
                            container_ids.push(id.clone());
                        }
                    }
                }
            }
            if !container_ids.is_empty() {
                return Some((group_name, container_ids));
            }
        }
        None
    }

    pub fn delete_container_group(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
        group_name: &str,
        ids: Vec<String>,
    ) {
        if let Some(client) = client.clone() {
            let count = ids.len();
            *toast = Some(format!(
                "⏳ Deleting {} containers in group {}...",
                count, group_name
            ));

            tokio::spawn(async move {
                for id in ids {
                    let res = remove_container(&client, &id)
                        .await
                        .map_err(|e| e.to_string());
                    let _ = tx
                        .send(AppEvent::ContainerDeleted { id, result: res })
                        .await;
                }
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }

    pub fn get_selected_container_name(&self) -> Option<String> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        match rows.get(selected_index)? {
            ContainerRow::ChildContainer { container, .. }
            | ContainerRow::StandaloneContainer { container } => container
                .names
                .as_ref()
                .and_then(|n| n.first())
                .map(|s| s.strip_prefix('/').unwrap_or(s).to_string()),
            _ => None,
        }
    }

    pub fn stop_log_stream(&mut self) {
        if let Some(handle) = self.log_task.take() {
            handle.abort();
        }
    }

    pub fn fetch_logs(&mut self, client: &Option<Docker>, tx: Sender<AppEvent>) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_container_id()) {
            let is_same_container = self.active_log_container_id.as_deref() == Some(id.as_str());

            if is_same_container && self.log_task.as_ref().map_or(false, |h| !h.is_finished()) {
                return;
            }

            self.stop_log_stream();

            let name = self
                .get_selected_container_name()
                .unwrap_or_else(|| "container".to_string());

            self.logs.clear();
            self.logs_scroll_offset = 0;
            self.is_loading_logs = true;
            self.active_log_container_id = Some(id.clone());
            self.active_log_container_name = Some(name.clone());

            let handle = tokio::spawn(async move {
                let options = LogsOptions {
                    stdout: true,
                    stderr: true,
                    tail: "150".to_string(),
                    follow: true,
                    timestamps: false,
                    ..Default::default()
                };

                let mut stream = client.logs(&id, Some(options));

                while let Some(log_result) = stream.next().await {
                    match log_result {
                        Ok(output) => {
                            let text = output.to_string();
                            let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
                            if !lines.is_empty() {
                                if tx
                                    .send(AppEvent::ContainerLogChunk {
                                        id: id.clone(),
                                        lines,
                                    })
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx
                                .send(AppEvent::ContainerLogError {
                                    id: id.clone(),
                                    error: e.to_string(),
                                })
                                .await;
                            break;
                        }
                    }
                }
            });

            self.log_task = Some(handle);
        }
    }

    pub fn max_scroll_offset(&self) -> usize {
        self.logs.len().saturating_sub(self.visible_height)
    }

    pub fn scroll_logs_down(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        let max_offset = self.max_scroll_offset();
        if self.logs_scroll_offset < max_offset {
            self.logs_scroll_offset = self.logs_scroll_offset.saturating_add(1);
        }
        self.auto_scroll = false;
    }

    pub fn scroll_logs_up(&mut self) {
        if self.logs.is_empty() {
            return;
        }
        if self.auto_scroll {
            self.logs_scroll_offset = self.max_scroll_offset();
        }
        self.logs_scroll_offset = self.logs_scroll_offset.saturating_sub(1);
        self.auto_scroll = false;
    }

    #[allow(dead_code)]
    pub fn scroll_logs_top(&mut self) {
        self.logs_scroll_offset = 0;
        self.auto_scroll = false;
    }

    #[allow(dead_code)]
    pub fn scroll_logs_bottom(&mut self) {
        self.logs_scroll_offset = self.max_scroll_offset();
        self.auto_scroll = true;
    }

    pub fn toggle_auto_scroll(&mut self) {
        self.auto_scroll = !self.auto_scroll;
        if self.auto_scroll && !self.logs.is_empty() {
            self.logs_scroll_offset = self.max_scroll_offset();
        }
    }
}

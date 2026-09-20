use std::collections::{BTreeMap, HashSet};
use bollard::models::Volume;
use ratatui::widgets::TableState;
use crate::components::generics::GroupHeader;

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
}


#![allow(dead_code)]

use bollard::Docker;
use crate::components::containers::ContainersTab;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ActiveTab {
    Containers,
    Images,
    Volumes,
    Networks,
    System,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ViewMode {
    Normal,
    FullLogs,
    SearchFilter,
    ConfirmDeleteModal,
}

pub struct App {
    pub client: Option<Docker>,
    pub active_tab: ActiveTab,
    pub view_mode: ViewMode,
    pub filter_query: String,
    pub should_quit: bool,
    pub toast_message: Option<String>,
    pub containers_tab: ContainersTab,
}

impl App {
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::Containers,
            view_mode: ViewMode::Normal,
            filter_query: String::new(),
            should_quit: false,
            client: None,
            toast_message: None,
            containers_tab: ContainersTab::new(),
        }
    }
}

#![allow(dead_code)]

use bollard::Docker;

use crate::components:: {
    container::ContainersTab,
    volume::VolumesTab,
    image::ImagesTab,
    network::NetworkTab,
};

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
    SearchFilter,
    ConfirmDeleteModal,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DeleteTarget {
    Volume(String),
    Container(String),
    Image(String),
    Network(String),
}

pub struct App {
    pub client: Option<Docker>,
    pub active_tab: ActiveTab,
    pub view_mode: ViewMode,
    pub filter_query: String,
    pub should_quit: bool,
    pub toast_message: Option<String>,
    pub containers_tab: ContainersTab,
    pub volumes_tab: VolumesTab,
    pub delete_target: Option<DeleteTarget>,
    pub images_tab: ImagesTab,
    pub network_tab: NetworkTab
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
            volumes_tab: VolumesTab::new(),
            delete_target: None,
            images_tab: ImagesTab::new(),
            network_tab: NetworkTab::new(),
        }
    }
}

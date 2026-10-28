#![allow(dead_code)]

use bollard::Docker;
use ratatui::layout::Rect;

use crate::components::{
    container::ContainersTab, image::ImagesTab, network::NetworkTab, volume::VolumesTab,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ActiveTab {
    Containers,
    Images,
    Volumes,
    Networks,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ViewMode {
    Normal,
    SearchFilter,
    ConfirmDeleteModal,
    LogViewer,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum DeleteTarget {
    Volume(String),
    Container(String),
    Image(String),
    Network(String),
    GroupContainers(String, Vec<String>),
    GroupVolumes(String, Vec<String>),
    GroupImages(String, Vec<String>),
    GroupNetworks(String, Vec<String>),
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
    pub network_tab: NetworkTab,
    pub hovered_tab: Option<usize>,
    pub mouse_pos: Option<(u16, u16)>,
    pub header_area: Rect,
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
            hovered_tab: None,
            mouse_pos: None,
            header_area: Rect::default(),
        }
    }

    pub fn get_tab_at_mouse(&self, mouse_col: u16, mouse_row: u16) -> Option<usize> {
        Self::tab_at_mouse(self.header_area, mouse_col, mouse_row)
    }

    pub fn tab_at_mouse(header_area: Rect, mouse_col: u16, mouse_row: u16) -> Option<usize> {
        if mouse_row != header_area.y + 1 {
            return None;
        }

        let titles = ["[1] Containers", "[2] Images", "[3] Volumes", "[4] Networks"];
        let divider_width = 3;
        let mut current_x = header_area.x + 2;

        for (i, title) in titles.iter().enumerate() {
            let title_width = title.chars().count() as u16;
            if mouse_col >= current_x && mouse_col < current_x + title_width {
                return Some(i);
            }
            current_x += title_width + divider_width;
        }

        None
    }
}

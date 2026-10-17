use crate::models::generics::GroupHeader;
use bollard::Docker;
use bollard::models::ImageSummary;
use std::ops::{Deref, DerefMut};
use tokio::sync::mpsc::Sender;

use crate::AppEvent;
use crate::components::tab::Tab;
use crate::enums::image::ImageRow;
use crate::services::docker::remove_image;

pub struct ImagesTab {
    pub tab: Tab,
    pub images: Vec<ImageSummary>,
}

impl Deref for ImagesTab {
    type Target = Tab;
    fn deref(&self) -> &Self::Target {
        &self.tab
    }
}

impl DerefMut for ImagesTab {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tab
    }
}

impl ImagesTab {
    pub fn new() -> Self {
        Self {
            tab: Tab::new(),
            images: Vec::new(),
        }
    }

    pub fn get_visible_rows(&self) -> Vec<ImageRow<'_>> {
        self.tab.build_grouped_rows(
            &self.images,
            |c| {
                c.labels
                    .get("com.docker.compose.project")
                    .map(|s| s.as_str())
            },
            |_| true,
            ImageRow::GroupHeader,
            |c, is_last| ImageRow::ChildImage {
                image: c,
                is_last_in_group: is_last,
            },
            |c| ImageRow::StandaloneImage { image: c },
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

    pub fn next_image(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_next(count);
    }

    pub fn previous_image(&mut self) {
        let count = self.get_visible_rows().len();
        self.tab.select_previous(count);
    }

    pub fn handle_mouse_click(&mut self, mouse: crossterm::event::MouseEvent) {
        let count = self.get_visible_rows().len();
        if self.tab.handle_mouse_click(mouse, count) {
            self.toggle_group();
        }
    }

    pub fn get_selected_image_id(&self) -> Option<String> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        match rows.get(selected_index)? {
            ImageRow::ChildImage { image, .. } | ImageRow::StandaloneImage { image } => {
                Some(image.id.clone())
            }
            _ => None,
        }
    }

    pub fn delete_image(
        &mut self,
        client: &Option<Docker>,
        tx: Sender<AppEvent>,
        toast: &mut Option<String>,
    ) {
        if let (Some(client), Some(id)) = (client.clone(), self.get_selected_image_id()) {
            let clean_id = id.strip_prefix("sha256:").unwrap_or(&id);
            let short_id = clean_id.chars().take(12).collect::<String>();
            *toast = Some(format!("⏳ Deleting image {}...", short_id));

            tokio::spawn(async move {
                let res = remove_image(&client, &id).await.map_err(|e| e.to_string());
                let _ = tx
                    .send(AppEvent::ImageActionDone {
                        id,
                        action: "deleted".into(),
                        result: res,
                    })
                    .await;

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                let _ = tx.send(AppEvent::ClearToast).await;
            });
        }
    }

    pub fn get_selected_group_images(&self) -> Option<(String, Vec<String>)> {
        let selected_index = self.table_state.selected()?;
        let rows = self.get_visible_rows();
        if let Some(ImageRow::GroupHeader(GroupHeader { name, .. })) = rows.get(selected_index) {
            let group_name = name.to_string();
            let mut image_ids = Vec::new();
            for img in &self.images {
                if let Some(proj) = img.labels.get("com.docker.compose.project") {
                    if proj == &group_name {
                        image_ids.push(img.id.clone());
                    }
                }
            }
            if !image_ids.is_empty() {
                return Some((group_name, image_ids));
            }
        }
        None
    }

    pub fn delete_image_group(
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
                "⏳ Deleting {} images in group {}...",
                count, group_name
            ));

            tokio::spawn(async move {
                use crate::services::docker::remove_image;
                for id in ids {
                    let res = remove_image(&client, &id).await.map_err(|e| e.to_string());
                    let _ = tx
                        .send(AppEvent::ImageActionDone {
                            id,
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

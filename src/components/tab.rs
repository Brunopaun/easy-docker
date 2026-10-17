use crate::models::generics::{AsGroupHeader, GroupHeader};
use ratatui::{layout::Rect, widgets::TableState};
use std::collections::{BTreeMap, HashSet};

#[derive(Default)]
pub struct Tab {
    pub table_state: TableState,
    pub collapsed_groups: HashSet<String>,
    pub is_loading: bool,
    pub table_area: Rect,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            collapsed_groups: HashSet::new(),
            is_loading: true,
            table_area: Rect::default(),
        }
    }

    pub fn select_next(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let next = match self.table_state.selected() {
            Some(i) => {
                if i + 1 >= count {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(next));
    }

    pub fn select_previous(&mut self, count: usize) {
        if count == 0 {
            return;
        }
        let prev = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(prev));
    }

    pub fn get_selected_group_header<'a, R: AsGroupHeader<'a>>(
        &self,
        rows: &[R],
    ) -> Option<(String, bool)> {
        let selected = self.table_state.selected()?;
        let row = rows.get(selected)?;
        let header = row.as_group_header()?;
        Some((header.name.to_string(), header.is_expanded))
    }

    pub fn toggle_group_name(&mut self, group_name: &str, is_expanded: bool) {
        if is_expanded {
            self.collapsed_groups.insert(group_name.to_string());
        } else {
            self.collapsed_groups.remove(group_name);
        }
    }

    pub fn adjust_selection(&mut self, new_count: usize) {
        if new_count == 0 {
            self.table_state.select(None);
        } else if let Some(selected) = self.table_state.selected() {
            if selected >= new_count {
                self.table_state.select(Some(new_count - 1));
            }
        }
    }

    pub fn build_grouped_rows<'a, T, R>(
        &self,
        items: &'a [T],
        get_project: impl Fn(&'a T) -> Option<&'a str>,
        is_running: impl Fn(&'a T) -> bool,
        make_header: impl Fn(GroupHeader<'a>) -> R,
        make_child: impl Fn(&'a T, bool) -> R,
        make_standalone: impl Fn(&'a T) -> R,
    ) -> Vec<R> {
        let mut grouped: BTreeMap<&str, Vec<&'a T>> = BTreeMap::new();
        let mut standalone: Vec<&'a T> = Vec::new();

        for item in items {
            if let Some(project) = get_project(item) {
                grouped.entry(project).or_default().push(item);
            } else {
                standalone.push(item);
            }
        }

        let mut rows = Vec::new();

        for (project_name, group_items) in &grouped {
            let total_count = group_items.len();
            let running_count = group_items.iter().filter(|c| is_running(c)).count();
            let is_expanded = !self.collapsed_groups.contains(*project_name);

            rows.push(make_header(GroupHeader {
                name: project_name,
                total_count,
                running_count,
                is_expanded,
            }));

            if is_expanded {
                let len = group_items.len();
                for (idx, c) in group_items.iter().enumerate() {
                    rows.push(make_child(c, idx == len - 1));
                }
            }
        }

        for c in standalone {
            rows.push(make_standalone(c));
        }

        rows
    }

    pub fn handle_mouse_click(
        &mut self,
        mouse: crossterm::event::MouseEvent,
        count: usize,
    ) -> bool {
        if mouse.kind == crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
        {
            let is_within_x = mouse.column >= self.table_area.x
                && mouse.column < self.table_area.x + self.table_area.width;
            let header_offset = self.table_area.y + 2;
            let is_within_y = mouse.row >= header_offset
                && mouse.row < self.table_area.y + self.table_area.height.saturating_sub(1);
            if is_within_x && is_within_y {
                let clicked_row = (mouse.row - header_offset) as usize;
                if clicked_row < count {
                    self.table_state.select(Some(clicked_row));
                    return true;
                }
            }
        }
        false
    }
}

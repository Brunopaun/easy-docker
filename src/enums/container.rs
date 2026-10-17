use crate::models::generics::{AsGroupHeader, GroupHeader};
use bollard::models::ContainerSummary;

pub enum ContainerRow<'a> {
    GroupHeader(GroupHeader<'a>),
    ChildContainer {
        container: &'a ContainerSummary,
        is_last_in_group: bool,
    },
    StandaloneContainer {
        container: &'a ContainerSummary,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContainerInspectorView {
    Details,
    Logs,
}

impl<'a> AsGroupHeader<'a> for ContainerRow<'a> {
    fn as_group_header(&self) -> Option<&GroupHeader<'a>> {
        match self {
            ContainerRow::GroupHeader(h) => Some(h),
            _ => None,
        }
    }
}

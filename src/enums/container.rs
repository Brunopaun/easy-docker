use crate::models::generics::GroupHeader;
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
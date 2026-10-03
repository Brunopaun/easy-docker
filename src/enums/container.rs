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
use crate::models::generics::{AsGroupHeader, GroupHeader};
use bollard::models::Network;
pub enum NetworkRow<'a> {
    GroupHeader(GroupHeader<'a>),
    ChildNetwork {
        network: &'a Network,
        is_last_in_group: bool,
    },
    StandaloneNetwork {
        network: &'a Network,
    },
}

impl<'a> AsGroupHeader<'a> for NetworkRow<'a> {
    fn as_group_header(&self) -> Option<&GroupHeader<'a>> {
        match self {
            NetworkRow::GroupHeader(h) => Some(h),
            _ => None,
        }
    }
}

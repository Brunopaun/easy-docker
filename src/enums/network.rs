use crate::models::generics::GroupHeader;
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
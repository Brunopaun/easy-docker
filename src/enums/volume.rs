use bollard::models::Volume;
use crate::models::generics::GroupHeader;

pub enum VolumeRow<'a> {
    GroupHeader(GroupHeader<'a>),
    ChildVolume {
        volume: &'a Volume,
        is_last_in_group: bool,
    },
    StandaloneVolume {
        volume: &'a Volume,
    },
}
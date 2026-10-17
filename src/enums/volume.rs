use crate::models::generics::{AsGroupHeader, GroupHeader};
use bollard::models::Volume;

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

impl<'a> AsGroupHeader<'a> for VolumeRow<'a> {
    fn as_group_header(&self) -> Option<&GroupHeader<'a>> {
        match self {
            VolumeRow::GroupHeader(h) => Some(h),
            _ => None,
        }
    }
}

use crate::models::generics::{AsGroupHeader, GroupHeader};
use bollard::models::ImageSummary;

pub enum ImageRow<'a> {
    GroupHeader(GroupHeader<'a>),
    ChildImage {
        image: &'a ImageSummary,
        is_last_in_group: bool,
    },
    StandaloneImage {
        image: &'a ImageSummary,
    },
}

impl<'a> AsGroupHeader<'a> for ImageRow<'a> {
    fn as_group_header(&self) -> Option<&GroupHeader<'a>> {
        match self {
            ImageRow::GroupHeader(h) => Some(h),
            _ => None,
        }
    }
}

use crate::models::generics::GroupHeader;
use bollard::models:: {
  ImageSummary
};

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
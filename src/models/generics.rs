pub struct GroupHeader<'a> {
    pub name: &'a str,
    pub total_count: usize,
    pub running_count: usize,
    pub is_expanded: bool,
}

pub trait AsGroupHeader<'a> {
    fn as_group_header(&self) -> Option<&GroupHeader<'a>>;
}
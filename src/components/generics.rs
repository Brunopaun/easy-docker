pub struct GroupHeader<'a> {
    pub name: &'a str,
    pub total_count: usize,
    pub running_count: usize,
    pub is_expanded: bool,
}
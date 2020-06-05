use std::time::SystemTime;

#[derive(Debug, Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageInfo<'a> {
    pub path: &'a str,
    pub modified: SystemTime,
    pub hash: u64,
}

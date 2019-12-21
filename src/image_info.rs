use std::time::SystemTime;

#[derive(Copy, Clone, Default)]
pub struct ImageInfo<'a> {
    pub path: &'a str,
    pub modified: Option<SystemTime>,
    pub hash: Option<u64>,
}


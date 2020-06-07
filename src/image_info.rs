use std::time::SystemTime;

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImageInfo {
    pub path: String,
    pub modified: SystemTime,
    pub file_size: u64,
    pub hash: u64,
}

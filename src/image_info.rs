use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ImageInfo {
    pub path: String,
    pub modified: u64,
    pub file_size: u64,
    pub hash: u64,
}

use std::time::Duration;

use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

use crate::domain::common::url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, CopyGetters, Serialize, Deserialize)]
pub struct NarFileLocation {
    #[getset(get = "pub")]
    source_url: Url,
    #[getset(get_copy = "pub")]
    timeout: Option<Duration>,
}

impl NarFileLocation {
    pub fn new(source_url: Url, timeout: Option<Duration>) -> Self {
        Self {
            source_url,
            timeout,
        }
    }
}

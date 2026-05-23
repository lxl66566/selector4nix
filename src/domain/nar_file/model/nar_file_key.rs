use getset::Getters;

use crate::domain::nar_info::model::NarFileName;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters)]
#[getset(get = "pub")]
pub struct NarFileKey {
    file_hash: String,
    compression: Option<String>,
}

impl NarFileKey {
    pub fn new(file_hash: String) -> Self {
        Self {
            file_hash,
            compression: None,
        }
    }

    pub fn with_compression<V>(mut self, compression: V) -> Self
    where
        V: Into<Option<String>>,
    {
        self.compression = compression.into().filter(|c| !c.is_empty());
        self
    }

    pub fn from_file_name(nar_file: &NarFileName) -> Self {
        let (file_hash, suffix) = nar_file
            .value()
            .split_once(".nar")
            .expect("`nar_file` should contains `\".nar\"`");
        let compression = suffix.trim_start_matches(".");
        Self::new(file_hash.to_string()).with_compression(compression.to_string())
    }

    pub fn to_file_name(&self) -> NarFileName {
        if let Some(compression) = &self.compression {
            NarFileName::new(format!("{}.nar.{}", self.file_hash, compression))
                .expect("converting `NarFileKey` to `NarFileName` should always be valid")
        } else {
            NarFileName::new(format!("{}.nar", self.file_hash))
                .expect("converting `NarFileKey` to `NarFileName` should always be valid")
        }
    }
}

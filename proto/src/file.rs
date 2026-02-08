use std::path::Path;

use snafu::{ResultExt, Snafu};

use crate::entry::Entry;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Failed to read file: {}", source))]
    ReadFile { source: std::io::Error },
    #[snafu(display("Failed to write file: {}", source))]
    WriteFile { source: std::io::Error },
    #[cfg(feature = "json")]
    #[snafu(display("Failed to parse JSON: {}", source))]
    ParseJson { source: serde_json::Error },
    #[cfg(feature = "json")]
    #[snafu(display("Failed to serialize JSON: {}", source))]
    SerializeJson { source: serde_json::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Entry {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            #[cfg(feature = "json")]
            "json" => Self::load_json(path),
            _ => Self::load_json(path), // Default to JSON
        }
    }

    #[cfg(feature = "json")]
    pub fn load_json<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).context(ReadFileSnafu)?;
        serde_json::from_str(&content).context(ParseJsonSnafu)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match extension {
            #[cfg(feature = "json")]
            "json" => self.save_json(path),
            _ => self.save_json(path), // Default to JSON
        }
    }

    #[cfg(feature = "json")]
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self).context(SerializeJsonSnafu)?;
        std::fs::write(path, content).context(WriteFileSnafu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use indexmap::IndexMap;

    #[test]
    #[cfg(feature = "json")]
    fn test_save_and_load_json() {
        let entry = Entry {
            id: Uuid::new_v4(),
            tags: IndexMap::new(),
            aged: IndexMap::new(),
            memo: "test memo".to_string(),
        };

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_entry.json");

        entry.save_json(&path).unwrap();
        let loaded = Entry::load_json(&path).unwrap();

        assert_eq!(entry.id, loaded.id);
        assert_eq!(entry.memo, loaded.memo);

        std::fs::remove_file(&path).ok();
    }
}

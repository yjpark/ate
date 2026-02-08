use std::path::{Path, PathBuf};
use std::collections::HashSet;
use snafu::{ResultExt, Snafu};

use crate::prelude::{ProtoEntry, GroupTag, GroupData, LeafData};

#[derive(Debug, Snafu)]
pub enum LoaderError {
    #[snafu(display("Failed to read directory {}: {}", path.display(), source))]
    ReadDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to read file {}: {}", path.display(), source))]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Failed to parse JSON from {}: {}", path.display(), source))]
    ParseJson {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[snafu(display("Serde feature not enabled"))]
    SerdeNotEnabled,
}

pub type Result<T> = std::result::Result<T, LoaderError>;

/// Recursively scan a directory for all JSON files
pub fn scan_json_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut json_files = Vec::new();
    scan_json_files_recursive(path, &mut json_files)?;
    Ok(json_files)
}

fn scan_json_files_recursive(path: &Path, json_files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = std::fs::read_dir(path).context(ReadDirSnafu { path })?;

    for entry in entries {
        let entry = entry.context(ReadDirSnafu { path })?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            scan_json_files_recursive(&entry_path, json_files)?;
        } else if entry_path.extension().and_then(|s| s.to_str()) == Some("json") {
            json_files.push(entry_path);
        }
    }

    Ok(())
}

/// Load proto entries from JSON files
#[cfg(feature = "serde")]
pub fn load_entries(paths: &[PathBuf]) -> Result<Vec<ProtoEntry>> {
    let mut entries = Vec::new();

    for path in paths {
        let content = std::fs::read_to_string(path).context(ReadFileSnafu { path })?;
        let entry: ProtoEntry = serde_json::from_str(&content).context(ParseJsonSnafu { path })?;
        entries.push(entry);
    }

    Ok(entries)
}

#[cfg(not(feature = "serde"))]
pub fn load_entries(_paths: &[PathBuf]) -> Result<Vec<ProtoEntry>> {
    Err(LoaderError::SerdeNotEnabled)
}

/// Infer GroupTags from InFolder references in entries
pub fn infer_group_tags(entries: &[ProtoEntry]) -> Vec<GroupTag> {
    let mut tag_uuids = HashSet::new();

    // Collect all unique InFolder references
    for entry in entries {
        for (_tag_id, leaf_tag) in &entry.tags {
            if let LeafData::InFolder(folder_uuid) = &leaf_tag.data {
                tag_uuids.insert(*folder_uuid);
            }
        }
    }

    // Create placeholder GroupTag for each UUID
    tag_uuids
        .into_iter()
        .map(|uuid| GroupTag {
            id: uuid,
            parent: None, // All inferred tags are roots
            data: GroupData::Custom {
                kind: "inferred".to_string(),
                value: uuid.to_string(),
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn test_scan_json_files() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        // Create test files
        fs::write(base_path.join("file1.json"), "{}").unwrap();
        fs::write(base_path.join("file2.txt"), "not json").unwrap();
        fs::create_dir(base_path.join("subdir")).unwrap();
        fs::write(base_path.join("subdir/file3.json"), "{}").unwrap();

        let json_files = scan_json_files(base_path).unwrap();

        assert_eq!(json_files.len(), 2);
        assert!(json_files.iter().all(|p| p.extension().unwrap() == "json"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_load_entries() {
        use indexmap::IndexMap;

        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();

        let entry = ProtoEntry {
            id: Uuid::new_v4(),
            tags: IndexMap::new(),
            aged: IndexMap::new(),
            memo: "test entry".to_string(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        fs::write(base_path.join("entry.json"), json).unwrap();

        let paths = vec![base_path.join("entry.json")];
        let entries = load_entries(&paths).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, entry.id);
        assert_eq!(entries[0].memo, entry.memo);
    }

    #[test]
    fn test_infer_group_tags() {
        use crate::prelude::LeafTag;
        use indexmap::IndexMap;

        let folder_id1 = Uuid::new_v4();
        let folder_id2 = Uuid::new_v4();

        let mut tags1 = IndexMap::new();
        tags1.insert(
            Uuid::new_v4(),
            LeafTag {
                id: Uuid::new_v4(),
                data: LeafData::InFolder(folder_id1),
            },
        );

        let mut tags2 = IndexMap::new();
        tags2.insert(
            Uuid::new_v4(),
            LeafTag {
                id: Uuid::new_v4(),
                data: LeafData::InFolder(folder_id2),
            },
        );
        tags2.insert(
            Uuid::new_v4(),
            LeafTag {
                id: Uuid::new_v4(),
                data: LeafData::InFolder(folder_id1), // Duplicate
            },
        );

        let entries = vec![
            ProtoEntry {
                id: Uuid::new_v4(),
                tags: tags1,
                aged: IndexMap::new(),
                memo: String::new(),
            },
            ProtoEntry {
                id: Uuid::new_v4(),
                tags: tags2,
                aged: IndexMap::new(),
                memo: String::new(),
            },
        ];

        let group_tags = infer_group_tags(&entries);

        assert_eq!(group_tags.len(), 2); // Deduplicated
        assert!(group_tags.iter().any(|t| t.id == folder_id1));
        assert!(group_tags.iter().any(|t| t.id == folder_id2));
        assert!(group_tags.iter().all(|t| t.parent.is_none()));
        assert!(group_tags.iter().all(|t| matches!(
            &t.data,
            GroupData::Custom { kind, .. } if kind == "inferred"
        )));
    }
}

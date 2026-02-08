use std::path::Path;
use uuid::Uuid;
use indexmap::IndexMap;
use chrono::{Utc, TimeZone};
use chrono_tz::America::New_York;

use crate::prelude::{ProtoEntry, LeafTag, LeafData, Recipient};

/// Generate deterministic UUID from seed
fn uuid_from_seed(seed: u128) -> Uuid {
    Uuid::from_u128(seed)
}

/// Mock data generator for testing
pub struct MockDataGenerator {
    /// Base seed for deterministic UUID generation
    seed: u128,
}

impl MockDataGenerator {
    /// Create a new mock data generator with a seed
    pub fn new(seed: u128) -> Self {
        Self { seed }
    }

    /// Create with default seed
    pub fn default() -> Self {
        Self::new(0x1234567890abcdef)
    }

    /// Generate deterministic folder UUIDs
    pub fn folder_uuids(&self) -> FolderUuids {
        FolderUuids {
            projects: uuid_from_seed(self.seed + 1000),
            project_a: uuid_from_seed(self.seed + 1001),
            project_b: uuid_from_seed(self.seed + 1002),
            project_c: uuid_from_seed(self.seed + 1003),
            archive: uuid_from_seed(self.seed + 1004),
            year_2025: uuid_from_seed(self.seed + 2000),
            jan_2025: uuid_from_seed(self.seed + 2001),
            feb_2025: uuid_from_seed(self.seed + 2002),
            mar_2025: uuid_from_seed(self.seed + 2003),
            alice: uuid_from_seed(self.seed + 3000),
            bob: uuid_from_seed(self.seed + 3001),
            charlie: uuid_from_seed(self.seed + 3002),
        }
    }

    /// Generate ~50 mock entries with ~12 unique folder references
    pub fn generate_entries(&self) -> Vec<ProtoEntry> {
        let folders = self.folder_uuids();
        let mut entries = Vec::new();

        // Projects folder entries (20 entries)
        for i in 0..20 {
            let entry_id = uuid_from_seed(self.seed + 10000 + i);
            let folder = match i % 5 {
                0 => folders.project_a,
                1 => folders.project_b,
                2 => folders.project_c,
                _ => folders.projects,
            };

            let mut tags = IndexMap::new();
            tags.insert(
                uuid_from_seed(self.seed + 20000 + i),
                LeafTag {
                    id: uuid_from_seed(self.seed + 20000 + i),
                    data: LeafData::InFolder(folder),
                },
            );

            // Add timestamp
            let days_ago = i as i64;
            let timestamp = Utc::now() - chrono::Duration::days(days_ago);
            tags.insert(
                uuid_from_seed(self.seed + 30000 + i),
                LeafTag {
                    id: uuid_from_seed(self.seed + 30000 + i),
                    data: LeafData::Added(timestamp, New_York),
                },
            );

            // Some entries have recipients
            if i % 3 == 0 {
                let recipient = match i % 9 {
                    0 => Recipient {
                        name: "Alice".to_string(),
                        pub_key: "age1alice...".to_string(),
                    },
                    3 => Recipient {
                        name: "Bob".to_string(),
                        pub_key: "age1bob...".to_string(),
                    },
                    _ => Recipient {
                        name: "Charlie".to_string(),
                        pub_key: "age1charlie...".to_string(),
                    },
                };
                tags.insert(
                    uuid_from_seed(self.seed + 40000 + i),
                    LeafTag {
                        id: uuid_from_seed(self.seed + 40000 + i),
                        data: LeafData::Recipient(recipient),
                    },
                );
            }

            entries.push(ProtoEntry {
                id: entry_id,
                tags,
                aged: IndexMap::new(),
                memo: format!("Project entry {}", i + 1),
            });
        }

        // Archive entries (15 entries)
        for i in 0..15 {
            let entry_id = uuid_from_seed(self.seed + 11000 + i);
            let mut tags = IndexMap::new();

            tags.insert(
                uuid_from_seed(self.seed + 21000 + i),
                LeafTag {
                    id: uuid_from_seed(self.seed + 21000 + i),
                    data: LeafData::InFolder(folders.archive),
                },
            );

            let days_ago = 30 + i as i64;
            let timestamp = Utc::now() - chrono::Duration::days(days_ago);
            tags.insert(
                uuid_from_seed(self.seed + 31000 + i),
                LeafTag {
                    id: uuid_from_seed(self.seed + 31000 + i),
                    data: LeafData::Added(timestamp, New_York),
                },
            );

            entries.push(ProtoEntry {
                id: entry_id,
                tags,
                aged: IndexMap::new(),
                memo: format!("Archive entry {}", i + 1),
            });
        }

        // Temporal organization entries (15 entries)
        for i in 0..15 {
            let entry_id = uuid_from_seed(self.seed + 12000 + i);
            let mut tags = IndexMap::new();

            // Distribute across months
            let folder = match i % 3 {
                0 => folders.jan_2025,
                1 => folders.feb_2025,
                _ => folders.mar_2025,
            };

            tags.insert(
                uuid_from_seed(self.seed + 22000 + i),
                LeafTag {
                    id: uuid_from_seed(self.seed + 22000 + i),
                    data: LeafData::InFolder(folder),
                },
            );

            // Use specific dates based on month
            let month = (i % 3) + 1;
            let day = (i % 28) + 1;
            let timestamp = Utc.with_ymd_and_hms(2025, month as u32, day as u32, 12, 0, 0).unwrap();

            tags.insert(
                uuid_from_seed(self.seed + 32000 + i),
                LeafTag {
                    id: uuid_from_seed(self.seed + 32000 + i),
                    data: LeafData::Added(timestamp, New_York),
                },
            );

            // Some have updates
            if i % 5 == 0 {
                let update_timestamp = timestamp + chrono::Duration::days(i as i64 % 7);
                tags.insert(
                    uuid_from_seed(self.seed + 33000 + i),
                    LeafTag {
                        id: uuid_from_seed(self.seed + 33000 + i),
                        data: LeafData::Updated(update_timestamp, New_York),
                    },
                );
            }

            entries.push(ProtoEntry {
                id: entry_id,
                tags,
                aged: IndexMap::new(),
                memo: format!("Temporal entry {}", i + 1),
            });
        }

        entries
    }

    /// Write entries to a directory as individual JSON files
    #[cfg(feature = "serde")]
    pub fn write_to_directory(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)?;

        let entries = self.generate_entries();
        for (i, entry) in entries.iter().enumerate() {
            let filename = format!("entry_{:03}.json", i + 1);
            let path = dir.join(filename);
            let json = serde_json::to_string_pretty(entry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            std::fs::write(path, json)?;
        }

        Ok(())
    }

    #[cfg(not(feature = "serde"))]
    pub fn write_to_directory(&self, _dir: &Path) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Serde feature not enabled",
        ))
    }
}

/// Container for all folder UUIDs used in mock data
#[derive(Debug, Clone)]
pub struct FolderUuids {
    pub projects: Uuid,
    pub project_a: Uuid,
    pub project_b: Uuid,
    pub project_c: Uuid,
    pub archive: Uuid,
    pub year_2025: Uuid,
    pub jan_2025: Uuid,
    pub feb_2025: Uuid,
    pub mar_2025: Uuid,
    pub alice: Uuid,
    pub bob: Uuid,
    pub charlie: Uuid,
}

impl FolderUuids {
    /// Get all unique folder UUIDs
    pub fn all(&self) -> Vec<Uuid> {
        vec![
            self.projects,
            self.project_a,
            self.project_b,
            self.project_c,
            self.archive,
            self.year_2025,
            self.jan_2025,
            self.feb_2025,
            self.mar_2025,
            self.alice,
            self.bob,
            self.charlie,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_uuids() {
        let gen1 = MockDataGenerator::new(12345);
        let gen2 = MockDataGenerator::new(12345);

        let uuids1 = gen1.folder_uuids();
        let uuids2 = gen2.folder_uuids();

        assert_eq!(uuids1.projects, uuids2.projects);
        assert_eq!(uuids1.archive, uuids2.archive);
    }

    #[test]
    fn test_generate_entries_count() {
        let gen = MockDataGenerator::default();
        let entries = gen.generate_entries();

        // Should generate 50 entries total (20 + 15 + 15)
        assert_eq!(entries.len(), 50);
    }

    #[test]
    fn test_generate_entries_have_folders() {
        let gen = MockDataGenerator::default();
        let entries = gen.generate_entries();

        // All entries should have at least one InFolder tag
        for entry in &entries {
            let has_folder = entry.tags.values().any(|tag| {
                matches!(tag.data, LeafData::InFolder(_))
            });
            assert!(has_folder, "Entry {} should have InFolder tag", entry.id);
        }
    }

    #[test]
    fn test_folder_uuids_unique() {
        let gen = MockDataGenerator::default();
        let uuids = gen.folder_uuids();
        let all = uuids.all();

        // All UUIDs should be unique
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "UUIDs at {} and {} should be different", i, j);
            }
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_write_to_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let gen = MockDataGenerator::default();

        gen.write_to_directory(temp_dir.path()).unwrap();

        // Check that files were created
        let entries = std::fs::read_dir(temp_dir.path()).unwrap();
        let count = entries.count();
        assert_eq!(count, 50);
    }
}

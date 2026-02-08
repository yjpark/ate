use std::sync::Arc;
use std::path::Path;
use dashmap::DashMap;
use snafu::{ResultExt, Snafu};
use uuid::Uuid;

use crate::tag::Tag;
use crate::entry::Entry;
use crate::loader;
use crate::converter;
use crate::prelude::ProtoTag;

#[derive(Debug, Snafu)]
pub enum DatabaseError {
    #[snafu(display("Loader error: {}", source))]
    Loader { source: loader::LoaderError },

    #[snafu(display("Converter error: {}", source))]
    Converter { source: converter::ConverterError },
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

/// Central container managing all tags and entries
#[derive(Debug, Default)]
pub struct Database {
    /// Root tags (no parent)
    roots: DashMap<Uuid, Tag>,
    /// All tags by ID
    tags: DashMap<Uuid, Tag>,
    /// All entries by ID
    entries: DashMap<Uuid, Arc<Entry>>,
}

impl Database {
    /// Create a new empty database
    pub fn new() -> Self {
        Self::default()
    }

    /// Load database from a folder containing JSON entry files
    pub fn load_from_folder(path: &Path) -> Result<Self> {
        let db = Self::new();

        // Scan for JSON files
        let json_files = loader::scan_json_files(path).context(LoaderSnafu)?;

        // Load proto entries
        let proto_entries = loader::load_entries(&json_files).context(LoaderSnafu)?;

        // Infer group tags from InFolder references
        let group_tags = loader::infer_group_tags(&proto_entries);

        // Build tag hierarchy
        let tag_map = converter::build_tag_hierarchy(group_tags);

        // Convert entries with weak tag references
        let entry_map = converter::convert_entries(&proto_entries, &tag_map).context(ConverterSnafu)?;

        // Link entries to tags (add strong references to tag items)
        converter::link_entries_to_tags(&proto_entries, &tag_map, &entry_map).context(ConverterSnafu)?;

        // Populate database
        for (id, tag) in tag_map {
            db.tags.insert(id, tag.clone());
            // If tag has no parent, it's a root
            if let ProtoTag::Group(group_tag) = tag.proto().as_ref() {
                if group_tag.parent.is_none() {
                    db.roots.insert(id, tag);
                }
            }
        }

        for (id, entry) in entry_map {
            db.entries.insert(id, entry);
        }

        Ok(db)
    }

    /// Add a tag to the database
    pub fn add_tag(&self, tag: Tag) {
        let id = tag.proto().id();
        self.tags.insert(id, tag.clone());

        // Check if it's a root tag
        if let ProtoTag::Group(group_tag) = tag.proto().as_ref() {
            if group_tag.parent.is_none() {
                self.roots.insert(id, tag);
            }
        }
    }

    /// Get a tag by ID
    pub fn get_tag(&self, id: &Uuid) -> Option<Tag> {
        self.tags.get(id).map(|t| t.value().clone())
    }

    /// Get all root tags
    pub fn root_tags(&self) -> Vec<Tag> {
        self.roots.iter().map(|t| t.value().clone()).collect()
    }

    /// Get all tags
    pub fn all_tags(&self) -> Vec<Tag> {
        self.tags.iter().map(|t| t.value().clone()).collect()
    }

    /// Add an entry to the database
    pub fn add_entry(&self, entry: Arc<Entry>) {
        self.entries.insert(entry.id, entry);
    }

    /// Get an entry by ID
    pub fn get_entry(&self, id: &Uuid) -> Option<Arc<Entry>> {
        self.entries.get(id).map(|e| Arc::clone(e.value()))
    }

    /// Get all entries
    pub fn all_entries(&self) -> Vec<Arc<Entry>> {
        self.entries.iter().map(|e| Arc::clone(e.value())).collect()
    }

    /// Get entries in a specific tag (non-recursive)
    pub fn entries_in_tag(&self, tag_id: &Uuid) -> Vec<Arc<Entry>> {
        self.get_tag(tag_id)
            .map(|tag| tag.entries())
            .unwrap_or_default()
    }

    /// Get entries in a tag and all its children (recursive)
    pub fn entries_in_tag_recursive(&self, tag_id: &Uuid) -> Vec<Arc<Entry>> {
        self.get_tag(tag_id)
            .map(|tag| tag.entries_recursive())
            .unwrap_or_default()
    }

    /// Get the number of tags in the database
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// Get the number of entries in the database
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get the number of root tags in the database
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_database() {
        let db = Database::new();
        assert_eq!(db.tag_count(), 0);
        assert_eq!(db.entry_count(), 0);
        assert_eq!(db.root_count(), 0);
    }

    #[test]
    fn test_add_get_tag() {
        use crate::prelude::{GroupTag, GroupData};
        use crate::tag::Arc as TagArc;

        let db = Database::new();
        let id = Uuid::new_v4();

        let proto_tag = ProtoTag::Group(GroupTag {
            id,
            parent: None,
            data: GroupData::Folder {
                name: "Test".to_string(),
            },
        });

        let tag = TagArc::new(proto_tag);
        db.add_tag(tag.clone());

        assert_eq!(db.tag_count(), 1);
        assert_eq!(db.root_count(), 1);

        let retrieved = db.get_tag(&id).unwrap();
        assert_eq!(retrieved.proto().id(), id);
    }

    #[test]
    fn test_add_get_entry() {
        let db = Database::new();
        let id = Uuid::new_v4();

        let entry = Entry::from_proto(&crate::prelude::ProtoEntry {
            id,
            tags: indexmap::IndexMap::new(),
            aged: indexmap::IndexMap::new(),
            memo: "test".to_string(),
        });

        db.add_entry(entry.clone());

        assert_eq!(db.entry_count(), 1);

        let retrieved = db.get_entry(&id).unwrap();
        assert_eq!(retrieved.id, id);
    }
}

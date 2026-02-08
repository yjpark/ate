use std::sync::Arc as StdArc;
use std::collections::HashMap;
use snafu::Snafu;
use uuid::Uuid;

use crate::prelude::{ProtoEntry, ProtoTag, GroupTag, LeafData};
use crate::tag::{Tag, Arc as TagArc};
use crate::entry::Entry;

#[derive(Debug, Snafu)]
pub enum ConverterError {
    #[snafu(display("Tag {} not found when linking entry {}", tag_id, entry_id))]
    TagNotFound { tag_id: Uuid, entry_id: Uuid },
}

pub type Result<T> = std::result::Result<T, ConverterError>;

/// Build tag hierarchy from GroupTags, returning a map of tag ID to Tag
pub fn build_tag_hierarchy(group_tags: Vec<GroupTag>) -> HashMap<Uuid, Tag> {
    let mut tags = HashMap::new();

    // First pass: create all tags
    for group_tag in &group_tags {
        let proto_tag = ProtoTag::Group(group_tag.clone());
        let tag = TagArc::new(proto_tag);
        tags.insert(group_tag.id, tag);
    }

    // Second pass: establish parent-child relationships
    for group_tag in &group_tags {
        if let Some(parent_id) = group_tag.parent {
            if let (Some(parent), Some(child)) = (tags.get(&parent_id), tags.get(&group_tag.id)) {
                parent.add_child(child.clone());
            }
        }
    }

    tags
}

/// Convert proto entries to model entries with weak tag references
pub fn convert_entries(
    proto_entries: &[ProtoEntry],
    tags: &HashMap<Uuid, Tag>,
) -> Result<HashMap<Uuid, StdArc<Entry>>> {
    let mut entries = HashMap::new();

    for proto_entry in proto_entries {
        let entry = Entry::from_proto(proto_entry);

        // Add weak tag references for InFolder tags
        for (tag_id, leaf_tag) in &proto_entry.tags {
            if let LeafData::InFolder(folder_id) = &leaf_tag.data {
                if let Some(tag) = tags.get(folder_id) {
                    entry.add_tag(*tag_id, tag.downgrade());
                }
                // Note: If tag not found, we skip it (entry exists but not in any folder)
            }
        }

        entries.insert(proto_entry.id, entry);
    }

    Ok(entries)
}

/// Link entries to tags by adding strong Arc<Entry> references to tag items
pub fn link_entries_to_tags(
    proto_entries: &[ProtoEntry],
    tags: &HashMap<Uuid, Tag>,
    entries: &HashMap<Uuid, StdArc<Entry>>,
) -> Result<()> {
    for proto_entry in proto_entries {
        let entry = entries
            .get(&proto_entry.id)
            .expect("Entry should exist in entries map");

        // Add entry to each InFolder tag
        for leaf_tag in proto_entry.tags.values() {
            if let LeafData::InFolder(folder_id) = &leaf_tag.data {
                if let Some(tag) = tags.get(folder_id) {
                    tag.add_entry(StdArc::clone(entry));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::GroupData;
    use indexmap::IndexMap;

    #[test]
    fn test_build_tag_hierarchy_no_parents() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let group_tags = vec![
            GroupTag {
                id: id1,
                parent: None,
                data: GroupData::Folder {
                    name: "Folder1".to_string(),
                },
            },
            GroupTag {
                id: id2,
                parent: None,
                data: GroupData::Folder {
                    name: "Folder2".to_string(),
                },
            },
        ];

        let tags = build_tag_hierarchy(group_tags);

        assert_eq!(tags.len(), 2);
        assert!(tags.contains_key(&id1));
        assert!(tags.contains_key(&id2));

        // Both should be roots (no parent)
        assert!(tags.get(&id1).unwrap().children().is_empty());
        assert!(tags.get(&id2).unwrap().children().is_empty());
    }

    #[test]
    fn test_build_tag_hierarchy_with_parents() {
        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();

        let group_tags = vec![
            GroupTag {
                id: parent_id,
                parent: None,
                data: GroupData::Folder {
                    name: "Parent".to_string(),
                },
            },
            GroupTag {
                id: child_id,
                parent: Some(parent_id),
                data: GroupData::Folder {
                    name: "Child".to_string(),
                },
            },
        ];

        let tags = build_tag_hierarchy(group_tags);

        assert_eq!(tags.len(), 2);

        let parent = tags.get(&parent_id).unwrap();
        let children = parent.children();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].proto().id(), child_id);
    }

    #[test]
    fn test_convert_entries() {
        use crate::prelude::LeafTag;

        let folder_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let group_tag = GroupTag {
            id: folder_id,
            parent: None,
            data: GroupData::Custom {
                kind: "inferred".to_string(),
                value: folder_id.to_string(),
            },
        };

        let tags = build_tag_hierarchy(vec![group_tag]);

        let mut proto_tags = IndexMap::new();
        proto_tags.insert(
            tag_id,
            LeafTag {
                id: tag_id,
                data: LeafData::InFolder(folder_id),
            },
        );

        let proto_entries = vec![ProtoEntry {
            id: entry_id,
            tags: proto_tags,
            aged: IndexMap::new(),
            memo: "test".to_string(),
        }];

        let entries = convert_entries(&proto_entries, &tags).unwrap();

        assert_eq!(entries.len(), 1);
        let entry = entries.get(&entry_id).unwrap();
        assert_eq!(entry.id, entry_id);
        assert_eq!(entry.tags.len(), 1);
        assert!(entry.tags.contains_key(&tag_id));
    }

    #[test]
    fn test_link_entries_to_tags() {
        use crate::prelude::LeafTag;

        let folder_id = Uuid::new_v4();
        let entry_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        let group_tag = GroupTag {
            id: folder_id,
            parent: None,
            data: GroupData::Custom {
                kind: "inferred".to_string(),
                value: folder_id.to_string(),
            },
        };

        let tags = build_tag_hierarchy(vec![group_tag]);

        let mut proto_tags = IndexMap::new();
        proto_tags.insert(
            tag_id,
            LeafTag {
                id: tag_id,
                data: LeafData::InFolder(folder_id),
            },
        );

        let proto_entries = vec![ProtoEntry {
            id: entry_id,
            tags: proto_tags,
            aged: IndexMap::new(),
            memo: "test".to_string(),
        }];

        let entries = convert_entries(&proto_entries, &tags).unwrap();
        link_entries_to_tags(&proto_entries, &tags, &entries).unwrap();

        // Verify the tag now contains the entry
        let tag = tags.get(&folder_id).unwrap();
        let tag_entries = tag.entries();
        assert_eq!(tag_entries.len(), 1);
        assert_eq!(tag_entries[0].id, entry_id);
    }
}

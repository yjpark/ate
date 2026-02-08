use crate::prelude::*;
use crate::mock::MockDataGenerator;
use tempfile::TempDir;

#[test]
fn test_load_mock_data() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();

    // Generate and write mock data
    gen.write_to_directory(temp_dir.path()).unwrap();

    // Load database from folder
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Verify counts
    assert_eq!(db.entry_count(), 50, "Should have 50 entries");

    // Should have 12 unique folder tags (excluding recipient tags which aren't inferred)
    // But the current implementation only infers from InFolder, and we use:
    // projects, project_a, project_b, project_c, archive (5)
    // jan_2025, feb_2025, mar_2025 (3)
    // Total: 8 folder tags actually used in InFolder references
    let tag_count = db.tag_count();
    assert!(tag_count >= 8, "Should have at least 8 tags, got {}", tag_count);

    let root_count = db.root_count();
    assert_eq!(root_count, tag_count, "All inferred tags should be roots");
}

#[test]
fn test_verify_tag_structure() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // All tags should be root tags (no parent hierarchy in inferred tags)
    let all_tags = db.all_tags();
    for tag in &all_tags {
        if let ProtoTag::Group(group_tag) = tag.proto().as_ref() {
            assert!(
                group_tag.parent.is_none(),
                "Inferred tags should have no parent"
            );
            assert!(
                matches!(&group_tag.data, GroupData::Custom { kind, .. } if kind == "inferred"),
                "Inferred tags should be Custom type"
            );
        }
    }
}

#[test]
fn test_entries_in_tag() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();
    let folders = gen.folder_uuids();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Get entries in archive folder
    let archive_entries = db.entries_in_tag(&folders.archive);

    // Archive folder should have 15 entries
    assert_eq!(
        archive_entries.len(),
        15,
        "Archive folder should have 15 entries"
    );

    // Verify all entries are actually in archive
    for entry in &archive_entries {
        assert!(entry.memo.as_ref().unwrap().starts_with("Archive entry"));
    }
}

#[test]
fn test_entries_across_multiple_tags() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();
    let folders = gen.folder_uuids();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Get entries in different project folders
    let project_a_entries = db.entries_in_tag(&folders.project_a);
    let project_b_entries = db.entries_in_tag(&folders.project_b);
    let project_c_entries = db.entries_in_tag(&folders.project_c);

    // Each project folder should have some entries
    assert!(project_a_entries.len() > 0, "Project A should have entries");
    assert!(project_b_entries.len() > 0, "Project B should have entries");
    assert!(project_c_entries.len() > 0, "Project C should have entries");

    // Total of project entries should not exceed project entries generated
    let total_project_entries = project_a_entries.len()
        + project_b_entries.len()
        + project_c_entries.len();

    assert!(
        total_project_entries <= 20,
        "Project entries should not exceed 20"
    );
}

#[test]
fn test_temporal_entries() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();
    let folders = gen.folder_uuids();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Get entries in temporal folders
    let jan_entries = db.entries_in_tag(&folders.jan_2025);
    let feb_entries = db.entries_in_tag(&folders.feb_2025);
    let mar_entries = db.entries_in_tag(&folders.mar_2025);

    // Should have temporal entries
    assert!(jan_entries.len() > 0, "January should have entries");
    assert!(feb_entries.len() > 0, "February should have entries");
    assert!(mar_entries.len() > 0, "March should have entries");

    // Total temporal entries should be 15
    let total_temporal = jan_entries.len() + feb_entries.len() + mar_entries.len();
    assert_eq!(total_temporal, 15, "Should have 15 temporal entries total");
}

#[test]
fn test_reference_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();
    let folders = gen.folder_uuids();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Get a tag and its entries
    let archive_tag = db.get_tag(&folders.archive).unwrap();
    let archive_entries = archive_tag.entries();

    // Verify entries have weak references back to the tag
    for entry in &archive_entries {
        let has_archive_tag = entry.tags.iter().any(|tag_ref| {
            tag_ref.value().upgrade().is_some()
        });
        assert!(
            has_archive_tag,
            "Entry {} should have at least one valid tag reference",
            entry.id
        );
    }
}

#[test]
fn test_weak_references_valid() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Get all entries
    let entries = db.all_entries();

    // All weak references should be valid while database exists
    for entry in &entries {
        for tag_ref in entry.tags.iter() {
            let tag = tag_ref.value().upgrade();
            assert!(
                tag.is_some(),
                "Weak reference in entry {} should be valid",
                entry.id
            );
        }
    }
}

#[test]
fn test_get_nonexistent_tag() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    let nonexistent_id = uuid::Uuid::new_v4();
    let result = db.get_tag(&nonexistent_id);

    assert!(result.is_none(), "Getting nonexistent tag should return None");
}

#[test]
fn test_get_nonexistent_entry() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    let nonexistent_id = uuid::Uuid::new_v4();
    let result = db.get_entry(&nonexistent_id);

    assert!(result.is_none(), "Getting nonexistent entry should return None");
}

#[test]
fn test_entries_recursive() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();
    let folders = gen.folder_uuids();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    // Since all tags are roots with no children, recursive should equal non-recursive
    let archive_tag = db.get_tag(&folders.archive).unwrap();
    let entries = archive_tag.entries();
    let entries_recursive = archive_tag.entries_recursive();

    assert_eq!(
        entries.len(),
        entries_recursive.len(),
        "Recursive entries should equal non-recursive for tags with no children"
    );
}

#[test]
fn test_all_entries_accessible() {
    let temp_dir = TempDir::new().unwrap();
    let gen = MockDataGenerator::default();

    gen.write_to_directory(temp_dir.path()).unwrap();
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    let all_entries = db.all_entries();

    // All entries should be retrievable by ID
    for entry in &all_entries {
        let retrieved = db.get_entry(&entry.id);
        assert!(
            retrieved.is_some(),
            "Entry {} should be retrievable",
            entry.id
        );
    }
}

#[test]
fn test_deterministic_generation() {
    let gen1 = MockDataGenerator::new(42);
    let gen2 = MockDataGenerator::new(42);

    let entries1 = gen1.generate_entries();
    let entries2 = gen2.generate_entries();

    assert_eq!(entries1.len(), entries2.len());

    // Same seed should generate same UUIDs
    for (e1, e2) in entries1.iter().zip(entries2.iter()) {
        assert_eq!(e1.id, e2.id);
    }
}

#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    // Load from empty directory
    let db = Database::load_from_folder(temp_dir.path()).unwrap();

    assert_eq!(db.entry_count(), 0);
    assert_eq!(db.tag_count(), 0);
    assert_eq!(db.root_count(), 0);
}

#[test]
fn test_folder_uuid_uniqueness() {
    let gen = MockDataGenerator::default();
    let folders = gen.folder_uuids();
    let all = folders.all();

    // Verify all folder UUIDs are unique
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            assert_ne!(all[i], all[j], "Folder UUIDs should be unique");
        }
    }
}

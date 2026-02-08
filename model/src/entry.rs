use std::sync::Arc;

use uuid::Uuid;
use dashmap::DashMap;
use edger_tree::prelude::Identifiable;

use crate::prelude::ProtoEntry;
use crate::tag::Weak as TagWeak;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entry {
    pub id: Uuid,
    pub tags: DashMap<Uuid, TagWeak>,
    pub aged: DashMap<Uuid, String>,
    pub memo: Option<String>,
}

impl Identifiable for Entry {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl Entry {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            tags: Default::default(),
            aged: Default::default(),
            memo: Default::default(),
        }
    }

    /// Convert a ProtoEntry to a model Entry
    /// Note: Tags are initially empty and must be populated later
    pub fn from_proto(proto: &ProtoEntry) -> Arc<Self> {
        let entry = Self {
            id: proto.id,
            tags: DashMap::new(),
            aged: proto.aged.iter().map(|(k, v)| (*k, v.clone())).collect(),
            memo: if proto.memo.is_empty() {
                None
            } else {
                Some(proto.memo.clone())
            },
        };
        Arc::new(entry)
    }

    /// Add a weak tag reference to this entry
    pub fn add_tag(&self, tag_id: Uuid, tag: TagWeak) {
        self.tags.insert(tag_id, tag);
    }
}
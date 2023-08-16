use std::sync::Weak;

use uuid::Uuid;
use dashmap::DashMap;
use edger_tree::prelude::Identifiable;

use crate::prelude::Tag;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entry {
    pub id: Uuid,
    pub tags: DashMap<Uuid, Weak<Tag>>,
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
}
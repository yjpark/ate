use indexmap::IndexMap;

use crate::prelude::{Uuid, LeafTag};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entry {
    pub id: Uuid,
    pub tags: IndexMap<Uuid, LeafTag>,
    pub aged: IndexMap<Uuid, String>,
    pub memo: String,
}
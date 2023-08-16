use std::sync::{Weak, Arc};

use uuid::Uuid;
use deref_derive::Deref;

use edger_tree::dashmap::DashTree;

use crate::prelude::{ProtoTag, Entry};

#[derive(Debug, Clone, Deref)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tag(DashTree<Uuid, Arc<ProtoTag>, Weak<Tag>, Arc<Tag>, Arc<Entry>>);

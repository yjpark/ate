use std::sync::{Weak as StdWeak, Arc as StdArc};

use uuid::Uuid;

use crate::prelude::{ProtoTag, Entry};

// Custom wrapper types for Arc and Weak to implement AsRef
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arc(StdArc<TagInner>);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Weak(StdWeak<TagInner>);

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TagInner {
    pub id: Uuid,
    pub data: StdArc<ProtoTag>,
    pub parent: Option<Weak>,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_dashmap", deserialize_with = "deserialize_dashmap"))]
    pub children: dashmap::DashMap<Uuid, Arc>,
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_dashmap_entries", deserialize_with = "deserialize_dashmap_entries"))]
    pub items: dashmap::DashMap<Uuid, StdArc<Entry>>,
}

#[cfg(feature = "serde")]
fn serialize_dashmap<S>(map: &dashmap::DashMap<Uuid, Arc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let vec: Vec<_> = map.iter().map(|r| (r.key().clone(), r.value().clone())).collect();
    vec.serialize(serializer)
}

#[cfg(feature = "serde")]
fn deserialize_dashmap<'de, D>(deserializer: D) -> Result<dashmap::DashMap<Uuid, Arc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let vec: Vec<(Uuid, Arc)> = Vec::deserialize(deserializer)?;
    let map = dashmap::DashMap::new();
    for (k, v) in vec {
        map.insert(k, v);
    }
    Ok(map)
}

#[cfg(feature = "serde")]
fn serialize_dashmap_entries<S>(map: &dashmap::DashMap<Uuid, StdArc<Entry>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::Serialize;
    let vec: Vec<_> = map.iter().map(|r| (r.key().clone(), r.value().clone())).collect();
    vec.serialize(serializer)
}

#[cfg(feature = "serde")]
fn deserialize_dashmap_entries<'de, D>(deserializer: D) -> Result<dashmap::DashMap<Uuid, StdArc<Entry>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let vec: Vec<(Uuid, StdArc<Entry>)> = Vec::deserialize(deserializer)?;
    let map = dashmap::DashMap::new();
    for (k, v) in vec {
        map.insert(k, v);
    }
    Ok(map)
}

pub type Tag = Arc;

impl AsRef<TagInner> for Arc {
    fn as_ref(&self) -> &TagInner {
        &self.0
    }
}

impl AsRef<TagInner> for Weak {
    fn as_ref(&self) -> &TagInner {
        unreachable!("Weak::as_ref should not be called")
    }
}

impl Arc {
    /// Create a new Tag from a ProtoTag
    pub fn new(proto: ProtoTag) -> Self {
        let tag_id = proto.id();
        Arc(StdArc::new(TagInner {
            id: tag_id,
            data: StdArc::new(proto),
            parent: None,
            children: Default::default(),
            items: Default::default(),
        }))
    }

    /// Add a child tag to this tag
    pub fn add_child(&self, child: Arc) {
        self.0.children.insert(child.0.id, child);
    }

    /// Add an entry to this tag's items
    pub fn add_entry(&self, entry: StdArc<Entry>) {
        self.0.items.insert(entry.id, entry);
    }

    /// Get all entries in this tag (non-recursive)
    pub fn entries(&self) -> Vec<StdArc<Entry>> {
        self.0.items.iter().map(|item| StdArc::clone(item.value())).collect()
    }

    /// Get all entries in this tag and its children (recursive)
    pub fn entries_recursive(&self) -> Vec<StdArc<Entry>> {
        let mut result = self.entries();
        for child in self.0.children.iter() {
            result.extend(child.value().entries_recursive());
        }
        result
    }

    /// Get the underlying ProtoTag
    pub fn proto(&self) -> &StdArc<ProtoTag> {
        &self.0.data
    }

    /// Get all child tags
    pub fn children(&self) -> Vec<Arc> {
        self.0.children.iter().map(|child| child.value().clone()).collect()
    }

    /// Downgrade to a weak reference
    pub fn downgrade(&self) -> Weak {
        Weak(StdArc::downgrade(&self.0))
    }
}

impl Weak {
    /// Upgrade to a strong reference
    pub fn upgrade(&self) -> Option<Arc> {
        self.0.upgrade().map(Arc)
    }
}

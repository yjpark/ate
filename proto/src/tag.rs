use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::prelude::{Uuid, Recipient};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tag {
    Group(GroupTag),
    Leaf(LeafTag),
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupTag {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub data: GroupData,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GroupData {
    Folder { name: String },
    Year { year: u16 },
    Month { year: u16, month: u8 },
    Day { year: u16, month: u8, day: u8 },
    Week { year: u16, week: u8 },
    Custom { kind: String, value: String },
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LeafTag {
    pub id: Uuid,
    pub data: LeafData,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LeafData {
    InFolder(Uuid),
    Recipient(Recipient),
    Added(DateTime<Utc>, Tz),
    Updated(DateTime<Utc>, Tz),
    Custom { kind: String, value: String },
}

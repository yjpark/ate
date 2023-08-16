pub use uuid;

pub mod age;
pub mod tag;
pub mod entry;

pub mod prelude {
    #[doc(hidden)]
    pub use uuid::Uuid;

    #[doc(hidden)]
    pub use crate::age::Recipient;

    #[doc(hidden)]
    pub use crate::tag::{Tag, GroupTag, GroupData, LeafTag, LeafData};

    #[doc(hidden)]
    pub use crate::entry::Entry;
}
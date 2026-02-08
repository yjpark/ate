pub use ate_proto;
pub use edger_tree;

pub mod tag;
pub mod entry;
pub mod loader;
pub mod converter;
pub mod database;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod lib_test;

pub mod prelude {
    pub use ate_proto::prelude::{*,
        Tag as ProtoTag,
        Entry as ProtoEntry,
    };

    #[doc(hidden)]
    pub use crate::tag::Tag;

    #[doc(hidden)]
    pub use crate::entry::Entry;

    #[doc(hidden)]
    pub use crate::database::Database;
}
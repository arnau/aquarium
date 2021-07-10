use anyhow::Result;

mod asset;
mod bulletin_entry;
mod bulletin_issue;
mod bulletin_mention;
mod note;
mod person;
mod project;
mod section;
mod service_account;
mod settings;
mod sketch;
mod sketch_tool;
mod tool;

pub use asset::*;
pub use bulletin_entry::*;
pub use bulletin_issue::*;
pub use bulletin_mention::*;
pub use note::*;
pub use person::*;
pub use project::*;
pub use section::*;
pub use service_account::*;
pub use settings::*;
pub use sketch::*;
pub use sketch_tool::*;

use super::Transaction;

/// A record able to operate on a SQLite storage.
pub trait Record: Sized {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>>;
    fn insert(&self, tx: &Transaction) -> Result<()>;
    fn delete(tx: &Transaction, id: &str) -> Result<()>;
}

/// A record set able to operate on a SQLite storage.
pub trait RecordSet: Sized {
    type Item: Record;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Query anything.
    fn query(tx: &Transaction, query: &str) -> Result<Self>;

    // TODO: Can I pass a predicate here?
    fn select(tx: &Transaction) -> Result<Self>;

    /// Inserts all records in the collection.
    fn insert(&self, tx: &Transaction) -> Result<()>;

    /// Deletes the whole set.
    fn delete(tx: &Transaction) -> Result<()>;
}

/// An auxiliary record able to operate on a SQLite storage.
pub trait AuxRecord: Sized {
    fn insert(&self, tx: &Transaction) -> Result<()>;
}

/// A record set for auxiliary data able to operate on a SQLite storage.
pub trait AuxRecordSet: Sized {
    type Item: AuxRecord;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn select(tx: &Transaction, id: &str) -> Result<Self>;
}

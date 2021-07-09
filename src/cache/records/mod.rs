use anyhow::Result;

mod note;

pub use note::*;

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

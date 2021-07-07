use anyhow::{self, Result};
use chrono::{DateTime, Utc};
pub use rusqlite::Transaction;
use rusqlite::{self, params, Connection};
use std::str::FromStr;

mod strategy;
pub use strategy::Strategy;

/// A Cache storage.
#[derive(Debug)]
pub struct Cache {
    pub timestamp: DateTime<Utc>,
    pub conn: Connection,
    pub strategy: Strategy,
}

impl Cache {
    pub fn connect_with_strategy(strategy: Strategy) -> Result<Cache> {
        let timestamp = Utc::now();
        let conn = match &strategy {
            Strategy::Disk(path) => {
                let conn = Connection::open(path)?;
                conn.pragma_update(None, "journal_mode", &"wal")?;
                conn.pragma_update(None, "foreign_keys", &"on")?;
                conn
            }
            Strategy::Memory => Connection::open_in_memory()?,
        };
        let bootstrap = include_str!("../sql/cache.sql");

        conn.execute_batch(&bootstrap)?;

        Ok(Cache {
            timestamp,
            conn,
            strategy,
        })
    }

    pub fn connect(path: &str) -> Result<Cache> {
        let strategy = Strategy::from_str(path)?;
        Self::connect_with_strategy(strategy)
    }

    pub fn disconnect(&self) -> Result<()> {
        if let Strategy::Disk(_) = self.strategy {
            self.conn
                .pragma_update(None, "wal_checkpoint", &"restart")?;
            self.conn.pragma_update(None, "journal_mode", &"delete")?;
        }

        Ok(())
    }

    pub fn transaction(&mut self) -> Result<Transaction> {
        let tx = self.conn.transaction()?;

        Ok(tx)
    }

    /// Remove all stale records for the given session.
    pub fn prune(&mut self) -> Result<()> {
        unimplemented!();
    }
}

pub trait ReadCache
where
    Self: Sized,
{
    type Item;

    /// Reads the cache to find a single item by Id.
    fn find(tx: &Transaction, id: &str) -> Option<Self::Item>;

    /// Reads the cache to get all items in the set.
    fn amass(tx: &Transaction) -> Vec<Self>;
}

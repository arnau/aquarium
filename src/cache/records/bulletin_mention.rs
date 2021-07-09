//! This module defines the bulletin mention and bulletin mention set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{AuxRecordSet, Record};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct BulletinMentionRecord {
    pub(crate) mention_url: String,
    pub(crate) entry_url: String,
}

impl Record for BulletinMentionRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    bulletin_mention
                WHERE
                    mention_url = ?;
            "#,
        )?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let record = Self::try_from(row)?;

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    fn insert(&self, tx: &Transaction) -> Result<()> {
        let values = params![&self.mention_url, &self.entry_url,];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                bulletin_mention
            VALUES
                (?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                bulletin_mention
            WHERE
                mention_url = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for BulletinMentionRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            mention_url: row.get(0)?,
            entry_url: row.get(1)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct BulletinMentionRecordSet {
    inner: Vec<BulletinMentionRecord>,
}

impl IntoIterator for BulletinMentionRecordSet {
    type Item = BulletinMentionRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl AuxRecordSet for BulletinMentionRecordSet {
    type Item = BulletinMentionRecord;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn select(tx: &Transaction, id: &str) -> Result<Self> {
        let mut inner = Vec::new();
        let mut stmt = tx.prepare(
            r#"
            SELECT
                *
            FROM
                bulletin_mention
            WHERE
                entry_url = ?;
            "#,
        )?;
        let mut rows = stmt.query(params![id])?;

        while let Some(row) = rows.next()? {
            let record = Self::Item::try_from(row)?;
            inner.push(record);
        }

        Ok(Self { inner })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;

    #[test]
    fn full_cycle() -> Result<()> {
        let record = BulletinMentionRecord {
            mention_url: "mention1".into(),
            entry_url: "entry1".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached =
            BulletinMentionRecord::select(&tx, &record.mention_url)?.expect("record to be cached");

        assert_eq!(record, cached);

        BulletinMentionRecord::delete(&tx, &record.mention_url)?;

        let void = BulletinMentionRecord::select(&tx, &record.mention_url)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = BulletinMentionRecord {
            mention_url: "mention1".into(),
            entry_url: "entry1".into(),
        };
        let record2 = BulletinMentionRecord {
            mention_url: "mention2".into(),
            entry_url: "entry1".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = BulletinMentionRecordSet::select(&tx, "entry1")?;

        assert_eq!(cached.len(), 2);

        tx.commit()?;

        Ok(())
    }
}

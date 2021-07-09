//! This module defines the asset and asset set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct AssetRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) content_type: String,
    pub(crate) content: Vec<u8>,
}

impl Record for AssetRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    asset
                WHERE
                    id = ?;
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
        let values = params![&self.id, &self.checksum, &self.content_type, &self.content,];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                asset
            VALUES
                (?, ?, ?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                asset
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for AssetRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            content_type: row.get(2)?,
            content: row.get(3)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct AssetRecordSet {
    inner: Vec<AssetRecord>,
}

impl IntoIterator for AssetRecordSet {
    type Item = AssetRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for AssetRecordSet {
    type Item = AssetRecord;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn query(tx: &Transaction, query: &str) -> Result<Self> {
        let mut inner = Vec::new();
        let mut stmt = tx.prepare(query)?;
        let mut rows = stmt.query(params![])?;

        while let Some(row) = rows.next()? {
            let record = Self::Item::try_from(row)?;
            inner.push(record);
        }

        Ok(Self { inner })
    }

    fn select(tx: &Transaction) -> Result<Self> {
        let mut inner = Vec::new();
        let mut stmt = tx.prepare(
            r#"
              SELECT
                  *
              FROM
                  asset;
            "#,
        )?;
        let mut rows = stmt.query(params![])?;

        while let Some(row) = rows.next()? {
            let record = Self::Item::try_from(row)?;
            inner.push(record);
        }

        Ok(Self { inner })
    }

    fn insert(&self, tx: &Transaction) -> Result<()> {
        for item in &self.inner {
            item.insert(tx)?;
        }

        Ok(())
    }

    fn delete(tx: &Transaction) -> Result<()> {
        let mut stmt = tx.prepare(r#"DELETE FROM asset;"#)?;
        stmt.execute(params![])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;

    #[test]
    fn full_cycle() -> Result<()> {
        let record = AssetRecord {
            id: "asset1".into(),
            checksum: "asset1".into(),
            content_type: "".into(),
            content: "".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = AssetRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        AssetRecord::delete(&tx, &record.id)?;

        let void = AssetRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = AssetRecord {
            id: "asset1".into(),
            checksum: "asset1".into(),
            content_type: "".into(),
            content: "".into(),
        };
        let record2 = AssetRecord {
            id: "asset2".into(),
            checksum: "asset2".into(),
            content_type: "".into(),
            content: "".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = AssetRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        AssetRecordSet::delete(&tx)?;

        let void = AssetRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

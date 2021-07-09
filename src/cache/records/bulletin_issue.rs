//! This module defines the bulletin and bulletin set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct BulletinRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) summary: String,
    pub(crate) publication_date: String,
}

impl Record for BulletinRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    bulletin_issue
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
        let values = params![
            &self.id,
            &self.checksum,
            &self.summary,
            &self.publication_date,
        ];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                bulletin_issue
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
                bulletin_issue
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for BulletinRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            summary: row.get(2)?,
            publication_date: row.get(3)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct BulletinRecordSet {
    inner: Vec<BulletinRecord>,
}

impl IntoIterator for BulletinRecordSet {
    type Item = BulletinRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for BulletinRecordSet {
    type Item = BulletinRecord;

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
                  bulletin_issue;
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
        let mut stmt = tx.prepare(r#"DELETE FROM bulletin_issue;"#)?;
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
        let record = BulletinRecord {
            id: "bulletin1".into(),
            checksum: "bulletin1".into(),
            summary: "".into(),
            publication_date: "2021-02-03".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = BulletinRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        BulletinRecord::delete(&tx, &record.id)?;

        let void = BulletinRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = BulletinRecord {
            id: "bulletin1".into(),
            checksum: "bulletin1".into(),
            summary: "".into(),
            publication_date: "2021-02-03".into(),
        };
        let record2 = BulletinRecord {
            id: "bulletin2".into(),
            checksum: "bulletin2".into(),
            summary: "".into(),
            publication_date: "2021-02-03".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = BulletinRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        BulletinRecordSet::delete(&tx)?;

        let void = BulletinRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

//! This module defines the bulletin entry and bulletin entry set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct BulletinEntryRecord {
    pub(crate) url: String,
    pub(crate) checksum: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) content_type: String,
    pub(crate) issue_id: Option<String>,
}

impl Record for BulletinEntryRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    bulletin_entry
                WHERE
                    url = ?;
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
            &self.url,
            &self.checksum,
            &self.title,
            &self.summary,
            &self.content_type,
            &self.issue_id,
        ];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                bulletin_entry
            VALUES
                (?, ?, ?, ?, ?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                bulletin_entry
            WHERE
                url = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for BulletinEntryRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            url: row.get(0)?,
            checksum: row.get(1)?,
            title: row.get(2)?,
            summary: row.get(3)?,
            content_type: row.get(4)?,
            issue_id: row.get(5)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct BulletinEntryRecordSet {
    inner: Vec<BulletinEntryRecord>,
}

impl IntoIterator for BulletinEntryRecordSet {
    type Item = BulletinEntryRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for BulletinEntryRecordSet {
    type Item = BulletinEntryRecord;

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
                  bulletin_entry;
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
        let mut stmt = tx.prepare(r#"DELETE FROM bulletin_entry;"#)?;
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
        let record = BulletinEntryRecord {
            url: "entry1".into(),
            checksum: "entry1".into(),
            title: "".into(),
            summary: "".into(),
            content_type: "".into(),
            issue_id: None,
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = BulletinEntryRecord::select(&tx, &record.url)?.expect("record to be cached");

        assert_eq!(record, cached);

        BulletinEntryRecord::delete(&tx, &record.url)?;

        let void = BulletinEntryRecord::select(&tx, &record.url)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = BulletinEntryRecord {
            url: "entry1".into(),
            checksum: "entry1".into(),
            title: "".into(),
            summary: "".into(),
            content_type: "".into(),
            issue_id: None,
        };
        let record2 = BulletinEntryRecord {
            url: "entry2".into(),
            checksum: "entry2".into(),
            title: "".into(),
            summary: "".into(),
            content_type: "".into(),
            issue_id: Some("bulletin1".into()),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = BulletinEntryRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        BulletinEntryRecordSet::delete(&tx)?;

        let void = BulletinEntryRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

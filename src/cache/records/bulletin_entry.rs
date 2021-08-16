//! This module defines the bulletin entry and bulletin entry set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{AuxRecord, AuxRecordSet};
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

impl BulletinEntryRecord {
    pub fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
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
}

impl AuxRecord for BulletinEntryRecord {
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

impl AuxRecordSet for BulletinEntryRecordSet {
    type Item = BulletinEntryRecord;
    type ResourceId = Option<String>;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn select(tx: &Transaction, id: Self::ResourceId) -> Result<Self> {
        let clause = if let Some(id) = id {
            format!("issue_id = '{}'", id)
        } else {
            "issue_id is NULL".to_string()
        };
        let query = format!(
            r#"
            SELECT
                *
            FROM
                bulletin_entry
            WHERE
                {};
            "#,
            clause
        );

        let mut inner = Vec::new();
        let mut stmt = tx.prepare(&query)?;
        let mut rows = stmt.query(params![])?;

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

        tx.commit()?;

        assert_eq!(record, cached);

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

        let cached = BulletinEntryRecordSet::select(&tx, None)?;

        tx.commit()?;

        assert_eq!(cached.len(), 1);

        Ok(())
    }
}

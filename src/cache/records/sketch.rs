//! This module defines the sketch and sketch set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct SketchRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) title: String,
    pub(crate) asset_id: String,
    pub(crate) author_id: String,
    pub(crate) publication_date: String,
    pub(crate) summary: Option<String>,
}

impl Record for SketchRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    sketch
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
            &self.title,
            &self.asset_id,
            &self.author_id,
            &self.publication_date,
            &self.summary,
        ];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                sketch
            VALUES
                (?, ?, ?, ?, ?, ?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                sketch
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for SketchRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            title: row.get(2)?,
            asset_id: row.get(3)?,
            author_id: row.get(4)?,
            publication_date: row.get(5)?,
            summary: row.get(6)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct SketchRecordSet {
    inner: Vec<SketchRecord>,
}

impl IntoIterator for SketchRecordSet {
    type Item = SketchRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for SketchRecordSet {
    type Item = SketchRecord;

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
                  sketch;
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
        let mut stmt = tx.prepare(r#"DELETE FROM sketch;"#)?;
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
        let record = SketchRecord {
            id: "sketch1".into(),
            checksum: "sketch1".into(),
            title: "".into(),
            asset_id: "".into(),
            author_id: "".into(),
            publication_date: "2021-02-03".into(),
            summary: None,
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = SketchRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        SketchRecord::delete(&tx, &record.id)?;

        let void = SketchRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = SketchRecord {
            id: "sketch1".into(),
            checksum: "sketch1".into(),
            title: "".into(),
            asset_id: "".into(),
            author_id: "".into(),
            publication_date: "2021-02-03".into(),
            summary: None,
        };
        let record2 = SketchRecord {
            id: "sketch2".into(),
            checksum: "sketch2".into(),
            title: "".into(),
            asset_id: "".into(),
            author_id: "".into(),
            publication_date: "2021-02-03".into(),
            summary: Some("".into()),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = SketchRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        SketchRecordSet::delete(&tx)?;

        let void = SketchRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

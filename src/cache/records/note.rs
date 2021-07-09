//! This module defines the note and note set records for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct NoteRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) publication_date: String,
    pub(crate) author_id: String,
    pub(crate) body: String,
}

impl Record for NoteRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<NoteRecord>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    note
                WHERE
                    id = ?;
            "#,
        )?;
        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let record = NoteRecord::try_from(row)?;

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
            &self.summary,
            &self.publication_date,
            &self.author_id,
            &self.body,
        ];
        let mut stmt = tx.prepare(
            r#"
              INSERT OR REPLACE INTO
                note
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
                note
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for NoteRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<NoteRecord> {
        let record = NoteRecord {
            id: row.get(0)?,
            checksum: row.get(1)?,
            title: row.get(2)?,
            summary: row.get(3)?,
            publication_date: row.get(4)?,
            author_id: row.get(5)?,
            body: row.get(6)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct NoteRecordSet {
    inner: Vec<NoteRecord>,
}

impl IntoIterator for NoteRecordSet {
    type Item = NoteRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for NoteRecordSet {
    type Item = NoteRecord;

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
                  note;
            "#,
        )?;
        let mut rows = stmt.query(params![])?;

        while let Some(row) = rows.next()? {
            let record = NoteRecord::try_from(row)?;
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
        let mut stmt = tx.prepare(r#"DELETE FROM note;"#)?;
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
        let record = NoteRecord {
            id: "a-note".into(),
            checksum: "fake".into(),
            title: "A simple note".into(),
            summary: "Lorem ipsum".into(),
            publication_date: "2021-07-09".into(),
            author_id: "bobsponge".into(),
            body: "A long note that _turns out_ to be…\n\n**quite short!**.".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = NoteRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        NoteRecord::delete(&tx, &record.id)?;

        let void = NoteRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = NoteRecord {
            id: "note1".into(),
            checksum: "note1".into(),
            title: "".into(),
            summary: "".into(),
            publication_date: "2021-07-09".into(),
            author_id: "bobsponge".into(),
            body: "".into(),
        };
        let record2 = NoteRecord {
            id: "note2".into(),
            checksum: "note2".into(),
            title: "".into(),
            summary: "".into(),
            publication_date: "2021-07-09".into(),
            author_id: "bobsponge".into(),
            body: "".into(),
        };

        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = NoteRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        NoteRecordSet::delete(&tx)?;

        let void = NoteRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

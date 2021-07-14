//! This module defines the section and section set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct SectionRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) title: String,
    pub(crate) resource_type: Option<String>,
    pub(crate) body: Option<String>,
}

impl Record for SectionRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    section
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
            &self.resource_type,
            &self.body,
        ];
        let mut stmt = tx.prepare(
            r#"
              INSERT OR REPLACE INTO
                section
              VALUES
                (?, ?, ?, ?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                section
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for SectionRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            title: row.get(2)?,
            resource_type: row.get(3)?,
            body: row.get(4)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct SectionRecordSet {
    inner: Vec<SectionRecord>,
}

impl IntoIterator for SectionRecordSet {
    type Item = SectionRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for SectionRecordSet {
    type Item = SectionRecord;

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
                  section;
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
        let mut stmt = tx.prepare(r#"DELETE FROM section;"#)?;
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
        let record = SectionRecord {
            id: "section1".into(),
            checksum: "section1".into(),
            title: "".into(),
            resource_type: None,
            body: None,
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = SectionRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        SectionRecord::delete(&tx, &record.id)?;

        let void = SectionRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = SectionRecord {
            id: "section1".into(),
            checksum: "section1".into(),
            title: "".into(),
            resource_type: None,
            body: Some("".into()),
        };
        let record2 = SectionRecord {
            id: "section2".into(),
            checksum: "section2".into(),
            title: "".into(),
            resource_type: Some("note".into()),
            body: None,
        };

        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = SectionRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        SectionRecordSet::delete(&tx)?;

        let void = SectionRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

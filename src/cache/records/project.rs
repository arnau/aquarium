//! This module defines the project and project set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) name: String,
    pub(crate) summary: String,
    pub(crate) body: String,
    pub(crate) status: String,
    pub(crate) start_date: String,
    pub(crate) end_date: Option<String>,
    pub(crate) source_url: Option<String>,
}

impl Record for ProjectRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    project
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
            &self.name,
            &self.summary,
            &self.body,
            &self.status,
            &self.start_date,
            &self.end_date,
            &self.source_url,
        ];
        let mut stmt = tx.prepare(
            r#"
              INSERT OR REPLACE INTO
                project
              VALUES
                (?, ?, ?, ?, ?, ?, ?, ?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                project
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for ProjectRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            name: row.get(2)?,
            summary: row.get(3)?,
            body: row.get(4)?,
            status: row.get(5)?,
            start_date: row.get(6)?,
            end_date: row.get(7)?,
            source_url: row.get(8)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct ProjectRecordSet {
    inner: Vec<ProjectRecord>,
}

impl IntoIterator for ProjectRecordSet {
    type Item = ProjectRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for ProjectRecordSet {
    type Item = ProjectRecord;

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
                  project;
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
        let mut stmt = tx.prepare(r#"DELETE FROM project;"#)?;
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
        let record = ProjectRecord {
            id: "project1".into(),
            checksum: "project1".into(),
            name: "".into(),
            summary: "".into(),
            body: "".into(),
            status: "ongoing".into(),
            start_date: "2021-07-09".into(),
            end_date: None,
            source_url: Some("https://foo.bar".into()),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = ProjectRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        ProjectRecord::delete(&tx, &record.id)?;

        let void = ProjectRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = ProjectRecord {
            id: "project1".into(),
            checksum: "project1".into(),
            name: "".into(),
            summary: "".into(),
            body: "".into(),
            status: "ongoing".into(),
            start_date: "2021-07-09".into(),
            end_date: None,
            source_url: None,
        };
        let record2 = ProjectRecord {
            id: "project2".into(),
            checksum: "project2".into(),
            name: "".into(),
            summary: "".into(),
            body: "".into(),
            status: "ended".into(),
            start_date: "2020-01-01".into(),
            end_date: Some("2021-02-03".into()),
            source_url: None,
        };

        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = ProjectRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        ProjectRecordSet::delete(&tx)?;

        let void = ProjectRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

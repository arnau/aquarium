//! This module defines the tool and tool set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct ToolRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) name: String,
    pub(crate) summary: Option<String>,
    pub(crate) url: Option<String>,
}

impl Record for ToolRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    tool
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
            &self.url,
        ];
        let mut stmt = tx.prepare(
            r#"
              INSERT OR REPLACE INTO
                tool
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
                tool
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for ToolRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            name: row.get(2)?,
            summary: row.get(3)?,
            url: row.get(4)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct ToolRecordSet {
    inner: Vec<ToolRecord>,
}

impl IntoIterator for ToolRecordSet {
    type Item = ToolRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for ToolRecordSet {
    type Item = ToolRecord;

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
                  tool;
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
        let mut stmt = tx.prepare(r#"DELETE FROM tool;"#)?;
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
        let record = ToolRecord {
            id: "tool1".into(),
            checksum: "tool1".into(),
            name: "".into(),
            summary: Some("".into()),
            url: Some("https://foo.bar".into()),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = ToolRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        ToolRecord::delete(&tx, &record.id)?;

        let void = ToolRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = ToolRecord {
            id: "tool1".into(),
            checksum: "tool1".into(),
            name: "".into(),
            summary: Some("".into()),
            url: None,
        };
        let record2 = ToolRecord {
            id: "tool2".into(),
            checksum: "tool2".into(),
            name: "".into(),
            summary: None,
            url: None,
        };

        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = ToolRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        ToolRecordSet::delete(&tx)?;

        let void = ToolRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

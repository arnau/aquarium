//! This module defines the update record for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct UpdateRecord {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) summary: Option<String>,
    pub(crate) section: String,
    pub(crate) date: String,
}

impl TryFrom<&Row<'_>> for UpdateRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<UpdateRecord> {
        let record = UpdateRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            section: row.get(3)?,
            date: row.get(4)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct UpdateRecordSet {
    inner: Vec<UpdateRecord>,
}

impl IntoIterator for UpdateRecordSet {
    type Item = UpdateRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl UpdateRecordSet {
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn select(tx: &Transaction) -> Result<Self> {
        let mut inner = Vec::new();
        let mut stmt = tx.prepare(
            r#"
            SELECT
                *
            FROM
                news;
            "#,
        )?;
        let mut rows = stmt.query(params![])?;

        while let Some(row) = rows.next()? {
            let record = UpdateRecord::try_from(row)?;
            inner.push(record);
        }

        Ok(Self { inner })
    }
}

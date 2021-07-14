//! This module defines the entrance for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::Record;
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct EntranceRecord {
    pub(crate) id: String,
    pub(crate) checksum: String,
    pub(crate) body: Option<String>,
}

impl Record for EntranceRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
            SELECT
                *
            FROM
                entrance
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
        let values = params![&self.id, &self.checksum, &self.body,];
        let mut stmt = tx.prepare(
            r#"
              INSERT OR REPLACE INTO
                entrance
              VALUES
                (?, ?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }

    fn delete(tx: &Transaction, id: &str) -> Result<()> {
        let mut stmt = tx.prepare(
            r#"
            DELETE FROM
                entrance
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for EntranceRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            checksum: row.get(1)?,
            body: row.get(2)?,
        };

        Ok(record)
    }
}

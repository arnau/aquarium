//! This module defines the sketch tool for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{AuxRecord, AuxRecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct SketchToolRecord {
    pub(crate) sketch_id: String,
    pub(crate) tool_id: String,
}

impl AuxRecord for SketchToolRecord {
    fn insert(&self, tx: &Transaction) -> Result<()> {
        let values = params![&self.sketch_id, &self.tool_id];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                sketch_tool
            VALUES
                (?, ?);
            "#,
        )?;

        stmt.execute(values)?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for SketchToolRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            sketch_id: row.get(0)?,
            tool_id: row.get(1)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct SketchToolRecordSet {
    inner: Vec<SketchToolRecord>,
}

impl IntoIterator for SketchToolRecordSet {
    type Item = SketchToolRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl AuxRecordSet for SketchToolRecordSet {
    type Item = SketchToolRecord;
    type ResourceId = String;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn select(tx: &Transaction, id: Self::ResourceId) -> Result<Self> {
        let mut inner = Vec::new();
        let mut stmt = tx.prepare(
            r#"
            SELECT
                *
            FROM
                sketch_tool
            WHERE
                sketch_id = ?;
            "#,
        )?;
        let mut rows = stmt.query(params![id])?;

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
    fn set_full_cycle() -> Result<()> {
        let record1 = SketchToolRecord {
            sketch_id: "sketch1".into(),
            tool_id: "tool1".into(),
        };
        let record2 = SketchToolRecord {
            sketch_id: "sketch1".into(),
            tool_id: "tool2".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = SketchToolRecordSet::select(&tx, "sketch1".to_string())?;

        assert_eq!(cached.len(), 2);

        tx.commit()?;

        Ok(())
    }
}

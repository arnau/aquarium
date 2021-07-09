//! This module defines the service_account and service_account set for the SQLite storage.

use anyhow::Result;
use std::convert::TryFrom;

use super::{Record, RecordSet};
use crate::cache::{params, Row, Transaction};

#[derive(Clone, Debug, PartialEq)]
pub struct ServiceAccountRecord {
    pub(crate) id: String,
    pub(crate) person_id: String,
    pub(crate) checksum: String,
    pub(crate) name: String,
    pub(crate) username: String,
    pub(crate) url: String,
}

impl Record for ServiceAccountRecord {
    fn select(tx: &Transaction, id: &str) -> Result<Option<Self>> {
        let mut stmt = tx.prepare(
            r#"
                SELECT
                    *
                FROM
                    service_account
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
            &self.person_id,
            &self.checksum,
            &self.name,
            &self.username,
            &self.url,
        ];
        let mut stmt = tx.prepare(
            r#"
            INSERT OR REPLACE INTO
                service_account
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
                service_account
            WHERE
                id = ?;
            "#,
        )?;

        stmt.execute(params![id])?;

        Ok(())
    }
}

impl TryFrom<&Row<'_>> for ServiceAccountRecord {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let record = Self {
            id: row.get(0)?,
            person_id: row.get(1)?,
            checksum: row.get(2)?,
            name: row.get(3)?,
            username: row.get(4)?,
            url: row.get(5)?,
        };

        Ok(record)
    }
}

#[derive(Clone, Debug)]
pub struct ServiceAccountRecordSet {
    inner: Vec<ServiceAccountRecord>,
}

impl IntoIterator for ServiceAccountRecordSet {
    type Item = ServiceAccountRecord;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl RecordSet for ServiceAccountRecordSet {
    type Item = ServiceAccountRecord;

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
                  service_account;
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
        let mut stmt = tx.prepare(r#"DELETE FROM service_account;"#)?;
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
        let record = ServiceAccountRecord {
            id: "account1".into(),
            person_id: "person1".into(),
            checksum: "account1".into(),
            name: "".into(),
            username: "".into(),
            url: "".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record.insert(&tx)?;

        let cached = ServiceAccountRecord::select(&tx, &record.id)?.expect("record to be cached");

        assert_eq!(record, cached);

        ServiceAccountRecord::delete(&tx, &record.id)?;

        let void = ServiceAccountRecord::select(&tx, &record.id)?;

        assert!(void.is_none());

        tx.commit()?;

        Ok(())
    }

    #[test]
    fn set_full_cycle() -> Result<()> {
        let record1 = ServiceAccountRecord {
            id: "account1".into(),
            person_id: "person1".into(),
            checksum: "account1".into(),
            name: "".into(),
            username: "".into(),
            url: "".into(),
        };
        let record2 = ServiceAccountRecord {
            id: "account2".into(),
            person_id: "person1".into(),
            checksum: "account2".into(),
            name: "".into(),
            username: "".into(),
            url: "".into(),
        };
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;

        record1.insert(&tx)?;
        record2.insert(&tx)?;

        let cached = ServiceAccountRecordSet::select(&tx)?;

        assert_eq!(cached.len(), 2);

        ServiceAccountRecordSet::delete(&tx)?;

        let void = ServiceAccountRecordSet::select(&tx)?;

        assert!(void.is_empty());

        tx.commit()?;

        Ok(())
    }
}

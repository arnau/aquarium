//! This module defines bulletin stash for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::BulletinEntry;
use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::Resource;

/// A bulletin stash resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BulletinStash {
    #[serde(rename = "type")]
    _type: String,
    entries: Vec<BulletinEntry>,
}

impl Resource for BulletinStash {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self._type
    }
}

impl Digest for BulletinStash {
    fn digest(&self, hasher: &mut Hasher) {
        self._type.digest(hasher);
        self.entries.digest(hasher);
    }
}

impl FromStr for BulletinStash {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let resource: BulletinStash = toml::from_str(&blob)?;

        Ok(resource)
    }
}

impl fmt::Display for BulletinStash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string(&self).expect("resource to serialise as TOML");

        write!(f, "{}", &toml)
    }
}

impl ReadCache for BulletinStash {
    type Item = BulletinStash;

    fn find(_tx: &Transaction, _id: &str) -> Result<Option<Self::Item>> {
        unimplemented!()
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let entries: Vec<BulletinEntry> = BulletinEntryRecordSet::select(tx, None)?
            .into_iter()
            .map(|account| BulletinEntry::from(account))
            .collect();
        let resource = Self {
            _type: "bulletin_stash".to_string(),
            entries,
        };

        Ok(resource)
    }
}

impl WriteCache for BulletinStash {
    type Item = BulletinStash;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        for entry in &resource.entries {
            let record = BulletinEntryRecord::from((None, entry));
            record.insert(&tx)?;
        }

        Ok(())
    }

    fn remove(_tx: &Transaction, _id: &str) -> Result<()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;

    #[test]
    fn full_cycle() -> Result<()> {
        let raw = r#"type = "bulletin_stash"

[[entries]]
url = "https://foo.bar"
title = "Foo"
summary = "Lorem Ipsum"
content_type = "text"

[[entries]]
url = "https://test.dev"
title = "test dev"
summary = "Test"
content_type = "text"
"#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = BulletinStash::from_str(raw)?;

        BulletinStash::add(&tx, resource.clone())?;

        let cached = BulletinStash::amass(&tx)?;

        tx.commit()?;

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        Ok(())
    }
}

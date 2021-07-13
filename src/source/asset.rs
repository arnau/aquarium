//! This module defines the asset for the Source stage.

use anyhow::Result;
use std::convert::TryFrom;
use std::iter::FromIterator;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::{Resource, ResourceSet};

/// An asset resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    id: String,
    content_type: String,
    content: Vec<u8>,
}

impl Resource for Asset {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Asset {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.content_type.digest(hasher);
        self.content.digest(hasher);
    }
}

impl Asset {
    pub fn new(id: String, content_type: String, content: Vec<u8>) -> Self {
        Self {
            id,
            content_type,
            content,
        }
    }
}

impl From<Asset> for AssetRecord {
    fn from(resource: Asset) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            content_type: resource.content_type,
            content: resource.content,
        }
    }
}

impl From<AssetRecord> for Asset {
    fn from(record: AssetRecord) -> Self {
        Self {
            id: record.id,
            content_type: record.content_type,
            content: record.content,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssetSet {
    inner: Vec<Asset>,
}

impl AssetSet {
    pub fn new(inner: Vec<Asset>) -> Self {
        Self { inner }
    }
}

impl ReadCache for AssetSet {
    type Item = Asset;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = AssetRecord::select(tx, id)? {
            let resource = Self::Item::from(record);

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = AssetRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(Self::Item::from)
            .collect::<Vec<_>>();

        Ok(Self::new(resources))
    }
}

impl WriteCache for AssetSet {
    type Item = Asset;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = AssetRecord::try_from(resource)?;
        record.insert(&tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        AssetRecord::delete(&tx, id)
    }
}

impl ResourceSet for AssetSet {}

impl IntoIterator for AssetSet {
    type Item = Asset;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Asset> for AssetSet {
    fn from_iter<I: IntoIterator<Item = Asset>>(iter: I) -> Self {
        let mut v = Vec::new();

        for item in iter {
            v.push(item);
        }

        Self::new(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::Cache;

    #[test]
    fn full_cycle() -> Result<()> {
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Asset::new(
            "foo.jpg".to_string(),
            "jpg".to_string(),
            vec![0, 1, 2, 1, 0],
        );

        AssetSet::add(&tx, resource.clone())?;

        let cached = AssetSet::find(&tx, &resource.id)?.expect("resource to be cached");

        assert_eq!(cached, resource);

        tx.commit()?;

        Ok(())
    }
}

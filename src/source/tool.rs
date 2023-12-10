//! This module defines the tool for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::markdown::take_frontmatter;
use crate::{Resource, ResourceSet};

/// A tool resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    id: String,
    name: String,
    summary: Option<String>,
    url: Option<String>,
}

impl Resource for Tool {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Tool {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.name.digest(hasher);
        self.summary.digest(hasher);
        self.url.digest(hasher);
    }
}

impl FromStr for Tool {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let (frontmatter, body) = take_frontmatter(blob)?;
        let metadata: Metadata = serde_yaml::from_str(frontmatter)?;
        let summary = if body.trim().is_empty() {
            None
        } else {
            Some(body.trim().to_string())
        };

        Ok(Self {
            id: metadata.id,
            name: metadata.name,
            summary,
            url: metadata.url,
        })
    }
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = Metadata::from(self);
        let yaml = serde_yaml::to_string(&metadata).expect("metadata to encode as yaml");

        writeln!(f, "---")?;
        write!(f, "{}", yaml)?;
        writeln!(f, "---")?;
        if let Some(summary) = &self.summary {
            write!(f, "{}", summary)?;
        }
        write!(f, "")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Metadata {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

impl From<&Tool> for Metadata {
    fn from(resource: &Tool) -> Self {
        Self {
            _type: "tool".into(),
            id: resource.id.clone(),
            name: resource.name.clone(),
            url: resource.url.clone(),
        }
    }
}

impl From<Tool> for ToolRecord {
    fn from(resource: Tool) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            name: resource.name,
            summary: resource.summary,
            url: resource.url,
        }
    }
}

impl From<ToolRecord> for Tool {
    fn from(record: ToolRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            summary: record.summary,
            url: record.url,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolSet {
    inner: Vec<Tool>,
}

impl ToolSet {
    pub fn new(inner: Vec<Tool>) -> Self {
        Self { inner }
    }
}

impl ReadCache for ToolSet {
    type Item = Tool;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = ToolRecord::select(tx, id)? {
            let resource = Self::Item::from(record);

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = ToolRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(Self::Item::from)
            .collect::<Vec<_>>();

        Ok(Self::new(resources))
    }
}

impl WriteCache for ToolSet {
    type Item = Tool;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = ToolRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        ToolRecord::delete(tx, id)
    }
}

impl ResourceSet for ToolSet {}

impl IntoIterator for ToolSet {
    type Item = Tool;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Tool> for ToolSet {
    fn from_iter<I: IntoIterator<Item = Tool>>(iter: I) -> Self {
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
        let raw = r#"---
type: tool
id: acme
name: Acme
url: https://acme.test/
---
This is a dummy tool."#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Tool::from_str(raw)?;

        ToolSet::add(&tx, resource.clone())?;

        let cached = ToolSet::find(&tx, &resource.id)?.expect("resource to be cached");

        tx.commit()?;

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        Ok(())
    }
}

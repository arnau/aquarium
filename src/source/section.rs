//! This module defines the section for the Source stage.

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

/// A note resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    id: String,
    title: String,
    resource_type: Option<String>,
    body: Option<String>,
}

impl Resource for Section {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Section {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.title.digest(hasher);
        self.resource_type.digest(hasher);
        self.body.digest(hasher);
    }
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let (frontmatter, body) = take_frontmatter(blob)?;
        let metadata: Metadata = serde_yaml::from_str(frontmatter)?;
        let body = if body.trim().is_empty() {
            None
        } else {
            Some(body.trim().to_string())
        };

        Ok(Self {
            id: metadata.id,
            title: metadata.title,
            resource_type: metadata.resource_type,
            body,
        })
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = Metadata::from(self);
        let yaml = serde_yaml::to_string(&metadata).expect("metadata to encode as yaml");

        writeln!(f, "---")?;
        write!(f, "{}", yaml)?;
        writeln!(f, "---")?;
        if let Some(body) = self.body.as_ref() {
            write!(f, "{}", body)?;
        }

        write!(f, "")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Metadata {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    resource_type: Option<String>,
}

impl From<&Section> for Metadata {
    fn from(resource: &Section) -> Metadata {
        Metadata {
            _type: "section".into(),
            id: resource.id.clone(),
            title: resource.title.clone(),
            resource_type: resource.resource_type.clone(),
        }
    }
}

impl From<Section> for SectionRecord {
    fn from(resource: Section) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            title: resource.title,
            resource_type: resource.resource_type,
            body: resource.body,
        }
    }
}

impl From<SectionRecord> for Section {
    fn from(record: SectionRecord) -> Self {
        Self {
            id: record.id,
            title: record.title,
            resource_type: record.resource_type,
            body: record.body,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SectionSet {
    inner: Vec<Section>,
}

impl SectionSet {
    pub fn new(inner: Vec<Section>) -> Self {
        Self { inner }
    }
}

impl ReadCache for SectionSet {
    type Item = Section;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = SectionRecord::select(tx, id)? {
            let resource = Self::Item::from(record);

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = SectionRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(Self::Item::from)
            .collect::<Vec<_>>();

        Ok(Self::new(resources))
    }
}

impl WriteCache for SectionSet {
    type Item = Section;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = SectionRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        SectionRecord::delete(tx, id)
    }
}

impl ResourceSet for SectionSet {}

impl IntoIterator for SectionSet {
    type Item = Section;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Section> for SectionSet {
    fn from_iter<I: IntoIterator<Item = Section>>(iter: I) -> Self {
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
type: section
id: notes
title: Notes
resource_type: note
---
Notes, reflections and reviews."#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Section::from_str(raw)?;

        SectionSet::add(&tx, resource.clone())?;

        let cached = SectionSet::find(&tx, &resource.id)?.expect("resource to be cached");

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        tx.commit()?;

        Ok(())
    }
}

//! This module defines the project for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::markdown::Markdown;
use crate::stamp::Date;
use crate::{Resource, ResourceSet};

/// A project resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    id: String,
    name: String,
    summary: String,
    body: String,
    status: String,
    start_date: Date,
    end_date: Option<Date>,
    source_url: Option<String>,
}

impl Resource for Project {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Project {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.name.digest(hasher);
        self.summary.digest(hasher);
        self.body.digest(hasher);
        self.status.digest(hasher);
        self.start_date.digest(hasher);
        self.end_date.digest(hasher);
        self.source_url.digest(hasher);
    }
}

impl FromStr for Project {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let Markdown {
            frontmatter,
            title,
            summary,
            body,
        } = Markdown::from_str(blob)?;
        let metadata: Metadata = serde_yaml::from_str(&frontmatter)?;
        let end_date = if let Some(date) = metadata.end_date {
            Some(Date::from_str(&date)?)
        } else {
            None
        };

        Ok(Self {
            id: metadata.id,
            name: title,
            summary: summary.expect("projects must have a summary"),
            body,
            status: metadata.status,
            start_date: Date::from_str(&metadata.start_date)?,
            end_date,
            source_url: metadata.source_url,
        })
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = Metadata::from(self);
        let yaml = serde_yaml::to_string(&metadata).expect("metadata to encode as yaml");

        write!(f, "{}", yaml)?;
        writeln!(f, "---")?;
        writeln!(f, "# {}", &self.name)?;
        writeln!(f, "\n{}\n", &self.summary)?;
        writeln!(f, "<!-- body -->\n")?;
        write!(f, "{}", &self.body)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Metadata {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    status: String,
    start_date: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    end_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_url: Option<String>,
}

impl From<&Project> for Metadata {
    fn from(resource: &Project) -> Metadata {
        Metadata {
            _type: "project".into(),
            id: resource.id.clone(),
            status: resource.status.clone(),
            start_date: resource.start_date.to_string(),
            end_date: resource.end_date.map(|x| x.to_string()),
            source_url: resource.source_url.clone(),
        }
    }
}

impl From<Project> for ProjectRecord {
    fn from(resource: Project) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            name: resource.name,
            summary: resource.summary,
            status: resource.status,
            start_date: resource.start_date.to_string(),
            end_date: resource.end_date.map(|s| s.to_string()),
            source_url: resource.source_url,
            body: resource.body,
        }
    }
}

impl TryFrom<ProjectRecord> for Project {
    type Error = anyhow::Error;

    fn try_from(record: ProjectRecord) -> Result<Self> {
        let end_date = if let Some(date) = record.end_date {
            Some(Date::from_str(&date)?)
        } else {
            None
        };
        let resource = Self {
            id: record.id,
            name: record.name,
            summary: record.summary,
            status: record.status,
            start_date: Date::from_str(&record.start_date)?,
            end_date,
            source_url: record.source_url,
            body: record.body,
        };

        Ok(resource)
    }
}

#[derive(Clone, Debug)]
pub struct ProjectSet {
    inner: Vec<Project>,
}

impl ProjectSet {
    pub fn new(inner: Vec<Project>) -> Self {
        Self { inner }
    }
}

impl ReadCache for ProjectSet {
    type Item = Project;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = ProjectRecord::select(tx, id)? {
            let resource = Self::Item::try_from(record)?;

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = ProjectRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(Self::Item::try_from)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(resources))
    }
}

impl WriteCache for ProjectSet {
    type Item = Project;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = ProjectRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        ProjectRecord::delete(tx, id)
    }
}

impl ResourceSet for ProjectSet {}

impl IntoIterator for ProjectSet {
    type Item = Project;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Project> for ProjectSet {
    fn from_iter<I: IntoIterator<Item = Project>>(iter: I) -> Self {
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
type: project
id: acme
status: ongoing
start_date: 2021-07-07
---
# Acme

This is a dummy project.

<!-- body -->

This text illustrates the shape of a project."#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Project::from_str(raw)?;

        ProjectSet::add(&tx, resource.clone())?;

        let cached = ProjectSet::find(&tx, &resource.id)?.expect("resource to be cached");

        tx.commit()?;

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        Ok(())
    }
}

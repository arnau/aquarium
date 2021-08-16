//! This module defines the sketch for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::stamp::Date;
use crate::{Resource, ResourceSet};

/// A sketch resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sketch {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    title: String,
    asset: String,
    author: String,
    publication_date: Date,
    tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

impl Resource for Sketch {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Sketch {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.title.digest(hasher);
        self.asset.digest(hasher);
        self.author.digest(hasher);
        self.publication_date.digest(hasher);
        self.summary.digest(hasher);
        self.tools.digest(hasher);
    }
}

impl FromStr for Sketch {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let resource: Sketch = toml::from_str(blob)?;

        Ok(resource)
    }
}

impl fmt::Display for Sketch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string(&self).expect("resource to serialise as TOML");

        write!(f, "{}", &toml)
    }
}

impl From<Sketch> for SketchRecord {
    fn from(resource: Sketch) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            title: resource.title,
            asset_id: resource.asset,
            author_id: resource.author,
            publication_date: resource.publication_date.to_string(),
            summary: resource.summary,
        }
    }
}

fn from_record(tx: &Transaction, record: SketchRecord) -> Result<Sketch> {
    let tools: Vec<String> = SketchToolRecordSet::select(tx, record.id.clone())?
        .into_iter()
        .map(|record| record.tool_id)
        .collect();

    let resource = Sketch {
        _type: "sketch".to_string(),
        id: record.id,
        title: record.title,
        asset: record.asset_id,
        author: record.author_id,
        publication_date: Date::from_str(&record.publication_date)?,
        summary: record.summary,
        tools,
    };

    Ok(resource)
}

#[derive(Clone, Debug)]
pub struct SketchSet {
    inner: Vec<Sketch>,
}

impl SketchSet {
    pub fn new(inner: Vec<Sketch>) -> Self {
        Self { inner }
    }
}

impl ReadCache for SketchSet {
    type Item = Sketch;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = SketchRecord::select(tx, id)? {
            let resource = from_record(tx, record)?;

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = SketchRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(|record| from_record(tx, record))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(resources))
    }
}

impl WriteCache for SketchSet {
    type Item = Sketch;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        for tool_id in &resource.tools {
            let record = SketchToolRecord {
                tool_id: tool_id.clone(),
                sketch_id: resource.id.clone(),
            };

            record.insert(tx)?;
        }

        let record = SketchRecord::try_from(resource)?;
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        SketchRecord::delete(tx, id)
    }
}

impl ResourceSet for SketchSet {}

impl IntoIterator for SketchSet {
    type Item = Sketch;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Sketch> for SketchSet {
    fn from_iter<I: IntoIterator<Item = Sketch>>(iter: I) -> Self {
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
    use crate::source::{Tool, ToolSet};

    #[test]
    fn full_cycle() -> Result<()> {
        let raw = r#"type = "sketch"
id = "calm-dragon"
title = "Calm dragon"
asset = "calm-dragon.png"
author = "arnau"
publication_date = 2017-09-29
tools = ["ipadpro"]
summary = "Dragon head drawn with SketchBook's fountain pen  and colored with Sketches Pro's watercolor"
"#;
        let raw_tool = r#"---
type: tool
id: ipadpro
name: iPad Pro
---"#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Sketch::from_str(raw)?;
        let tool = Tool::from_str(raw_tool)?;

        SketchSet::add(&tx, resource.clone())?;
        ToolSet::add(&tx, tool)?;

        let cached = SketchSet::find(&tx, &resource.id)?.expect("resource to be cached");

        tx.commit()?;

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        Ok(())
    }
}

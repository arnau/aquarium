//! This module defines bulletin for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use super::BulletinEntry;
use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::{Resource, ResourceSet};

/// A bulletin issue resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bulletin {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    publication_date: String,
    #[serde(deserialize_with = "super::de_trim")]
    summary: String,
    entries: Vec<BulletinEntry>,
}

impl Resource for Bulletin {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Bulletin {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.publication_date.digest(hasher);
        self.summary.digest(hasher);
        self.entries.digest(hasher);
    }
}

impl FromStr for Bulletin {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let resource: Bulletin = toml::from_str(blob)?;

        Ok(resource)
    }
}

impl fmt::Display for Bulletin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string(&self).expect("resource to serialise as TOML");

        write!(f, "{}", &toml)
    }
}

impl From<Bulletin> for BulletinRecord {
    fn from(resource: Bulletin) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            summary: resource.summary,
            publication_date: resource.publication_date.to_string(),
        }
    }
}

fn from_record(tx: &Transaction, record: BulletinRecord) -> Result<Bulletin> {
    let entries: Vec<BulletinEntry> =
        BulletinEntryRecordSet::select(tx, Some(record.id.to_string()))?
            .into_iter()
            .map(BulletinEntry::from)
            .collect();
    let resource = Bulletin {
        _type: "bulletin".to_string(),
        id: record.id,
        publication_date: record.publication_date,
        summary: record.summary,
        entries,
    };

    Ok(resource)
}

#[derive(Clone, Debug)]
pub struct BulletinSet {
    inner: Vec<Bulletin>,
}

impl BulletinSet {
    pub fn new(inner: Vec<Bulletin>) -> Self {
        Self { inner }
    }
}

impl ReadCache for BulletinSet {
    type Item = Bulletin;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = BulletinRecord::select(tx, id)? {
            let resource = from_record(tx, record)?;

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = BulletinRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(|record| from_record(tx, record))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(resources))
    }
}

impl WriteCache for BulletinSet {
    type Item = Bulletin;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        for entry in &resource.entries {
            let record = BulletinEntryRecord::from((Some(resource.id.clone()), entry));
            record.insert(tx)?;
        }

        let record = BulletinRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        BulletinRecord::delete(tx, id)
    }
}

impl ResourceSet for BulletinSet {}

impl IntoIterator for BulletinSet {
    type Item = Bulletin;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Bulletin> for BulletinSet {
    fn from_iter<I: IntoIterator<Item = Bulletin>>(iter: I) -> Self {
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
        let raw = r#"type = "bulletin"
id = "2021-W01"
publication_date = "2021-01-10"
summary = "This week has been about preql, sqlite, web component styling, apache arrow and hexagonal grids."

[[entries]]
url = "https://nolanlawson.com/2021/01/03/options-for-styling-web-components/"
title = "Options for styling web components"
summary = "An article explaining the tradeoffs of styling Web Components."
content_type = "text"

[[entries]]
url = "https://www.redblobgames.com/grids/hexagons/"
title = "Hexagon Grids"
summary = "A guide on how to make hexagonal grids, implement a coordinate system, distances and more."
content_type = "text"

[[entries]]
url = "https://github.com/inukshuk/sqleton"
title = "sqleton"
summary = "A command-line interface to visualise SQLite database schemas."
content_type = "text"

[[entries]]
url = "https://github.com/erezsh/Preql"
title = "Preql"
summary = "An interpreted, relational programming language, that specializes in database queries."
content_type = "text"

[[entries]]
url = "https://www.influxdata.com/blog/apache-arrow-parquet-flight-and-their-ecosystem-are-a-game-changer-for-olap/"
title = "Apache Arrow, Parquet, Flight and Their Ecosystem are a Game Changer for OLAP"
summary = "An article introducing Apache Arrow, how its components fit together and its overall maturity."
content_type = "text"

[[entries]]
url = "https://napi.rs/"
title = "napi-rs"
summary = "A library for building pre-compiled NodeJS addons in Rust."
content_type = "text"
"#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Bulletin::from_str(raw)?;

        BulletinSet::add(&tx, resource.clone())?;

        let cached = BulletinSet::find(&tx, &resource.id)?.expect("resource to be cached");

        tx.commit()?;

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        Ok(())
    }
}

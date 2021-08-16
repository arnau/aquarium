//! This module defines the entrance for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::markdown::take_frontmatter;
use crate::Resource;

/// An entrance resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Entrance {
    id: String,
    body: Option<String>,
}

impl Resource for Entrance {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Entrance {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.body.digest(hasher);
    }
}

impl FromStr for Entrance {
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
            body,
        })
    }
}

impl fmt::Display for Entrance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = Metadata::from(self);
        let yaml = serde_yaml::to_string(&metadata).expect("metadata to encode as yaml");

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
}

impl From<&Entrance> for Metadata {
    fn from(resource: &Entrance) -> Metadata {
        Metadata {
            _type: "entrance".into(),
            id: resource.id.clone(),
        }
    }
}

impl From<Entrance> for EntranceRecord {
    fn from(resource: Entrance) -> Self {
        Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            body: resource.body,
        }
    }
}

impl From<EntranceRecord> for Entrance {
    fn from(record: EntranceRecord) -> Self {
        Self {
            id: record.id,
            body: record.body,
        }
    }
}

impl ReadCache for Entrance {
    type Item = Entrance;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = EntranceRecord::select(tx, id)? {
            let resource = Self::Item::from(record);

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(_tx: &Transaction) -> Result<Self> {
        unimplemented!()
    }
}

impl WriteCache for Entrance {
    type Item = Entrance;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = EntranceRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        EntranceRecord::delete(tx, id)
    }
}

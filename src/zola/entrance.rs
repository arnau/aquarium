//! This module covers the entrance from a Zola point of view.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::ZolaResource;
use crate::cache::records::*;
use crate::cache::Transaction;
use crate::resource_type::ResourceType;
use crate::stamp::Date;

#[derive(Debug, Clone)]
pub struct Entrance {
    pub metadata: Metadata,
    pub body: Option<String>,
}

impl ZolaResource for Entrance {
    fn id(&self) -> &str {
        &self.metadata.extra.id
    }

    fn path(&self) -> String {
        format!("_index.md")
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Entrance)
    }
}

impl fmt::Display for Entrance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = toml::to_string(&self.metadata).expect("metadata to serialize as TOML");

        writeln!(f, "+++")?;
        write!(f, "{}", &metadata)?;
        writeln!(f, "+++")?;
        if let Some(body) = &self.body {
            write!(f, "{}", body)?;
        }

        write!(f, "")
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub(crate) title: String,
    pub(crate) template: String,
    pub(crate) extra: Extra,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extra {
    pub(crate) id: String,
    pub(crate) latest_updates: Vec<Update>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Update {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) summary: Option<String>,
    pub(crate) section: String,
    pub(crate) date: Date,
    pub(crate) path: String,
}

fn amass_updates(tx: &Transaction) -> Result<Vec<Update>> {
    UpdateRecordSet::select(tx)?
        .into_iter()
        .map(Update::try_from)
        .collect()
}

impl TryFrom<UpdateRecord> for Update {
    type Error = anyhow::Error;

    fn try_from(record: UpdateRecord) -> Result<Self> {
        let date = Date::from_str(&record.date)?;
        let path = if &record.section == "bulletins" {
            format!("/{}/{}/{}", &record.section, date.year(), &record.id)
        } else {
            format!("/{}/{}", &record.section, &record.id)
        };
        let resource = Self {
            path,
            id: record.id,
            title: record.title,
            section: record.section,
            summary: record.summary,
            date,
        };

        Ok(resource)
    }
}

pub fn find(tx: &Transaction) -> Result<Option<Entrance>> {
    if let Some(record) = EntranceRecord::select(&tx, "entrance")? {
        let latest_updates = amass_updates(&tx)?;
        let extra = Extra {
            id: record.id,
            latest_updates,
        };
        let metadata = Metadata {
            title: "Recent updates".to_string(),
            template: "index.html".to_string(),
            extra,
        };
        let resource = Entrance {
            metadata,
            body: record.body,
        };

        Ok(Some(resource))
    } else {
        Ok(None)
    }
}

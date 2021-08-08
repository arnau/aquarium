//! This module defines the settings for the Zola stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::ZolaResource;
use crate::cache::records::*;
use crate::cache::Transaction;
use crate::resource_type::ResourceType;

/// A settings resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "type")]
    _type: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub copyright: String,
    pub navigation: Vec<String>,
    licence: Licence,
}

impl ZolaResource for Settings {
    fn id(&self) -> &str {
        &self.id
    }

    fn path(&self) -> String {
        "settings.toml".to_owned()
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Settings)
    }
}

impl FromStr for Settings {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let resource: Settings = toml::from_str(blob)?;

        Ok(resource)
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = toml::to_string(&self).expect("settings to encode as toml");

        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Licence {
    url: String,
    name: String,
}

impl TryFrom<SettingsRecord> for Settings {
    type Error = anyhow::Error;

    fn try_from(record: SettingsRecord) -> Result<Self> {
        let s = String::from_utf8(record.blob)?;
        let resource = Settings::from_str(&s)?;

        Ok(resource)
    }
}

pub fn find(tx: &Transaction, id: &str) -> Result<Option<Settings>> {
    if let Some(record) = SettingsRecord::select(tx, id)? {
        let resource = Settings::try_from(record)?;

        Ok(Some(resource))
    } else {
        Ok(None)
    }
}

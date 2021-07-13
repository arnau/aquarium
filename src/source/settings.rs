//! This module defines the settings for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::{Resource, ResourceSet};

/// A settings resource.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    title: String,
    description: String,
    url: String,
    copyright: String,
    navigation: Vec<String>,
    licence: Licence,
}

impl Resource for Settings {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Settings {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.title.digest(hasher);
        self.description.digest(hasher);
        self.url.digest(hasher);
        self.copyright.digest(hasher);
        self.navigation.digest(hasher);
        self.licence.digest(hasher);
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

impl Digest for Licence {
    fn digest(&self, hasher: &mut Hasher) {
        self.url.digest(hasher);
        self.name.digest(hasher);
    }
}

impl TryFrom<Settings> for SettingsRecord {
    type Error = anyhow::Error;

    fn try_from(resource: Settings) -> Result<Self> {
        let blob = resource.to_string();
        let record = Self {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            blob: blob.as_bytes().into(),
        };

        Ok(record)
    }
}

impl TryFrom<SettingsRecord> for Settings {
    type Error = anyhow::Error;

    fn try_from(record: SettingsRecord) -> Result<Self> {
        let s = String::from_utf8(record.blob)?;
        let resource = Settings::from_str(&s)?;

        Ok(resource)
    }
}

#[derive(Clone, Debug)]
pub struct SettingsSet {
    inner: Vec<Settings>,
}

impl SettingsSet {
    pub fn new(inner: Vec<Settings>) -> Self {
        Self { inner }
    }
}

impl ReadCache for SettingsSet {
    type Item = Settings;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = SettingsRecord::select(tx, id)? {
            let resource = Self::Item::try_from(record)?;

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = SettingsRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(Self::Item::try_from)
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::new(resources))
    }
}

impl WriteCache for SettingsSet {
    type Item = Settings;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = SettingsRecord::try_from(resource)?;
        record.insert(&tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        SettingsRecord::delete(&tx, id)
    }
}

impl ResourceSet for SettingsSet {}

impl IntoIterator for SettingsSet {
    type Item = Settings;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Settings> for SettingsSet {
    fn from_iter<I: IntoIterator<Item = Settings>>(iter: I) -> Self {
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
        let raw = r#"type = "settings"
id = "main"
title = "Aquarium example"
description = "An example for Aquarium"
url = "https://aquarium.netlify.app/"
copyright = "2021, Arnau Siches"
navigation = ["notes", "sketches", "bulletins", "projects"]

[licence]
url = "http://creativecommons.org/licenses/by-nc/4.0/"
name = "Creative Commons Attribution-NonCommercial 4.0 International License"
"#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Settings::from_str(raw)?;

        SettingsSet::add(&tx, resource.clone())?;

        let cached = SettingsSet::find(&tx, &resource.id)?.expect("resource to be cached");

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        tx.commit()?;

        Ok(())
    }
}

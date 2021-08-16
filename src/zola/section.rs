//! This module covers the [Zola section] point of view.
//!
//! [Zola section]: https://www.getzola.org/documentation/content/section/
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::ZolaResource;
use crate::cache::records::*;
use crate::cache::Transaction;
use crate::markdown::strip;
use crate::resource_type::ResourceType;

/// Represents a [Zola section].
///
/// This is a reflection of the Source [`crate::source::section::Section`].
///
/// [Zola section]: https://www.getzola.org/documentation/content/section/
#[derive(Debug, Clone)]
pub struct Section {
    pub metadata: Metadata,
    pub body: Option<String>,
}

impl ZolaResource for Section {
    fn id(&self) -> &str {
        &self.metadata.extra.id
    }

    fn path(&self) -> String {
        format!("{}/", self.id())
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        self.metadata.extra.resource_type.as_ref()
    }
}

impl fmt::Display for Section {
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
    pub(crate) description: Option<String>,
    pub(crate) slug: String,
    pub(crate) template: String,
    pub(crate) sort_by: String,
    pub(crate) insert_anchor_links: String,
    pub(crate) in_search_index: bool,
    pub(crate) generate_feed: bool,
    pub(crate) extra: Extra,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extra {
    pub(crate) id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resource_type: Option<ResourceType>,
}

impl TryFrom<SectionRecord> for Section {
    type Error = anyhow::Error;

    fn try_from(record: SectionRecord) -> Result<Self> {
        let resource_type = if let Some(resource_type) = &record.resource_type {
            Some(ResourceType::from_str(resource_type)?)
        } else {
            None
        };
        let extra = Extra {
            id: record.id.clone(),
            resource_type,
        };
        let metadata = Metadata {
            title: record.title,
            description: record.body.as_ref().map(|s| strip(s)),
            slug: record.id.clone(),
            template: format!("{}.html", &record.id),
            sort_by: "date".to_string(),
            insert_anchor_links: "left".to_string(),
            in_search_index: true,
            generate_feed: false,
            extra,
        };
        let resource = Self {
            metadata,
            body: record.body,
        };

        Ok(resource)
    }
}

pub fn amass(tx: &Transaction) -> Result<Vec<Section>> {
    let records = SectionRecordSet::select(tx)?;
    let mut result = Vec::new();

    for record in records {
        let resource = Section::try_from(record)?;

        result.push(resource);
    }

    Ok(result)
}

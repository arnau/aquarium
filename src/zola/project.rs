//! This module covers the [Zola page] for a project.
//!
//! [Zola page]: https://www.getzola.org/documentation/content/page/
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::ZolaResource;
use crate::cache::{Row, Transaction};
use crate::markdown::strip;
use crate::resource_type::ResourceType;
use crate::stamp::Date;

#[derive(Debug, Clone)]
pub struct Project {
    pub metadata: Metadata,
    pub body: String,
}

impl ZolaResource for Project {
    fn id(&self) -> &str {
        &self.metadata.extra.id
    }

    fn path(&self) -> String {
        format!("{}.md", self.id())
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Project)
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = toml::to_string(&self.metadata).expect("metadata to serialize as TOML");

        writeln!(f, "+++")?;
        write!(f, "{}", &metadata)?;
        writeln!(f, "+++")?;
        write!(f, "{}", &self.body)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) date: Date,
    pub(crate) template: String,
    pub(crate) in_search_index: bool,
    pub(crate) extra: Extra,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extra {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) status: String,
    pub(crate) start_date: Date,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) end_date: Option<Date>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source_url: Option<String>,
}

impl TryFrom<&Row<'_>> for Project {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let summary: String = row.get(2)?;
        let body: String = row.get(3)?;
        let status: String = row.get(4)?;
        let raw_start_date: String = row.get(5)?;
        let raw_end_date: Option<String> = row.get(6)?;
        let source_url: Option<String> = row.get(7)?;

        let start_date = Date::from_str(&raw_start_date)?;
        let end_date = if let Some(end_date) = raw_end_date {
            Some(Date::from_str(&end_date)?)
        } else {
            None
        };
        let clean_title = strip(&title);
        let clean_description = strip(&summary);

        let extra = Extra {
            id,
            title,
            summary,
            status,
            start_date: start_date.clone(),
            end_date,
            source_url,
        };
        let metadata = Metadata {
            title: clean_title,
            description: clean_description,
            date: start_date,
            template: "project.html".to_owned(),
            in_search_index: true,
            extra,
        };
        let resource = Self { metadata, body };

        Ok(resource)
    }
}

pub fn amass(tx: &Transaction) -> Result<Vec<Project>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT
            id,
            name,
            summary,
            body,
            status,
            start_date,
            end_date,
            source_url
        FROM
            project
        "#,
    )?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let record = Project::try_from(row)?;
        set.push(record);
    }

    Ok(set)
}

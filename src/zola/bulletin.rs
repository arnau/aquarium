//! This module covers the Zola page for a bulletin.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::ZolaResource;
use crate::cache::{params, Row, Transaction};
use crate::markdown::strip;
use crate::resource_type::ResourceType;
use crate::stamp::Date;

#[derive(Debug, Clone)]
pub struct Bulletin {
    pub metadata: Metadata,
    pub body: String,
}

impl ZolaResource for Bulletin {
    fn id(&self) -> &str {
        &self.metadata.extra.id
    }

    fn path(&self) -> String {
        format!("{}.md", self.id())
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Bulletin)
    }
}

impl fmt::Display for Bulletin {
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
    pub(crate) slug: String,
    pub(crate) template: String,
    pub(crate) in_search_index: bool,
    pub(crate) extra: Extra,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extra {
    pub(crate) id: String,
    pub(crate) entries: Vec<Entry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Entry {
    pub(crate) url: String,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) content_type: String,
}

impl TryFrom<&Row<'_>> for Entry {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let title: String = row.get(1)?;
        let resource = Self {
            url: row.get(0)?,
            slug: slug::slugify(&title),
            title,
            summary: row.get(2)?,
            content_type: row.get(3)?,
        };

        Ok(resource)
    }
}

impl TryFrom<&Row<'_>> for Bulletin {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id: String = row.get(0)?;
        let body: String = row.get(1)?;
        let description = strip(&body);
        let raw_date: String = row.get(2)?;
        let date = Date::from_str(&raw_date)?;

        let extra = Extra {
            id: id.clone(),
            entries: Vec::new(),
        };
        let metadata = Metadata {
            slug: id.clone(),
            title: id,
            description,
            date,
            template: "bulletin.html".to_owned(),
            in_search_index: true,
            extra,
        };
        let resource = Self { metadata, body };

        Ok(resource)
    }
}

fn select_entries(tx: &Transaction, issue_id: &str) -> Result<Vec<Entry>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT
            url,
            title,
            summary,
            content_type
        FROM
            bulletin_entry
        WHERE
            issue_id = ?
        "#,
    )?;
    let mut rows = stmt.query(params![issue_id])?;

    while let Some(row) = rows.next()? {
        let record = Entry::try_from(row)?;
        set.push(record);
    }

    Ok(set)
}

pub fn amass(tx: &Transaction, year: &str) -> Result<Vec<Bulletin>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT
            id,
            summary,
            publication_date
        FROM
            bulletin_issue
        WHERE
            strftime('%Y', publication_date) = ?
        "#,
    )?;
    let mut rows = stmt.query(params![year])?;

    while let Some(row) = rows.next()? {
        let mut resource = Bulletin::try_from(row)?;
        let entries = select_entries(tx, resource.id())?;
        resource.metadata.extra.entries = entries;

        set.push(resource);
    }

    Ok(set)
}

//! This module covers the [Zola page] for a note.
//!
//! [Zola page]: https://www.getzola.org/documentation/content/page/
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::author::Author;
use super::ZolaResource;
use crate::cache::{Row, Transaction};
use crate::markdown;
use crate::resource_type::ResourceType;
use crate::stamp::Date;

#[derive(Debug, Clone)]
pub struct Note {
    pub metadata: Metadata,
    pub body: String,
}

impl ZolaResource for Note {
    fn id(&self) -> &str {
        &self.metadata.extra.id
    }

    fn path(&self) -> String {
        format!("{}.md", self.id())
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Note)
    }
}

impl fmt::Display for Note {
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
    pub(crate) author: Author,
}

impl TryFrom<&Row<'_>> for Note {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let summary: String = row.get(2)?;
        let date: String = row.get(3)?;
        let body: String = row.get(5)?;

        let author = Author {
            id: row.get(4)?,
            name: row.get(6)?,
            guest: row.get(7)?,
        };
        let extra = Extra {
            id: id.clone(),
            title: title.clone(),
            summary: summary.clone(),
            author,
        };
        let metadata = Metadata {
            title: markdown::strip(&title),
            description: markdown::strip(&summary),
            date: Date::from_str(&date)?,
            template: "note.html".to_owned(),
            in_search_index: true,
            extra,
        };
        let resource = Self {
            metadata,
            body: markdown::enrich(&body)?,
        };

        Ok(resource)
    }
}

pub fn amass(tx: &Transaction) -> Result<Vec<Note>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT
            note.id,
            note.title,
            note.summary,
            note.publication_date,
            note.author_id,
            note.body,
            person.name,
            person.guest
        FROM
            note
        INNER JOIN
            person
        ON
            note.author_id = person.id
            "#,
    )?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let record = Note::try_from(row)?;
        set.push(record);
    }

    Ok(set)
}

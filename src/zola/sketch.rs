//! This module covers the Zola page for a sketch.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::author::Author;
use super::ZolaResource;
use crate::cache::{params, Row, Transaction};
use crate::markdown::strip;
use crate::resource_type::ResourceType;
use crate::stamp::Date;

#[derive(Debug, Clone)]
pub struct Asset {
    id: String,
    blob: Vec<u8>,
}

impl Asset {
    pub fn blob(&self) -> &[u8] {
        &self.blob
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct SketchBundle {
    sketch: Sketch,
    asset: Asset,
}

#[derive(Debug, Clone)]
pub struct Sketch {
    pub metadata: Metadata,
    pub body: Option<String>,
}

impl ZolaResource for Sketch {
    fn id(&self) -> &str {
        &self.metadata.extra.id
    }

    fn path(&self) -> String {
        format!("index.md")
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Bulletin)
    }
}

impl fmt::Display for Sketch {
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
    pub(crate) date: Date,
    pub(crate) slug: String,
    pub(crate) template: String,
    pub(crate) in_search_index: bool,
    pub(crate) extra: Extra,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Extra {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) asset_id: String,
    pub(crate) author: Author,
    pub(crate) tools: Vec<Tool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tool {
    id: String,
    name: String,
    url: Option<String>,
}

impl TryFrom<&Row<'_>> for Tool {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id: String = row.get(0)?;
        let name: String = row.get(1)?;
        let url: Option<String> = row.get(2)?;
        let resource = Self { id, name, url };

        Ok(resource)
    }
}

impl TryFrom<&Row<'_>> for SketchBundle {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let body: Option<String> = row.get(2)?;
        let date: String = row.get(3)?;
        let asset_id: String = row.get(5)?;
        let asset: Vec<u8> = row.get(8)?;
        let author = Author {
            id: row.get(4)?,
            name: row.get(6)?,
            guest: row.get(7)?,
        };
        let extra = Extra {
            id: id.clone(),
            title: title.clone(),
            asset_id: asset_id.clone(),
            author,
            tools: Vec::new(),
        };
        let metadata = Metadata {
            title: strip(&title),
            description: body.as_ref().map(|s| strip(&s)),
            slug: id.clone(),
            date: Date::from_str(&date)?,
            template: "sketch.html".to_owned(),
            in_search_index: true,
            extra,
        };
        let resource = Sketch { metadata, body };

        Ok(SketchBundle {
            sketch: resource,
            asset: Asset {
                blob: asset,
                id: asset_id,
            },
        })
    }
}

pub fn select_tools(tx: &Transaction, sketch_id: &str) -> Result<Vec<Tool>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT
            tool.id,
            tool.name,
            tool.url
        FROM
            tool
        INNER JOIN
            sketch_tool
        ON
            tool.id = sketch_tool.tool_id
        WHERE
            sketch_tool.sketch_id = ?
        "#,
    )?;
    let mut rows = stmt.query(params![sketch_id])?;

    while let Some(row) = rows.next()? {
        let resource = Tool::try_from(row)?;

        set.push(resource);
    }

    Ok(set)
}

pub fn amass(tx: &Transaction) -> Result<Vec<(Sketch, Asset)>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT
            sketch.id,
            sketch.title,
            sketch.summary,
            sketch.publication_date,
            sketch.author_id,
            sketch.asset_id,
            person.name,
            person.guest,
            asset.content
        FROM
            sketch
        INNER JOIN
            person
        ON
            sketch.author_id = person.id
        INNER JOIN
            asset
        ON
            sketch.asset_id = asset.id
            "#,
    )?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let SketchBundle { mut sketch, asset } = SketchBundle::try_from(row)?;
        let tools = select_tools(tx, sketch.id())?;
        sketch.metadata.extra.tools = tools;

        set.push((sketch, asset));
    }

    Ok(set)
}

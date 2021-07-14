//! This module covers the Zola sectionfor a bulletin year.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use super::ZolaResource;
use crate::cache::{Row, Transaction};
use crate::resource_type::ResourceType;
use crate::stamp::Date;

#[derive(Debug, Clone)]
pub struct BulletinYear {
    pub metadata: Metadata,
}

impl ZolaResource for BulletinYear {
    fn id(&self) -> &str {
        &self.metadata.title
    }

    fn path(&self) -> String {
        format!("{}/", self.id())
    }

    fn resource_type(&self) -> Option<&ResourceType> {
        Some(&ResourceType::Section)
    }
}

impl fmt::Display for BulletinYear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = toml::to_string(&self.metadata).expect("metadata to serialize as TOML");

        writeln!(f, "+++")?;
        write!(f, "{}", &metadata)?;
        writeln!(f, "+++")
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
pub struct Extra {}

impl TryFrom<&Row<'_>> for BulletinYear {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self> {
        let id: String = row.get(0)?;
        let description = format!("Issues from {}", &id);
        let date = Date::from_str(&format!("{}-01-01", &id))?;

        let extra = Extra {};
        let metadata = Metadata {
            title: id,
            description,
            date,
            template: "bulletins_year.html".to_owned(),
            in_search_index: true,
            extra,
        };
        let resource = Self { metadata };

        Ok(resource)
    }
}

pub fn amass(tx: &Transaction) -> Result<Vec<BulletinYear>> {
    let mut set = Vec::new();
    let mut stmt = tx.prepare(
        r#"
        SELECT DISTINCT
            strftime('%Y', publication_date) as id
        FROM
            bulletin_issue
        "#,
    )?;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let resource = BulletinYear::try_from(row)?;

        set.push(resource);
    }

    Ok(set)
}

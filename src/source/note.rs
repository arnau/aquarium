//! This module defines the note and note set for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::checksum::{Digest, Hasher};
use crate::markdown::Markdown;
use crate::stamp::Date;
use crate::Resource;

/// A note.
#[derive(Clone, Debug)]
pub struct Note {
    id: String,
    title: String,
    summary: String,
    publication_date: Date,
    author: String,
    section: String,
    body: String,
}

impl Resource for Note {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl Digest for Note {
    fn digest(&self, hasher: &mut Hasher) {
        self.id.digest(hasher);
        self.title.digest(hasher);
        self.summary.digest(hasher);
        self.publication_date.digest(hasher);
        self.author.digest(hasher);
        self.section.digest(hasher);
        self.body.digest(hasher);
    }
}

impl FromStr for Note {
    type Err = anyhow::Error;

    fn from_str(blob: &str) -> Result<Self, Self::Err> {
        let Markdown {
            frontmatter,
            title,
            summary,
            body,
        } = Markdown::from_str(blob)?;
        let metadata: Metadata = serde_yaml::from_str(&frontmatter)?;

        Ok(Self {
            id: metadata.id,
            title,
            summary: summary.expect("notes must have a summary"),
            publication_date: Date::from_str(&metadata.publication_date)?,
            author: metadata.author,
            section: "notes".into(),
            body,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Metadata {
    id: String,
    publication_date: String,
    author: String,
}

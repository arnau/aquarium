//! This module defines the note and note set for the Source stage.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::iter::FromIterator;
use std::str::FromStr;

use crate::cache::records::*;
use crate::cache::{ReadCache, Transaction, WriteCache};
use crate::checksum::{Digest, Hasher};
use crate::markdown::Markdown;
use crate::stamp::Date;
use crate::{Resource, ResourceSet};

/// A note resource.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    id: String,
    title: String,
    summary: String,
    publication_date: Date,
    author: String,
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
            body,
        })
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let metadata = Metadata::from(self);
        let yaml = serde_yaml::to_string(&metadata).expect("note metadata to encode as yaml");

        writeln!(f, "---")?;
        write!(f, "{}", yaml)?;
        writeln!(f, "---")?;
        writeln!(f, "# {}", &self.title)?;
        writeln!(f, "\n{}\n", &self.summary)?;
        writeln!(f, "<!-- body -->\n")?;
        write!(f, "{}", &self.body)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Metadata {
    #[serde(rename = "type")]
    _type: String,
    id: String,
    publication_date: String,
    author: String,
}

impl From<&Note> for Metadata {
    fn from(resource: &Note) -> Metadata {
        Metadata {
            _type: "note".into(),
            id: resource.id.clone(),
            publication_date: resource.publication_date.to_string(),
            author: resource.author.clone(),
        }
    }
}

impl From<Note> for NoteRecord {
    fn from(resource: Note) -> NoteRecord {
        NoteRecord {
            checksum: resource.checksum().to_string(),
            id: resource.id,
            title: resource.title,
            summary: resource.summary,
            publication_date: resource.publication_date.to_string(),
            author_id: resource.author,
            body: resource.body,
        }
    }
}

impl TryFrom<NoteRecord> for Note {
    type Error = anyhow::Error;

    fn try_from(record: NoteRecord) -> Result<Note> {
        let resource = Note {
            id: record.id,
            title: record.title,
            summary: record.summary,
            publication_date: Date::from_str(&record.publication_date)?,
            author: record.author_id,
            body: record.body,
        };

        Ok(resource)
    }
}

#[derive(Clone, Debug)]
pub struct NoteSet {
    inner: Vec<Note>,
}

impl NoteSet {
    pub fn new(inner: Vec<Note>) -> Self {
        Self { inner }
    }
}

impl ReadCache for NoteSet {
    type Item = Note;

    fn find(tx: &Transaction, id: &str) -> Result<Option<Self::Item>> {
        if let Some(record) = NoteRecord::select(tx, id)? {
            let resource = Note::try_from(record)?;

            Ok(Some(resource))
        } else {
            Ok(None)
        }
    }

    fn amass(tx: &Transaction) -> Result<Self> {
        let records = NoteRecordSet::select(tx)?;
        let resources = records
            .into_iter()
            .map(Note::try_from)
            .collect::<Result<Vec<Note>>>()?;

        Ok(Self::new(resources))
    }
}

impl WriteCache for NoteSet {
    type Item = Note;

    fn add(tx: &Transaction, resource: Self::Item) -> Result<()> {
        let record = NoteRecord::from(resource);
        record.insert(tx)?;

        Ok(())
    }

    fn remove(tx: &Transaction, id: &str) -> Result<()> {
        NoteRecord::delete(tx, id)
    }
}

impl ResourceSet for NoteSet {}

impl IntoIterator for NoteSet {
    type Item = Note;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<Note> for NoteSet {
    fn from_iter<I: IntoIterator<Item = Note>>(iter: I) -> Self {
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
        let raw = r#"---
type: note
id: a-note
publication_date: 2021-07-07
author: arnau
---
# A simple note

This note is showing the minimum required for a note.

<!-- body -->

From here onwards it's the body of the note so the first H1 will become the _title_ and any content before the `<!-- body
-->` mark will become the _summary_."#;
        let mut cache = Cache::connect(":memory:")?;
        let tx = cache.transaction()?;
        let resource = Note::from_str(raw)?;

        NoteSet::add(&tx, resource.clone())?;

        let cached = NoteSet::find(&tx, &resource.id)?.expect("note to be cached");

        assert_eq!(&cached.to_string(), raw);
        assert_eq!(cached, resource);

        tx.commit()?;

        Ok(())
    }
}

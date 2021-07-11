//! This module deals with the source stage.
//!
//! The source stage is responsible for taking any known resource from the file system and stored into the cache for
//! other stages to consume.

use anyhow::Result;
use log::{info, warn};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;
use walkdir::{DirEntry, WalkDir};

pub mod note;
pub mod person;
pub mod project;
pub mod section;

pub use note::{Note, NoteSet};
pub use person::{Person, PersonSet};
pub use project::{Project, ProjectSet};
pub use section::{Section, SectionSet};

use crate::cache::{Transaction, WriteCache};
use crate::resource_type::ResourceType;
use crate::Cache;

/// Walks through the given path and caches any know resource.
pub fn read(source_dir: &Path, cache: &mut Cache) -> Result<()> {
    let tx = cache.transaction()?;
    let walker = WalkDir::new(source_dir).into_iter();

    for result in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = result?;
        let path = entry.path();

        if path.is_file() {
            process_source(&path, &tx)?;
        }
    }

    tx.commit()?;

    Ok(())
}

fn process_source(entry: &Path, tx: &Transaction) -> Result<()> {
    let path = entry.display().to_string();
    let mut file = File::open(&entry)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let resource_type = ResourceType::from_hint(&contents);

    match resource_type {
        // ResourceType::Bulletin => {}
        ResourceType::Note => {
            let resource = Note::from_str(&contents)?;
            NoteSet::add(&tx, resource)?;
            info!("note: {}", &path);
        }
        ResourceType::Person => {
            let resource = Person::from_str(&contents)?;
            PersonSet::add(&tx, resource)?;
            info!("person: {}", &path);
        }
        ResourceType::Project => {
            let resource = Project::from_str(&contents)?;
            ProjectSet::add(&tx, resource)?;
            info!("project: {}", &path);
        }
        ResourceType::Section => {
            let resource = Section::from_str(&contents)?;
            SectionSet::add(&tx, resource)?;
            info!("section: {}", &path);
        }
        // ResourceType::Settings => {}
        // ResourceType::Sketch => {}
        // ResourceType::Tool => {}
        _ => {
            warn!("unknown type {}", &path);
        }
    }
    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

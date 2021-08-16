//! This module deals with the source stage.
//!
//! The source stage is responsible for taking any known resource from the file system and stored into the cache for
//! other stages to consume.

use anyhow::Result;
use log::{info, warn};
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;
use walkdir::{DirEntry, WalkDir};

pub mod asset;
pub mod bulletin_entry;
pub mod bulletin_issue;
pub mod bulletin_stash;
pub mod entrance;
pub mod note;
pub mod person;
pub mod project;
pub mod section;
pub mod settings;
pub mod sketch;
pub mod tool;

pub use asset::{Asset, AssetSet};
pub use bulletin_entry::BulletinEntry;
pub use bulletin_issue::{Bulletin, BulletinSet};
pub use bulletin_stash::BulletinStash;
pub use entrance::Entrance;
pub use note::{Note, NoteSet};
pub use person::{Person, PersonSet};
pub use project::{Project, ProjectSet};
pub use section::{Section, SectionSet};
pub use settings::{Settings, SettingsSet};
pub use sketch::{Sketch, SketchSet};
pub use tool::{Tool, ToolSet};

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
            process_source(path, &tx)?;
        }
    }

    tx.commit()?;

    Ok(())
}

fn process_source(entry: &Path, tx: &Transaction) -> Result<()> {
    let path = entry.display();
    let mut file = File::open(&entry)?;
    let resource_extensions = ["md", "toml"];

    // Binary assets
    if let Some(osstr) = entry.extension() {
        let extension = osstr.to_string_lossy();
        let id = entry
            .file_name()
            .expect("file name to exist for an asset")
            .to_string_lossy();

        if !resource_extensions.iter().any(|rex| rex == &extension) {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            let resource = Asset::new(id.to_string(), extension.to_string(), buffer);
            AssetSet::add(tx, resource)?;
            info!("source(asset): {}", &path);

            return Ok(());
        }
    }

    // Textual resources
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Skipping anything without a hint.
    if let Ok(resource_type) = ResourceType::from_hint(&contents) {
        match resource_type {
            ResourceType::BulletinStash => {
                info!("source(bulletin_stash): {}", &path);
                let resource = BulletinStash::from_str(&contents)?;
                BulletinStash::add(tx, resource)?;
            }
            ResourceType::Bulletin => {
                info!("source(bulletin): {}", &path);
                let resource = Bulletin::from_str(&contents)?;
                BulletinSet::add(tx, resource)?;
            }
            ResourceType::Entrance => {
                info!("source(entrance): {}", &path);
                let resource = Entrance::from_str(&contents)?;
                Entrance::add(tx, resource)?;
            }
            ResourceType::Note => {
                info!("source(note): {}", &path);
                let resource = Note::from_str(&contents)?;
                NoteSet::add(tx, resource)?;
            }
            ResourceType::Person => {
                info!("source(person): {}", &path);
                let resource = Person::from_str(&contents)?;
                PersonSet::add(tx, resource)?;
            }
            ResourceType::Project => {
                info!("source(project): {}", &path);
                let resource = Project::from_str(&contents)?;
                ProjectSet::add(tx, resource)?;
            }
            ResourceType::Section => {
                info!("source(section): {}", &path);
                let resource = Section::from_str(&contents)?;
                SectionSet::add(tx, resource)?;
            }
            ResourceType::Settings => {
                info!("source(settings): {}", &path);
                let resource = Settings::from_str(&contents)?;
                SettingsSet::add(tx, resource)?;
            }
            ResourceType::Sketch => {
                info!("source(sketch): {}", &path);
                let resource = Sketch::from_str(&contents)?;
                SketchSet::add(tx, resource)?;
            }
            ResourceType::Tool => {
                info!("source(tool): {}", &path);
                let resource = Tool::from_str(&contents)?;
                ToolSet::add(tx, resource)?;
            }
            ResourceType::Unknown(s) => {
                warn!("unknown type '{}' {}", &s, &path);
            } // _ => {
              //     warn!("unimplemented {}", &path);
              // }
        }
    }

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

pub(crate) fn de_trim<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Ok(value.trim().to_string())
}

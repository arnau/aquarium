use anyhow::Result;
use log::{info, warn};
use std::fmt;
use std::fs;
use std::path::Path;

use crate::resource_type::ResourceType;
use crate::Cache;

pub mod author;
pub mod bulletin;
pub mod bulletin_year;
pub mod entrance;
pub mod note;
pub mod project;
pub mod section;
pub mod settings;
pub mod sketch;

// TODO: Review convergence with `Resource`.
pub trait ZolaResource: fmt::Display {
    fn id(&self) -> &str;

    fn path(&self) -> String;

    fn resource_type(&self) -> Option<&ResourceType>;
}

pub fn write(sink_dir: &Path, cache: &mut Cache) -> Result<()> {
    let tx = cache.transaction()?;

    // Agressively clean previous build.
    if sink_dir.exists() {
        fs::remove_dir_all(sink_dir)?;
    }
    fs::create_dir(sink_dir)?;

    write_resource(
        sink_dir,
        Box::new(settings::find(&tx, "main")?.expect("settings to exist")),
    )?;
    write_resource(
        sink_dir,
        Box::new(entrance::find(&tx)?.expect("entrance to exist")),
    )?;

    let sections = section::amass(&tx)?;
    for section in sections {
        let section_path = sink_dir.join(&section.path());
        let resource_type = section.resource_type();

        fs::create_dir(&section_path)?;
        fs::write(&section_path.join("_index.md"), &section.to_string())?;
        info!("zola(section): {}", section.id());

        match resource_type {
            Some(ResourceType::Note) => {
                let resources = note::amass(&tx)?;
                for resource in resources {
                    write_resource(&section_path, Box::new(resource))?;
                }
            }
            Some(ResourceType::Project) => {
                let resources = project::amass(&tx)?;
                for resource in resources {
                    write_resource(&section_path, Box::new(resource))?;
                }
            }
            Some(ResourceType::Sketch) => {
                let resources = sketch::amass(&tx)?;
                for (resource, asset) in resources {
                    let resource_path = section_path.join(resource.id());
                    fs::create_dir(&resource_path)?;
                    write_resource(&resource_path, Box::new(resource))?;
                    write_asset(&resource_path, asset)?;
                }
            }
            Some(ResourceType::Bulletin) => {
                let resources = bulletin_year::amass(&tx)?;
                for resource in resources {
                    let year_path = section_path.join(&resource.path());
                    fs::create_dir(&year_path)?;
                    fs::write(&year_path.join("_index.md"), &resource.to_string())?;
                    info!("zola(section): {}", resource.id());

                    let bulletins = bulletin::amass(&tx, resource.id())?;
                    for bulletin in bulletins {
                        write_resource(&year_path, Box::new(bulletin))?;
                    }
                }
            }
            Some(typ) => {
                warn!("'{}' is an unimplemented zola resource", typ);
            }
            None => (),
        }
    }

    tx.commit()?;

    Ok(())
}

fn write_resource(sink_dir: &Path, resource: Box<dyn ZolaResource>) -> Result<()> {
    fs::write(sink_dir.join(&resource.path()), &resource.to_string())?;
    info!(
        "zola({}): {}",
        resource
            .resource_type()
            .expect("resource type to be defined"),
        &resource.id()
    );

    Ok(())
}

fn write_asset(path: &Path, asset: sketch::Asset) -> Result<()> {
    fs::write(path.join(asset.id()), asset.blob())?;
    info!("zola(asset): {}", asset.id());

    Ok(())
}

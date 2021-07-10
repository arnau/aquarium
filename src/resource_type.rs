use anyhow::{bail, Result};
use std::fmt;
use std::str::FromStr;

/// Main resource types.
///
/// Auxiliary types such as ServiceAccount are not considered here as they are never represented on their own.
#[derive(Debug, Clone)]
pub enum ResourceType {
    Bulletin,
    Note,
    Person,
    Project,
    Section,
    Settings,
    Sketch,
    Tool,
    Unknown,
}

impl ResourceType {
    /// Peeks the first few characters expecting to find a type declaration.
    pub fn from_hint(text: &str) -> ResourceType {
        // Markdown+Yaml based

        if text.starts_with("---\ntype: bulletin\n") {
            return ResourceType::Bulletin;
        }

        if text.starts_with("---\ntype: note\n") {
            return ResourceType::Note;
        }

        if text.starts_with("---\ntype: project\n") {
            return ResourceType::Project;
        }

        if text.starts_with("---\ntype: section\n") {
            return ResourceType::Section;
        }

        if text.starts_with("---\ntype: sketch\n") {
            return ResourceType::Section;
        }

        if text.starts_with("---\ntype: tool\n") {
            return ResourceType::Tool;
        }

        // TOML based.

        if text.trim().starts_with("type = \"person\"\n") {
            return ResourceType::Person;
        }

        if text.trim().starts_with("type = \"settings\"\n") {
            return ResourceType::Settings;
        }

        ResourceType::Unknown
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ResourceType::*;

        let s = match self {
            Bulletin => "bulletin",
            Note => "note",
            Person => "person",
            Project => "project",
            Section => "section",
            Settings => "settings",
            Sketch => "sketch",
            Tool => "tool",
            Unknown => "unknown",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for ResourceType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ResourceType::*;

        match s {
            "bulletin" => Ok(Bulletin),
            "note" => Ok(Note),
            "person" => Ok(Person),
            "project" => Ok(Project),
            "section" => Ok(Section),
            "settings" => Ok(Settings),
            "sketch" => Ok(Sketch),
            "tool" => Ok(Tool),
            _ => bail!(format!("'{}' is not a known resource type", s)),
        }
    }
}

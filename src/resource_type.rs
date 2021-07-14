use anyhow::{bail, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

lazy_static! {
    static ref HINT_RE: Regex = Regex::new(
        r#"(?x)
        # YAML frontmatter
        ^\s*---\s*\n\s*type:\s*(?P<yaml_hint>\S+)\s*\n
        |
        # TOML
        ^\s*type\s*=\s*"(?P<toml_hint>\S+)"\s*\n
    "#
    )
    .unwrap();
}

/// Main resource types.
///
/// Auxiliary types such as ServiceAccount are not considered here as they are never represented on their own.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Bulletin,
    BulletinStash,
    Entrance,
    Note,
    Person,
    Project,
    Section,
    Settings,
    Sketch,
    Tool,
    Unknown(String),
}

impl ResourceType {
    /// Peeks the first few characters expecting to find a type declaration.
    pub fn from_hint(text: &str) -> Result<Self> {
        if let Some(groups) = HINT_RE.captures(text) {
            if let Some(s) = groups.name("yaml_hint") {
                return ResourceType::from_str(s.as_str());
            }

            if let Some(s) = groups.name("toml_hint") {
                return ResourceType::from_str(s.as_str());
            }
        }

        bail!("hint not found")
    }

    pub fn to_hint(&self) -> String {
        use ResourceType::*;

        match self {
            // TOML
            typ @ (Bulletin | BulletinStash | Person | Settings | Sketch) => {
                format!("type = \"{}\"\n", typ)
            }
            // Yaml frontmatter
            typ @ (Entrance | Note | Project | Section | Tool) => format!("---\ntype: {}\n", typ),
            _ => panic!("no hint for unknown type"),
        }
    }
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ResourceType::*;

        let s = match self {
            Bulletin => "bulletin",
            BulletinStash => "bulletin_stash",
            Entrance => "entrance",
            Note => "note",
            Person => "person",
            Project => "project",
            Section => "section",
            Settings => "settings",
            Sketch => "sketch",
            Tool => "tool",
            Unknown(_) => "unknown",
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
            "bulletin_stash" => Ok(BulletinStash),
            "entrance" => Ok(Entrance),
            "note" => Ok(Note),
            "person" => Ok(Person),
            "project" => Ok(Project),
            "section" => Ok(Section),
            "settings" => Ok(Settings),
            "sketch" => Ok(Sketch),
            "tool" => Ok(Tool),
            unknown => Ok(Unknown(unknown.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_hint() -> Result<()> {
        let blob = r#"---
type: note
id: a-note
"#;
        let actual = ResourceType::from_hint(blob)?;

        assert_eq!(actual, ResourceType::Note);

        Ok(())
    }

    #[test]
    fn person_hint() -> Result<()> {
        let blob = r#"
type = "person"
id = "xxx"
"#;
        let actual = ResourceType::from_hint(blob)?;

        assert_eq!(actual, ResourceType::Person);

        Ok(())
    }

    #[test]
    fn unknown_hint() -> Result<()> {
        let blob = r#"---
type: fox
id: xxx
"#;
        let actual = ResourceType::from_hint(blob)?;

        assert_eq!(actual, ResourceType::Unknown("fox".to_string()));

        Ok(())
    }

    #[test]
    fn hint_not_found() -> Result<()> {
        let blob = r#"---
id: xxx
"#;
        let actual = ResourceType::from_hint(blob);

        assert!(actual.is_err());

        Ok(())
    }
}

use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// A cache strategy express what is available as a storage mechanism.
#[derive(Debug, Clone, PartialEq)]
pub enum Strategy {
    Memory,
    Disk(PathBuf),
}

impl FromStr for Strategy {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            ":memory:" => Ok(Strategy::Memory),
            s => {
                let path = Path::new(s);
                Ok(Strategy::Disk(path.into()))
            }
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Strategy::Memory => ":memory:".to_string(),
            Strategy::Disk(path) => path.display().to_string(),
        };

        write!(f, "{}", s)
    }
}

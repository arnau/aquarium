use anyhow::Result;
use clap::Clap;
use std::fs;
use std::path::PathBuf;

/// Cleans the artefacts created by the build command.
#[derive(Debug, Clap)]
pub struct Cmd {
    /// Cache path. If not provided it won't attempt to remove it.
    #[clap(long, value_name = "path")]
    cache_path: Option<PathBuf>,
    /// The path to the sink to build into.
    #[clap(long, short = 'o', value_name = "path")]
    output_path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<()> {
        if let Some(ref path) = self.cache_path {
            fs::remove_file(path)?;
        }

        fs::remove_dir_all(&self.output_path)?;

        Ok(())
    }
}

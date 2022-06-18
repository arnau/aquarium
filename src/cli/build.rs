use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use crate::cache::{Cache, Strategy};
use crate::feed;
use crate::source;
use crate::zola;

/// Manages bulletins
#[derive(Debug, Parser)]
pub struct Cmd {
    /// Cache path.
    #[clap(long, value_name = "path", default_value = ":memory:")]
    cache_path: Strategy,
    /// The path to the source to build from.
    #[clap(long, short = 'i', value_name = "path")]
    input_path: PathBuf,
    /// The path to the sink to build into.
    #[clap(long, short = 'o', value_name = "path")]
    output_path: PathBuf,
}

impl Cmd {
    pub fn run(&self) -> Result<()> {
        let mut cache = Cache::connect_with_strategy(self.cache_path.clone())?;

        source::read(&self.input_path, &mut cache)?;
        zola::write(&self.output_path.join("content"), &mut cache)?;
        feed::write(&self.output_path.join("static"), &mut cache)?;

        Ok(())
    }
}

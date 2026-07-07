//! Generates the Swift `SharedTypes` package from the core via facet typegen.

use std::path::PathBuf;

use anyhow::Result;
use changes_ffi::Changes;
use clap::Parser;
use crux_core::type_generation::facet::{Config, TypeRegistry};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    output_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let typegen = TypeRegistry::new().register_app::<Changes>()?.build()?;
    let config = Config::builder("SharedTypes", &args.output_dir).build();
    typegen.swift(&config)?;
    Ok(())
}

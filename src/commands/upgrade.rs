use alpm::Alpm;
use clap::Clap;
use std::{cell::RefCell, path::PathBuf};

use crate::{commands::CommandHandler, config::Config};

/// Upgrade operations
#[derive(Clap, Clone)]
pub struct Command {
    /// Do not reinstall up to date packages
    #[clap(long)]
    pub needed: bool,
    /// Skip dependency checks (-dd to skip all checks)
    #[clap(short = 'd', long, parse(from_occurrences))]
    pub nodeps: i32,
    /// Overwrite conflicting files
    #[clap(long, parse(from_os_str))]
    pub overwrite: Vec<PathBuf>,
    /// Mark packages as non-explicitly installed
    #[clap(long, conflicts_with = "asexplicit")]
    pub asdeps: bool,
    /// Mark packages as explicitly installed
    #[clap(long)]
    pub asexplicit: bool,
    /// Ignore a upgrade
    #[clap(long)]
    pub ignore: Vec<String>,
    /// Add a virtual package to satisfy dependencies
    #[clap(long)]
    pub assume_installed: Vec<String>,
    /// Only modify database entries, not package files
    #[clap(long)]
    pub dbonly: bool,
    /// Do not execute the install scriptlet if one exists
    #[clap(long)]
    pub noscriptlet: bool,
    /// print the targets instead of performing the operation
    #[clap(short, long, conflicts_with = "dbonly", conflicts_with = "noscriptlet")]
    pub print: bool,
    /// Specify how the targets should be printed
    #[clap(long)]
    pub print_format: Option<String>,
    /// The files to upgrade
    #[clap(required = true)]
    pub files: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config) {
        todo!("Impl UPGRADE!")
    }
}

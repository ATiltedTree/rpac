use alpm::Alpm;
use clap::Clap;
use std::{cell::RefCell, path::PathBuf};

use crate::{commands::CommandHandler, config::Config};

/// Sync operations
#[derive(Clap, Clone)]
pub struct Command {
    /// Remove old packages from cache (-cc for all)
    #[clap(
        short,
        long,
        parse(from_occurrences),
        conflicts_with = "groups",
        conflicts_with = "info",
        conflicts_with = "list",
        conflicts_with = "refresh",
        conflicts_with = "search",
        conflicts_with = "sysupgrade",
        conflicts_with = "downloadonly"
    )]
    pub clean: i32,
    /// View all members of a groups (-gg to view all groups members)
    #[clap(
        short,
        long,
        parse(from_occurrences),
        conflicts_with = "sysupgrade",
        conflicts_with = "downloadonly"
    )]
    pub groups: i32,
    /// View package information (-ii for extended information)
    #[clap(
        short,
        long,
        parse(from_occurrences),
        conflicts_with = "groups",
        conflicts_with = "list",
        conflicts_with = "search",
        conflicts_with = "sysupgrade",
        conflicts_with = "downloadonly"
    )]
    pub info: i32,
    /// Show all packages in a repo
    #[clap(
        short,
        long,
        conflicts_with = "groups",
        conflicts_with = "sysupgrade",
        conflicts_with = "downloadonly"
    )]
    pub list: Option<String>,
    /// Show less information for query and search
    #[clap(short, long)]
    pub quiet: bool,
    /// Search installed packages for matching strings
    #[clap(
        short,
        long,
        conflicts_with = "groups",
        conflicts_with = "list",
        conflicts_with = "sysupgrade",
        conflicts_with = "downloadonly"
    )]
    pub search: Option<String>,
    /// Upgrade installed packages (-uu enables downgrades)
    #[clap(short = 'u', long, parse(from_occurrences))]
    pub sysupgrade: i32,
    /// Download packages but do not install/upgrade anything
    #[clap(short = 'w', long)]
    pub downloadonly: bool,
    /// Download fresh databases from the server (-yy to force a refresh)
    #[clap(short = 'y', long, parse(from_occurrences))]
    pub refresh: i32,
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
    /// The packages to install
    #[clap()]
    pub packages: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config) {
        todo!("Impl SYNC!")
    }
}

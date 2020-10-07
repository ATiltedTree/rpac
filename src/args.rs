use clap::{AppSettings, Clap};
use std::path::PathBuf;

use crate::commands::*;

#[derive(Clap, Clone)]
#[clap(version, author, about, global_setting = AppSettings::ColoredHelp, setting = AppSettings::GlobalVersion, setting = AppSettings::VersionlessSubcommands)]
pub struct Args {
    /// Set a custom database location
    #[clap(short = 'b', long, parse(from_os_str))]
    pub dbpath: Option<PathBuf>,
    /// Set a custom installation root
    #[clap(short, long, parse(from_os_str))]
    pub root: Option<PathBuf>,
    /// The verbosity
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    /// Set a custom architecture
    #[clap(long)]
    pub arch: Option<String>,
    /// Operate in a mounted guest system (root-only)
    #[clap(long)]
    pub sysroot: bool,
    /// Set a custom package cache location
    #[clap(long, parse(from_os_str))]
    pub cachedir: Option<PathBuf>,
    /// Set a custom hook location
    #[clap(long, parse(from_os_str))]
    pub hookdir: Option<PathBuf>,
    /// Set a custom configuration file
    #[clap(long, parse(from_os_str))]
    pub config: Option<PathBuf>,
    /// Set a custom home directory for GnuPG
    #[clap(long, parse(from_os_str))]
    pub gpgdir: Option<PathBuf>,
    /// Set a custom log file
    #[clap(long, parse(from_os_str))]
    pub logfile: Option<PathBuf>,
    /// Do not ask for confirmation
    #[clap(long)]
    pub noconfirm: bool,
    /// Always ask for confirmation
    #[clap(long)]
    pub confirm: bool,
    /// Use relaxed timeouts for download
    #[clap(long)]
    pub disable_download_timeout: bool,
    /// The operation to do
    #[clap(subcommand)]
    pub command: Command,
}

use alpm::{Alpm, PackageReason, TransFlag};
use clap::Clap;
use std::{cell::RefCell, fs, path::PathBuf};

use crate::{commands::CommandHandler, config::Config};

/// Database operations
#[derive(Clap, Clone)]
pub struct Command {
    /// Mark packages as non-explicitly installed
    #[clap(long, conflicts_with = "asexplicit")]
    pub asdeps: bool,
    /// Mark packages as explicitly installed
    #[clap(long)]
    pub asexplicit: bool,
    /// Test local database for validity (-kk for sync databases)
    #[clap(short = 'k', long, parse(from_occurrences), requires = "packages")]
    pub check: i32,
    /// Suppress output of success messages
    #[clap(short, long)]
    pub quiet: bool,
    /// The packages to query
    pub packages: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config) {
        if self.check >= 1 {
            let handle = alpm_handle.borrow();
            let db_path = handle.dbpath();
            for entry in fs::read_dir(PathBuf::from(db_path).join("local")).unwrap() {
                let entry = entry.unwrap();
                if entry.file_name() == "ALPM_DB_VERSION" {
                    continue;
                } else {
                    assert!(entry.path().join("desc").exists());
                    assert!(entry.path().join("files").exists())
                }
                todo!("Check conflicts");
            }
        }
        if self.asdeps || self.asexplicit {
            let handle = alpm_handle.borrow_mut();
            let reason = if self.asdeps {
                PackageReason::Depend
            } else {
                PackageReason::Explicit
            };
            let _ = handle.trans_init(TransFlag::empty()).unwrap();
            for mut pkg in handle.localdb().pkgs().unwrap() {
                pkg.set_reason(reason).unwrap();
            }
        }
    }
}

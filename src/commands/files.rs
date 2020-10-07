use alpm::Alpm;
use clap::Clap;
use regex::RegexSet;
use std::cell::RefCell;

use crate::{commands::CommandHandler, config::Config};

/// Files operations
#[derive(Clap, Clone)]
pub struct Command {
    /// List the files owned by the queried package
    #[clap(short, long)]
    pub list: bool,
    /// Download fresh package databases (-yy to force a refresh)
    #[clap(short = 'y', long, parse(from_occurrences))]
    pub refresh: i32,
    /// Show less information for query and search
    #[clap(short, long)]
    pub quiet: bool,
    /// The targets to search for
    #[clap(required_unless_present = "refresh")]
    pub targets: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, _config: Config) {
        if self.refresh >= 1 {
            let mut handle = alpm_handle.borrow_mut();
            for mut db in handle.syncdbs_mut() {
                db.update(!matches!(self.refresh, 1)).unwrap();
            }
        }

        if self.list {
            let handle = alpm_handle.borrow();
            for package_name in &self.targets {
                let packages = handle.syncdbs().filter_map(|db| db.pkg(package_name).ok());
                for package in packages {
                    for file in package.files().files() {
                        println!("{}: {}", package.name(), file.name());
                    }
                }
            }
        } else if !self.targets.is_empty() {
            let handle = alpm_handle.borrow();
            let regex = RegexSet::new(&self.targets).unwrap();
            for db in handle.syncdbs() {
                for pkg in db.pkgs().unwrap() {
                    let file_list = pkg.files();

                    let found: Vec<&str> = file_list
                        .files()
                        .iter()
                        .filter_map(|val| {
                            if regex.is_match(val.name()) {
                                Some(val.name())
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !found.is_empty() {
                        println!(
                            "{}/{} {} {}",
                            db.name(),
                            pkg.name(),
                            pkg.version(),
                            handle
                                .localdb()
                                .pkg(pkg.name())
                                .map_or_else(|_| "", |_| "[installed]")
                        );
                        for file in found {
                            println!("  {}", file);
                        }
                    }
                }
            }
        } else {
            unreachable!()
        }
    }
}

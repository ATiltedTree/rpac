use alpm::{Alpm, PrepareReturn, TransFlag};
use clap::Clap;
use dialoguer::Confirm;
use std::cell::RefCell;

use crate::{commands::CommandHandler, config::Config};

/// Remove operations
#[derive(Clap, Clone)]
pub struct Command {
    /// Skip dependency checks (-dd to skip all checks)
    #[clap(short = 'd', long, parse(from_occurrences))]
    pub nodeps: i32,
    /// Remove packages and all dependent packages
    #[clap(short, long)]
    pub cascade: bool,
    /// Remove packages and their configuration files
    #[clap(short, long, conflicts_with = "print", conflicts_with = "dbonly")]
    pub nosave: bool,
    /// Remove unnecessary dependencies
    #[clap(short = 's', long)]
    pub recurse: bool,
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
    /// The packages to remove
    #[clap(required = true)]
    pub packages: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config) {
        alpm_handle.borrow().trans_init(TransFlag::empty()).unwrap();
        for pkg in &self.packages {
            let handle = alpm_handle.borrow();
            if let Ok(pkg) = handle.localdb().pkg(pkg) {
                handle.trans_remove_pkg(pkg).unwrap();
            } else if let Ok(group) = handle.localdb().group(pkg) {
                for member in group.packages() {
                    handle.trans_remove_pkg(member).unwrap();
                }
            }
        }
        if let Err(err) = alpm_handle.borrow_mut().trans_prepare() {
            match err {
                (err, _) => match err {
                    PrepareReturn::UnsatisfiedDeps(deps) => {
                        for dep in deps {
                            println!(
                                "removing {} breaks {}",
                                dep.causing_pkg().unwrap(),
                                dep.depend()
                            );
                        }
                    }
                    _ => {}
                },
            }
        }

        if Confirm::new()
            .with_prompt("Do you want to remove these packages?")
            .interact()
            .unwrap()
        {
            alpm_handle.borrow_mut().trans_commit().unwrap();
        }
    }
}

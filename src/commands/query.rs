use alpm::{Alpm, AlpmList, Db, Package, PackageReason};
use clap::Clap;
use path_absolutize::Absolutize;
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use crate::{commands::CommandHandler, config::Config};

/// Query operations
#[derive(Clap, Clone)]
pub struct Command {
    /// View the changelog
    #[clap(short, long)]
    pub changelog: bool,
    /// List packages installed as dependencies [filter]
    #[clap(short, long, conflicts_with = "explicit")]
    pub deps: bool,
    /// List packages explicitly installed [filter]
    #[clap(short, long)]
    pub explicit: bool,
    /// View all members of a group
    #[clap(
        short,
        long,
        conflicts_with = "changelog",
        conflicts_with = "check",
        conflicts_with = "info",
        conflicts_with = "list"
    )]
    pub groups: bool,
    /// View package information (-ii for backup files)
    #[clap(short, long, parse(from_occurrences))]
    pub info: isize,
    /// Check that package files exist (-kk for file properties)
    #[clap(short = 'k', long, parse(from_occurrences))]
    pub check: isize,
    /// List the files owned by the package
    #[clap(short, long)]
    pub list: bool,
    /// List installed packages not found in sync db(s) [filter]
    #[clap(short = 'm', long)]
    pub foreign: bool,
    /// List installed packages only found in sync db(s) [filter]
    #[clap(short, long, conflicts_with = "foreign")]
    pub native: bool,
    /// Query the packages that owns <file>
    #[clap(
        short,
        long,
        conflicts_with = "groups",
        conflicts_with = "changelog",
        conflicts_with = "check",
        conflicts_with = "info",
        conflicts_with = "list",
        conflicts_with = "deps",
        conflicts_with = "explicit",
        conflicts_with = "upgrade",
        conflicts_with = "unrequired",
        conflicts_with = "native",
        conflicts_with = "foreign"
    )]
    pub owns: bool,
    /// Query a package file instead of the database
    #[clap(
        short = 'p',
        long,
        parse(from_os_str),
        conflicts_with = "groups",
        conflicts_with = "search",
        conflicts_with = "owns"
    )]
    pub file: Option<PathBuf>,
    /// Show less information for query and search
    #[clap(short, long)]
    pub quiet: bool,
    /// Search installed packages for matching strings
    #[clap(
        short,
        long,
        conflicts_with = "groups",
        conflicts_with = "owns",
        conflicts_with = "changelog",
        conflicts_with = "check",
        conflicts_with = "info",
        conflicts_with = "list",
        conflicts_with = "deps",
        conflicts_with = "explicit",
        conflicts_with = "upgrade",
        conflicts_with = "unrequired",
        conflicts_with = "native",
        conflicts_with = "foreign"
    )]
    pub search: bool,
    /// List packages not (optionally) required by any package (-tt to ignore optdepends) [filter]
    #[clap(short = 't', long, parse(from_occurrences))]
    pub unrequired: isize,
    /// List outdated packages [filter]
    #[clap(short, long)]
    pub upgrade: bool,
    /// The packages to query
    #[clap()]
    pub targets: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config) {
        match self {
            _search if self.search => {
                let handle = alpm_handle.borrow();
                let packages = if self.targets.is_empty() {
                    handle.localdb().pkgs()
                } else {
                    handle.localdb().search(&self.targets)
                };
                for package in packages.unwrap() {
                    if self.quiet {
                        println!("{}", package.name())
                    } else {
                        print!(
                            "{}/{} {}",
                            handle.localdb().name(),
                            package.name(),
                            package.version()
                        );
                        let groups = package.groups();
                        if groups.is_empty() {
                            println!();
                        } else {
                            println!(" ({})", groups.collect::<Vec<&str>>().join(" "));
                        }

                        println!("    {}", package.desc().unwrap());
                    }
                }
            }
            _groups if self.groups => {
                let handle = alpm_handle.borrow();
                if self.targets.is_empty() {
                    let localdb = handle.localdb();
                    // groups() actually returns all packages causing alpm to sigsegv because we
                    // tell it that the data is actually a group allthough it is a packages.
                    // TODO: Create issue in alpm crate
                    let groups = localdb
                        .groups()
                        .unwrap()
                        .map(|group| (group.packages(), group.name()));

                    for (packages, group) in groups {
                        for package in packages
                            .filter(|package| self.filter_package(package, handle.syncdbs()))
                        {
                            println!("{} {}", group, package.name())
                        }
                    }
                } else {
                    let groups = self
                        .targets
                        .clone()
                        .into_iter()
                        .filter_map(|target| handle.localdb().group(target).ok())
                        .map(|group| (group.packages(), group.name()));

                    for (packages, group) in groups {
                        for package in packages
                            .filter(|package| self.filter_package(package, handle.syncdbs()))
                        {
                            if self.quiet {
                                println!("{}", package.name())
                            } else {
                                println!("{} {}", group, package.name())
                            }
                        }
                    }
                }
            }
            _all if self.targets.is_empty() => {
                if self.file.is_some() || self.owns {
                    eprintln!("NO");
                } else {
                    let handle = alpm_handle.borrow();
                    for package in handle
                        .localdb()
                        .pkgs()
                        .unwrap()
                        .filter(|pkg| self.filter_package(pkg, handle.syncdbs()))
                    {
                        // TODO: Implement full info print
                        println!("{:?}", package.name())
                    }
                }
            }
            _owns if self.owns => {
                let handle = alpm_handle.borrow();
                let files = self.targets.clone().into_iter().filter_map(|target| {
                    let path = PathBuf::from(target);

                    if let Ok(resolved) = path.absolutize() {
                        if resolved.exists() {
                            Some(resolved.into_owned())
                        } else {
                            resolve_path(path)
                        }
                    } else {
                        resolve_path(path)
                    }
                });

                for file in files {
                    for package in handle.localdb().pkgs().unwrap() {
                        let is_owned = package
                            .files()
                            .files()
                            .iter()
                            .map(|file| {
                                let mut file = file.name().to_string();
                                file.insert(0, '/');
                                PathBuf::from(file)
                            })
                            .any(|package_file| package_file == file);
                        if is_owned {
                            println!(
                                "{} is owned by {} {}",
                                file.display(),
                                package.name(),
                                package.version()
                            )
                        }
                    }
                }
            }
            _ => {
                // Query locale db with named
            }
        }

        // Locality and upgrade cause check_syncdb?
    }
}

impl Command {
    fn filter_package(&self, package: &Package, sync_dbs: AlpmList<Db>) -> bool {
        if self.explicit && package.reason() != PackageReason::Explicit
            || self.deps && package.reason() != PackageReason::Depend
        {
            false
        } else if let Some(locality) = PackageLocality::new(self.native, self.foreign) {
            locality == compute_locality(package, sync_dbs)
        } else if self.unrequired >= 1 {
            if self.unrequired == 1 {
                package.required_by().is_empty() && package.optional_for().is_empty()
            } else {
                package.required_by().is_empty()
            }
        } else {
            !(self.upgrade && package.sync_new_version(sync_dbs).is_some())
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum PackageLocality {
    Native,
    Foreign,
}

impl PackageLocality {
    fn new(native: bool, foreign: bool) -> Option<Self> {
        match (native, foreign) {
            (true, false) => Some(PackageLocality::Native),
            (false, true) => Some(PackageLocality::Foreign),
            (true, true) => unreachable!(),
            (false, false) => None,
        }
    }
}

fn compute_locality(pkg: &Package, sync_dbs: AlpmList<Db>) -> PackageLocality {
    for sync_db in sync_dbs {
        if sync_db.pkg(pkg.name()).is_ok() {
            return PackageLocality::Native;
        }
    }
    PackageLocality::Foreign
}

fn resolve_path<P: AsRef<Path>>(filename: P) -> Option<PathBuf> {
    let path_env = std::env::var_os("PATH")?;
    for path in path_env.to_string_lossy().split(':').map(PathBuf::from) {
        let resolved = path.join(&filename);
        if resolved.exists() {
            return Some(resolved);
        }
    }

    None
}

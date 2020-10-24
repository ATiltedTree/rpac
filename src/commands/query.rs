use alpm::{Alpm, AlpmList, Db, Package, PackageReason, Usage};
use clap::Clap;
use path_absolutize::Absolutize;
use std::{
    cell::RefCell,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{commands::CommandHandler, config::Config, utils::{Join,EnumFormatter,check}};

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
    #[clap(short, long = "upgrades")]
    pub upgrade: bool,
    /// The packages to query
    #[clap()]
    pub targets: Vec<String>,
}

impl CommandHandler for Command {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config) {
        if self.foreign || self.native || self.upgrade {
            let handle = alpm_handle.borrow();
            let mut should_return = false;
            for db in handle.syncdbs() {
                if let Err(err) = db.is_valid() {
                    println!("database '{}' is not valid ({})",db.name(),err);
                    should_return = true;
                }
            }
            if should_return {
                return;
            }
        }
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
                        self.display_package(&package, &handle)
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
                            .map(|file| PathBuf::from(format!("{}{}", handle.root(), file.name())))
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
                let handle = alpm_handle.borrow();
                let packages = self
                    .targets
                    .iter()
                    .filter_map(|target| handle.localdb().pkg(target).ok())
                    .filter(|pkg| self.filter_package(pkg,handle.syncdbs()));
                for package in packages {
                    self.display_package(&package, &handle)
                }
            }
        }

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

    fn display_package(&self, package: &Package, handle: &Alpm) {
        if self.info > 0 {
            println!("Name : {}", package.name());
            println!("Version : {}", package.version());
            println!(
                "Description : {}",
                package.desc().map_or("None", |desc| desc)
            );
            println!("Architecture : {}", package.arch().map_or("None", |a| a));
            println!("URL : {}", package.url().map_or("None", |p| p));
            println!(
                "Licenses : {}",
                package.licenses().join(" ")
            );
            println!("Groups : {}", package.groups().join(" "));
            println!(
                "Provides : {}",
                package.provides().join(" ")
            );
            println!(
                "Depends On : {}",
                package.depends().join(" ")
            );
            println!(
                "Optional Deps : {}",
                package.optdepends().join("\n")
            );
            println!(
                "Required By : {}",
                package.required_by().join(" ")
            );
            println!(
                "Optional For : {}",
                package.optional_for().join(" ")
            );
            println!(
                "Conflicts With : {}",
                package.conflicts().join(" ")
            );
            println!(
                "Replaces : {}",
                package.replaces().join(" ")
            );
            println!("Installed Size : {}", package.size());
            println!("Packager : {}", package.packager().map_or("None", |p| p));
            println!("Build Date : {}", package.build_date());
            if let Some(date) = package.install_date() {
                println!("Install Date : {}", date);
            }
            println!("Install Reason : {}",EnumFormatter::from(package.reason()));
            println!("Install Script : {}", package.has_scriptlet());
            println!("Validated By : {}", EnumFormatter::from(package.validation()));
            println!();
        }

        if self.list {
            for file in package.files().files() {
                if !self.quiet {
                    print!("{} ", package.name())
                }
                println!("{}{}", handle.root(), file.name())
            }
        }

        if self.changelog {
            if let Ok(mut changelog) = package.changelog() {
                println!("Changelog for {}:", package.name());
                let mut changelog_string = String::new();
                changelog.read_to_string(&mut changelog_string).unwrap();
                println!("{}", changelog_string);
            }
        }
        if self.check > 0 {
            let errors = if self.check == 1 {
                check(false, package, &handle)
            } else {
                check(true, package, &handle)
            };

            if errors != 0 {
                println!("{}: {} total files",package.name(), package.files().files().len());
                println!("{} missing files",errors);
            }
        }

        if !self.info > 0 && !self.list && !self.changelog && !self.check > 0 {
            if !self.quiet {
                print!("{} {}", package.name(), package.version());
                if self.upgrade {
                    print!(
                        " -> {}",
                        package
                            .sync_new_version(handle.syncdbs())
                            .unwrap()
                            .version()
                    );

                    if package.should_ignore()
                        || !package
                            .db()
                            .unwrap()
                            .usage()
                            .unwrap()
                            .contains(Usage::UPGRADE)
                    {
                        print!(" [ignored]");
                    }
                }
                println!();
            } else {
                println!("{}", package.name());
            }
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

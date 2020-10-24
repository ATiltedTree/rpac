use alpm::{Alpm, Depend, Package, PackageReason, PackageValidation,AlpmList};
use std::{fmt::Write, path::PathBuf};

use crate::callbacks::*;

pub fn register_cbs(handle: &Alpm) {
    init(handle);
    QuestionCallback::register();
    LogCallback::register();
    DlCallback::register();
    EventCallback::register();
    ProgressCallback::register();
}


pub trait Join : Iterator  {
    fn join(&mut self, sep: &str) -> String
        where Self::Item: std::fmt::Display
    {
        match self.next() {
            None => "None".to_string(),
            Some(first_elt) => {
                // estimate lower bound of capacity needed
                let (lower, _) = self.size_hint();
                let mut result = String::with_capacity(sep.len() * lower);
                write!(&mut result, "{}", first_elt).unwrap();
                for elt in self {
                    result.push_str(sep);
                    write!(&mut result, "{}", elt).unwrap();
                }
                result
            }
        }
    }
}

impl<T: ?Sized> Join for T where T: Iterator  { }

pub struct EnumFormatter(String);

impl From<PackageReason> for EnumFormatter {
    fn from(reson: PackageReason) -> Self {
        match reson {
            PackageReason::Explicit => Self("Explicit".to_string()),
            PackageReason::Depend => Self("Dependency".to_string()),
        }
    }
}

impl From<PackageValidation> for EnumFormatter {
    fn from(validation: PackageValidation) -> Self {
        let mut me = Vec::new();

        if validation.contains(PackageValidation::MD5SUM) {
            me.push("MD5");
        }

        if validation.contains(PackageValidation::SHA256SUM) {
            me.push("SHA256");
        }

        if validation.contains(PackageValidation::SIGNATURE) {
            me.push("Signature")
        }

        if validation.contains(PackageValidation::NONE) {
            me.push("None")
        }

        Self(me.join(" "))
    }
}

impl std::fmt::Display for EnumFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

pub fn check(full: bool, package: &Package, handle: &Alpm) -> usize {
    let root = PathBuf::from(handle.root());
    let mut errors = 0;

    if full {
        todo!("Implement Full check using mtree")
    } else {
        for file in package.files().files() {
            let path = root.join(file.name());
            if path.exists() {
                if file.name().ends_with('/') != path.is_dir() {
                    eprintln!(
                        "{}: {} (File type mismatch)",
                        package.name(),
                        path.display()
                    );
                    errors += 1;
                }
            } else {
                errors += 1;
            }
        }
    }
    errors
}

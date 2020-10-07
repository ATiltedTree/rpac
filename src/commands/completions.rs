use alpm::Alpm;
use clap::Clap;
use std::cell::RefCell;

use crate::{commands::CommandHandler, config::Config};

/// Auto-generated completions
#[derive(Clap, Clone)]
pub struct Command {
    #[clap(env("SHELL"))]
    pub shell: String,
}

impl CommandHandler for Command {
    fn handle(&self, _alpm_handle: RefCell<Alpm>, _config: Config) {}
}

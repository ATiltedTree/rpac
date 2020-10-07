mod completions;
mod database;
mod files;
mod query;
mod remove;
mod sync;
mod upgrade;

use alpm::Alpm;
use clap::Clap;
use enum_dispatch::enum_dispatch;
use std::cell::RefCell;

use crate::config::Config;

#[enum_dispatch]
#[derive(Clap, Clone)]
pub enum Command {
    #[clap(long_flag = "database", short_flag = 'D')]
    Database(database::Command),
    #[clap(long_flag = "files", short_flag = 'F')]
    Files(files::Command),
    #[clap(long_flag = "query", short_flag = 'Q')]
    Query(query::Command),
    #[clap(long_flag = "remove", short_flag = 'R')]
    Remove(remove::Command),
    #[clap(long_flag = "sync", short_flag = 'S')]
    Sync(sync::Command),
    #[clap(long_flag = "upgrade", short_flag = 'U')]
    Upgrade(upgrade::Command),
    #[clap(long_flag = "completions")]
    Completions(completions::Command),
}

#[enum_dispatch(Command)]
pub trait CommandHandler {
    fn handle(&self, alpm_handle: RefCell<Alpm>, config: Config);
}

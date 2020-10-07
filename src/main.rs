mod args;
mod callbacks;
mod commands;
mod config;
mod utils;

use {
    crate::{args::Args, commands::CommandHandler, config::Config},
    alpm::{Alpm, SigLevel},
    clap::Clap,
    std::{cell::RefCell, fs},
};

const DEFAULT_CONFIG_PATH: &str = "/etc/rpac.toml";

fn main() {
    env_logger::init();
    ctrlc::set_handler(|| panic!("Caught CTRL+C! Unwinding...")).unwrap();
    let opts: Args = args::Args::parse();
    let config: Config = {
        let config_path = opts
            .config
            .unwrap_or_else(|| DEFAULT_CONFIG_PATH.parse().unwrap());
        let data = match fs::read_to_string(config_path.to_owned()) {
            Ok(data) => data,
            Err(err) => {
                eprintln!(
                    "Config was not found at {:?}: {:?}. Using defaults!",
                    config_path, err
                );
                "".to_string()
            }
        };

        toml::from_str(data.as_str()).expect("Config parse error!")
    };
    let mut handle = Alpm::new(
        config.paths.root.to_str().unwrap(),
        config.paths.database.to_str().unwrap(),
    )
    .expect("Could not obtain database lock");
    if let commands::Command::Files(_) = opts.command {
        handle.set_dbext(".files");
    } else {
        handle.set_dbext(".db");
    }
    for db in &config.databases {
        let registered_db = handle
            .register_syncdb_mut(db.name.clone(), SigLevel::empty())
            .unwrap();
        let servers = db
            .servers
            .iter()
            .map(|server| server.replace("$repo", db.name.as_str()))
            .map(|server| server.replace("$arch", config.arch.as_str()))
            .collect::<Vec<String>>();

        for server in servers {
            registered_db.add_server(server).unwrap();
        }
    }
    utils::register_cbs(&handle);
    //opts.command.handle(RefCell::new(handle), config);
    //opts.command.handle(RefCell::new(handle), config);
    opts.command.handle(RefCell::new(handle), config);
}

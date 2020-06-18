use clap::{Clap,};

#[derive(Clap)]
#[clap(version, author, about)]
struct Opts {
    /// The verbosity
    #[clap(short, long, parse(from_occurrences))] // Make global once supported
    verbose: i32,
    /// The operation to do
    #[clap(subcommand)]
    command: Command
}

/// The operation to do
#[derive(Clap)]
enum Command {
    Install(Install),
    Completions(Completions)
}

/// Install packages
#[derive(Clap)]
struct Install {
    packages: Vec<String>
}

/// Auto-generated completions
#[derive(Clap)]
struct Completions {
    #[clap(env("SHELL"))]
    shell: String
}

fn main() {
    let l: Opts = Opts::parse();
    match l.command {
        Command::Install(i) => {
            if l.verbose > 0 {
                println!("VERBOSE");
            }
            println!("Packages to install: {}", i.packages.join(" "))
        }
        Command::Completions(command) => {
            // Not possible yet
        }
    }
}

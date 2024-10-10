use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Network-wide config file location
    #[arg(short, long)]
    network_conf: PathBuf,

    /// Server config file location
    #[arg(short, long)]
    server_conf: PathBuf,

    /// Run in foreground without daemonising
    #[arg(short, long)]
    foreground: bool,
}

/// Main entry point.
///
/// Because the tokio runtime can't survive forking, `main()` loads the application
/// configs (in order to report as many errors as possible before daemonising), daemonises,
/// initialises the tokio runtime, and begins the async entry point [`sable_main`].
pub fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    sable_server::run::run_server::<sable_history::HistoryServer>(
        args.server_conf,
        args.network_conf,
        args.foreground,
        None,
        None::<PathBuf>,
    )
}

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab")]
struct Opts {
    /// Network-wide config file location
    #[structopt(short, long)]
    network_conf: PathBuf,

    /// Server config file location
    #[structopt(short, long)]
    server_conf: PathBuf,

    /// Run in foreground without daemonising
    #[structopt(short, long)]
    foreground: bool,
}

/// Main entry point.
///
/// Because the tokio runtime can't survive forking, `main()` loads the application
/// configs (in order to report as many errors as possible before daemonising), daemonises,
/// initialises the tokio runtime, and begins the async entry point [`sable_main`].
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::from_args();

    sable_server::run::run_server::<
        sable_services::ServicesServer<sable_services::database::jsonfile::JsonDatabase>,
    >(
        opts.server_conf,
        opts.network_conf,
        opts.foreground,
        None,
        None::<PathBuf>,
    )
}

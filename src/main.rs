mod scanner;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all connected USB storage devices
    ListDevices,
    /// Scan a new device
    Scan { path: String },
    /// Query a previously indexed device (Not yet implemented)
    Query,
    /// Manage the background indexer service (Not yet implemented)
    Service,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListDevices => {
            scanner::list_storage_devices();
        }
        Commands::Scan { path } => {
            scanner::scan_directory(path);
        }
        Commands::Query => {
            println!("'query' is not yet implemented.");
        }
        Commands::Service => {
            println!("'service' is not yet implemented.");
        }
    }
}
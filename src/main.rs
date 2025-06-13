use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gifmeta", version, about = "Inspect and edit GIF metadata")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show metadata
    Info {
        /// Path to the GIF file
        #[arg()]
        path: PathBuf,
    },

    /// Set a fixed frame delay (in 10ms units)
    SetDelay {
        #[arg()]
        path: PathBuf,
        #[arg()]
        delay: u16,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Set loop count
    SetLoop {
        #[arg()]
        path: PathBuf,
        #[arg()]
        count: u16,
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { path } => gifmeta::print_info(&path),
        Commands::SetDelay { path, delay, output } => gifmeta::set_frame_delay(&path, delay, output),
        Commands::SetLoop { path, count, output } => gifmeta::set_loop_count(&path, count, output),
    }
}

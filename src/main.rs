use clap::{Parser, Subcommand};
use std::path::PathBuf;
pub mod utils;

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

        /// Show per-frame delay information
        #[arg(long)]
        show_frames: bool,
    },

    /// Modify GIF file
    Mod {
        #[arg()]
        input: PathBuf,

        #[arg(long)]
        loop_count: Option<u16>,

        #[arg(long)]
        delay: Option<u16>,

        #[arg(long)]
        delays: Option<String>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { path, show_frames } => {
            let _ = gifmeta::get_metadata(&path, show_frames);
        }
        Commands::Mod {
            input,
            loop_count,
            delay,
            delays,
            output,
        } => {
            let delays_map = delays
                .as_ref()
                .map(|s| utils::parse_csv::parse_keyval_csv(s))
                .transpose()
                .unwrap_or(None);
            let _ = gifmeta::mod_gif(&input, output, loop_count, delay, delays_map);
        }
    }
}

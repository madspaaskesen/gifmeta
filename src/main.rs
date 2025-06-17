use clap::{Parser, Subcommand};
use std::path::PathBuf;
pub mod parse_csv;

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

    /// Get loop count
    GetLoop {
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

    /// Show delay for each frame
    ShowFrameDelays { path: PathBuf },

    /// Set delays for specific frames
    SetFrameDelay {
        #[arg(long)]
        frame_numbers: String,
        #[arg(long)]
        delay_values: String,
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { path } => {
            let _ = gifmeta::get_metadata(&path);
        }
        Commands::GetLoop { path } => {
            let _ = gifmeta::get_loop_count(&path);
        }
        Commands::SetDelay {
            path,
            delay,
            output,
        } => gifmeta::set_frame_delay(&path, delay, output),
        Commands::SetLoop {
            path,
            count,
            output,
        } => gifmeta::set_loop_count(&path, count, output),
        Commands::ShowFrameDelays { path } => {
            let _ = gifmeta::show_frame_delays(&path);
        }
        Commands::SetFrameDelay {
            frame_numbers,
            delay_values,
            input,
            output,
        } => {
            let frames = parse_csv::parse_csv::<usize>(&frame_numbers).unwrap();
            let delays = parse_csv::parse_csv::<u16>(&delay_values).unwrap();
            let _ = gifmeta::set_selected_frame_delays(&input, frames, delays, output);
        }
    }
}

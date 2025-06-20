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
        #[arg(value_name = "INPUT")]
        path: PathBuf,

        /// Show per-frame delay information
        #[arg(long)]
        show_frames: bool,

        /// Get output as json
        #[arg(long)]
        json: bool, // ← Add this
    },

    /// Modify GIF file
    Mod {
        #[arg(value_name = "INPUT")]
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
        Commands::Info {
            path,
            show_frames,
            json,
        } => {
            let meta_result = gifmeta::get_metadata(&path, show_frames);
            let meta_data = meta_result.ok();

            if let Some(meta) = meta_data {
                if json {
                    let json_str = serde_json::to_string_pretty(&meta).unwrap();
                    println!("{}", json_str);
                } else {
                    println!("✅ Metadata for : {}\n", path.display());
                    println!("🖼️ Dimensions   : {} × {}", meta.width, meta.height);
                    println!("🖼️ Frame count  : {}", meta.frame_count);
                    println!("⏱️ Duration     : {} centiseconds", meta.total_duration_cs);
                    println!("🔄Loop         : {:?}", meta.loop_count);
                    println!("🎨Has palette  : {:?}", meta.has_global_palette);
                    println!("🎨Palette size : {:?}", meta.global_palette_size.unwrap());
                    println!("🎨Transparency : {:?}", meta.uses_transparency);
                    if show_frames {
                        println!("\n🧩Frame delays :");
                        for frame in &meta.frames {
                            println!(
                                "  • Frame {:>3}: {:>4} cs{}",
                                frame.index,
                                frame.delay_cs,
                                match frame.transparent_index {
                                    Some(idx) => format!(" (transparent index: {})", idx),
                                    None => "".to_string(),
                                }
                            );
                        }
                    }
                }
            }
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

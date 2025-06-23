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
        /// Path to the GIF file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Loop count, 0 infinite, else how many times to play.
        #[arg(long)]
        loop_count: Option<u16>,

        /// Delay for all frame
        #[arg(long)]
        delay: Option<u16>,

        /// Delays example 1=10,5=20
        #[arg(long)]
        delays: Option<String>,

        /// Output Path to the GIF file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Preview a single frame of a GIF file as PNG or base64.
    Preview {
        /// Path to the GIF file
        #[arg(value_name = "INPUT")]
        input: PathBuf,

        /// Frame index to extract (0-based)
        #[arg(long, value_name = "INDEX")]
        frame: Option<u16>,

        /// Output the frame as a base64-encoded PNG string (prints to stdout)
        #[arg(long)]
        as_base64: bool,

        /// Path to save the extracted PNG frame (e.g. frame0.png)
        #[arg(short, long, value_name = "FILE")]
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
        Commands::Preview {
            input,
            frame,
            as_base64,
            output,
        } => {
            let frame_index = frame.unwrap_or(0) as usize;
            match gifmeta::get_frame_image(input.to_string_lossy().to_string(), frame_index) {
                Ok(png_bytes) => {
                    if as_base64 {
                        let encoded = base64::encode(png_bytes);
                        println!("{}", encoded);
                    } else if let Some(out_path) = output {
                        std::fs::write(&out_path, png_bytes).expect("Failed to write output PNG");
                        println!("Frame {} written to {}", frame_index, out_path.display());
                    } else {
                        eprintln!("Specify either --as-base64 or --output <path>");
                    }
                }
                Err(e) => {
                    eprintln!("Error extracting frame: {}", e);
                }
            }
        }
    }
}

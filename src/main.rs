use base64::engine::Engine;
use clap::{Parser, Subcommand};
use std::path::PathBuf; // Import the Engine trait for encode()

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

        /// How many times the GIF plays (1 = once, 2 = twice, …). Use --infinite to loop forever.
        #[arg(long, value_name = "TIMES", conflicts_with = "infinite")]
        loop_count: Option<u16>,

        /// Loop forever
        #[arg(long)]
        infinite: bool,

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

        /// Raw view: only the pixels stored in this frame (no earlier frames composited).
        /// Default is the "pretty" view — the frame as it looks when the GIF plays.
        #[arg(long)]
        raw: bool,

        /// Path to save the extracted PNG frame (e.g. frame0.png)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Export ALL frames as PNG files into --output-dir (frame_000.png, frame_001.png, …)
        #[arg(long, requires = "output_dir", conflicts_with_all = ["frame", "as_base64", "output"])]
        all: bool,

        /// Folder for --all (created if missing)
        #[arg(long, value_name = "DIR")]
        output_dir: Option<PathBuf>,
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
                    println!(
                        "🔄Loop         : {}",
                        gifmeta::LoopMode::from_count(meta.loop_count)
                    );
                    println!("🎨Has palette  : {:?}", meta.has_global_palette);
                    println!("🎨Palette size : {}", meta.global_palette_size.unwrap_or(0));
                    println!("🎨Transparency : {:?}", meta.uses_transparency);
                    println!(
                        "🧩Optimized    : {} ({} keyframes / {} frames)",
                        if meta.is_delta_optimized { "yes" } else { "no" },
                        meta.keyframe_count,
                        meta.frame_count
                    );
                    if show_frames {
                        println!("\n🧩Frames (🔑 = keyframe, Δ = delta) :");
                        for frame in &meta.frames {
                            println!(
                                "  {} Frame {:>3}: {:>4} cs  rect {},{} {}×{}  dispose {:?}{}",
                                if frame.is_keyframe { "🔑" } else { "Δ " },
                                frame.index,
                                frame.delay_cs,
                                frame.left,
                                frame.top,
                                frame.width,
                                frame.height,
                                frame.disposal,
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
            infinite,
            delay,
            delays,
            output,
        } => {
            let delays_map = match delays
                .as_deref()
                .map(gifmeta::utils::parse_csv::parse_keyval_csv)
            {
                Some(Err(e)) => {
                    eprintln!("❌ --delays: {}", e);
                    std::process::exit(2);
                }
                Some(Ok(m)) => Some(m),
                None => None,
            };
            let loop_mode = match (infinite, loop_count) {
                (true, _) => Some(gifmeta::LoopMode::Infinite),
                (false, Some(0)) => {
                    eprintln!("⚠️  --loop-count 0 is deprecated, use --infinite");
                    Some(gifmeta::LoopMode::Infinite)
                }
                (false, Some(n)) => Some(gifmeta::LoopMode::Times(n)),
                (false, None) => None,
            };
            let opts = gifmeta::ModOptions {
                output,
                loop_mode,
                delay_all: delay,
                delays: delays_map,
            };
            match gifmeta::modify_gif(&input, opts) {
                Ok(out) => println!("✅ Modifications applied → {}", out.display()),
                Err(e) => {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Preview {
            input,
            frame,
            as_base64,
            raw,
            output,
            all,
            output_dir,
        } => {
            let frame_index = frame.unwrap_or(0) as usize;
            let mode = if raw {
                gifmeta::PreviewMode::Raw
            } else {
                gifmeta::PreviewMode::Composited
            };
            if all {
                let dir = output_dir.expect("clap enforces --output-dir");
                if let Err(e) = export_all_frames(&input, &dir, mode) {
                    eprintln!("❌ {}", e);
                    std::process::exit(1);
                }
                return;
            }
            match gifmeta::get_frame_image_with_mode(
                input.to_string_lossy().to_string(),
                frame_index,
                mode,
            ) {
                Ok(png_bytes) => {
                    if as_base64 {
                        let encoded = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                        println!("data:image/png;base64,{}", encoded);
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

/// Writes every frame of `input` as `frame_000.png`, `frame_001.png`, … into `dir`.
fn export_all_frames(
    input: &std::path::Path,
    dir: &std::path::Path,
    mode: gifmeta::PreviewMode,
) -> Result<(), String> {
    let count = gifmeta::get_metadata(&input.to_path_buf(), false)?.frame_count as usize;
    let width = count.saturating_sub(1).to_string().len().max(3);
    std::fs::create_dir_all(dir).map_err(|e| format!("Could not create {}: {e}", dir.display()))?;
    for frame in gifmeta::FrameRenderer::open(input, mode)? {
        let frame = frame?;
        let png = gifmeta::utils::extract_frame_as_png::encode_png(&frame.image)?;
        let path = dir.join(format!("frame_{:0width$}.png", frame.index, width = width));
        std::fs::write(&path, png)
            .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    }
    println!("✅ {} frames written to {}", count, dir.display());
    Ok(())
}

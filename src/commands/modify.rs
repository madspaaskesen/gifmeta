// src/commands/modify.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use gif::{DecodeOptions, Encoder, Repeat};
use uuid::Uuid;

use crate::commands::info;

//use crate::commands::info;
//use crate::gifmeta_structs::{FrameMeta, GifMetadata};

/// Applies modifications to a GIF file, including loop count, global delay, and specific frame delays.
///
/// If `output` is None, writes to a temp file instead of overwriting input.
///
/// # Arguments
/// - `input`: Path to the input `.gif`
/// - `loop_count`: Optional new loop count (0 = infinite)
/// - `global_delay`: Optional new delay for all frames (centiseconds)
/// - `frame_delays`: Optional list of (index, delay) overrides
/// - `output`: Optional path to save modified GIF
pub fn apply_modifications(
    input: &PathBuf,
    loop_count: Option<u16>,
    global_delay: Option<u16>,
    frame_delays: Option<HashMap<usize, u16>>,
    output: Option<PathBuf>,
) -> Result<(), String> {
    // Load GIF metadata + frames
    let original_loop_count = info::get_metadata(input, false)
        .ok()
        .map(|meta| meta.loop_count)
        .unwrap_or(1); // fallback if metadata fails
    let file = File::open(input).map_err(|e| format!("Failed to open input: {}", e))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    decoder.allow_unknown_blocks(true);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("Decode error: {}", e))?;

    // Decide output file
    let out_path = output
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("{}.mod.gif", Uuid::new_v4())));
    let out_file =
        File::create(&out_path).map_err(|e| format!("Failed to create output: {}", e))?;
    let mut writer = BufWriter::new(out_file);

    // Init encoder with global palette
    let mut encoder = Encoder::new(
        &mut writer,
        reader.width(),
        reader.height(),
        reader.global_palette().unwrap_or(&[]),
    )
    .map_err(|e| format!("Encoder init error: {}", e))?;

    // Apply loop count if specified
    let effective_loop_count = loop_count.unwrap_or(original_loop_count);
    let repeat_mode = if effective_loop_count == 0 {
        Repeat::Infinite
    } else {
        Repeat::Finite(effective_loop_count - 1)
    };
    let _set_repeat = encoder.set_repeat(repeat_mode);

    let mut index = 0;
    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("Frame read error: {}", e))?
    {
        let mut owned = frame.clone();

        // Set global delay if provided
        if let Some(d) = global_delay {
            owned.delay = d;
        }

        // Override specific frame delay
        if let Some(ref overrides) = frame_delays {
            for (target_idx, new_delay) in overrides.iter() {
                if *target_idx == index {
                    owned.delay = *new_delay;
                }
            }
        }

        encoder
            .write_frame(&owned)
            .map_err(|e| format!("Frame write error: {}", e))?;

        index += 1;
    }

    println!("✅ Modifications applied → {}", out_path.display());
    Ok(())
}

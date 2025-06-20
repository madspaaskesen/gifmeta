use gif::DecodeOptions;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::gifmeta_structs;
use crate::utils::loop_count;

/// Extracts GIF metadata (dimensions, frame count, duration).
///
/// # Arguments
/// * `path` - Path to a `.gif` file
///
/// # Returns
/// * `Ok(GifMetadata)` on success
/// * `Err(String)` if file cannot be read or decoded
pub fn get_metadata(path: &Path) -> Result<gifmeta_structs::GifMetadata, String> {
    let file = File::open(path).map_err(|_| "❌ Failed to open file")?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    decoder.allow_unknown_blocks(true);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|_| "❌ Failed to decode GIF")?;

    let global_palette = reader.global_palette();
    let has_global_palette = global_palette.is_some();
    let global_palette_size = global_palette.map(|p| p.len() / 3); // RGB triplets

    let mut uses_transparency = false;
    let mut frame_count = 0;
    let mut total_duration = 0;
    let mut frames = Vec::new();

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|_| "❌ Error reading frame")?
    {
        if frame.transparent.is_some() {
            uses_transparency = true;
        }

        frames.push(gifmeta_structs::FrameMeta {
            index: frame_count,
            delay_cs: frame.delay,
            transparent_index: frame.transparent,
        });
        frame_count += 1;
        total_duration += frame.delay as u32;
    }

    let loop_count = loop_count::extract_loop_count(path).unwrap_or(0);

    Ok(gifmeta_structs::GifMetadata {
        width: reader.width(),
        height: reader.height(),
        frame_count: frame_count.try_into().unwrap(),
        total_duration_cs: total_duration,
        loop_count: loop_count,
        frames: frames,
        has_global_palette: has_global_palette,
        global_palette_size: global_palette_size,
        uses_transparency: uses_transparency,
    })
}

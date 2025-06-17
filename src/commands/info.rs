use gif::DecodeOptions;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::GifMetadata;

/// Extracts GIF metadata (dimensions, frame count, duration).
///
/// # Arguments
/// * `path` - Path to a `.gif` file
///
/// # Returns
/// * `Ok(GifMetadata)` on success
/// * `Err(String)` if file cannot be read or decoded
pub fn get_metadata(path: &Path) -> Result<GifMetadata, String> {
    let file = File::open(path).map_err(|_| "❌ Failed to open file")?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    decoder.allow_unknown_blocks(true);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|_| "❌ Failed to decode GIF")?;

    let mut frame_count = 0;
    let mut total_duration = 0;

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|_| "❌ Error reading frame")?
    {
        frame_count += 1;
        total_duration += frame.delay as u32;
    }

    Ok(GifMetadata {
        width: reader.width(),
        height: reader.height(),
        frame_count,
        total_duration_cs: total_duration,
    })
}

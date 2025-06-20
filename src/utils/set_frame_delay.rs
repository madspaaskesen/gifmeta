use gif::{DecodeOptions, Encoder};
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// Applies a fixed delay (in centiseconds) to all frames in a GIF.
///
/// # Arguments
/// - `path`: Path to the input GIF file.
/// - `delay`: Delay to apply to each frame (centiseconds).
/// - `output`: Optional output path. If None, overwrites the input file.
pub fn set_frame_delay(input: &PathBuf, delay: u16, output: Option<PathBuf>) -> Result<(), String> {
    // Open input file
    let file = File::open(input).map_err(|e| format!("Failed to open input: {}", e))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("Decode error: {}", e))?;

    // Determine output path
    let (mut writer, out_path): (BufWriter<Box<dyn Write>>, PathBuf) = match output {
        Some(ref path) => {
            let file = File::create(path).map_err(|e| format!("Failed to create output: {}", e))?;
            (BufWriter::new(Box::new(file)), path.clone())
        }
        None => {
            let tmp = NamedTempFile::new().map_err(|e| format!("Tempfile error: {}", e))?;
            let path = tmp.path().to_path_buf();
            (BufWriter::new(Box::new(tmp)), path)
        }
    };

    // Encode
    let mut encoder = Encoder::new(
        &mut writer,
        reader.width(),
        reader.height(),
        reader.global_palette().unwrap_or(&[]),
    )
    .map_err(|e| format!("Encoder init error: {}", e))?;

    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("Frame read error: {}", e))?
    {
        let mut owned = frame.clone();
        owned.delay = delay;
        encoder
            .write_frame(&owned)
            .map_err(|e| format!("Frame write error: {}", e))?;
    }

    // If using a temp file, replace the original input
    if output.is_none() {
        fs::rename(&out_path, input).map_err(|e| format!("Failed to overwrite original: {}", e))?;
    }

    println!(
        "✅ Frame delay of {}cs applied to all frames → {}",
        delay,
        out_path.display()
    );

    Ok(())
}

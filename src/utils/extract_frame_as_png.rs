use std::io::Cursor;

use crate::utils::render::{render_frame, PreviewMode};

/// Renders a frame as it looks when the GIF plays (composited) and returns PNG bytes.
pub fn extract_frame_as_png(path: &str, frame_index: usize) -> Result<Vec<u8>, String> {
    extract_frame_as_png_with_mode(path, frame_index, PreviewMode::Composited)
}

/// Renders a frame in the given [`PreviewMode`] and returns PNG bytes.
pub fn extract_frame_as_png_with_mode(
    path: &str,
    frame_index: usize,
    mode: PreviewMode,
) -> Result<Vec<u8>, String> {
    let image = render_frame(path, frame_index, mode)?;
    let mut buf = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode error: {e}"))?;
    Ok(buf)
}

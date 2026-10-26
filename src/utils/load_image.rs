//! Loads still images (PNG, JPEG) as RGBA, used when inserting frames or creating a GIF.

use std::path::Path;

use image::RgbaImage;

/// Reads only the image header and returns `(width, height)`. Cheap — no pixel decoding.
pub fn image_dimensions(path: &Path) -> Result<(u32, u32), String> {
    image::image_dimensions(path).map_err(|e| format!("Could not read {}: {}", path.display(), e))
}

/// Decodes an image file (PNG or JPEG) to RGBA.
///
/// Pixels with alpha 0 become transparent in the GIF; all other pixels are treated as opaque
/// (GIF has no partial transparency).
pub fn load_rgba(path: &Path) -> Result<RgbaImage, String> {
    image::open(path)
        .map(|img| img.to_rgba8())
        .map_err(|e| format!("Could not read {}: {}", path.display(), e))
}

/// Loads an image and checks it has the expected size.
pub fn load_rgba_sized(path: &Path, width: u32, height: u32) -> Result<RgbaImage, String> {
    let img = load_rgba(path)?;
    if img.dimensions() != (width, height) {
        return Err(format!(
            "{} is {}×{}, but must be {}×{} (same size as the GIF)",
            path.display(),
            img.width(),
            img.height(),
            width,
            height
        ));
    }
    Ok(img)
}

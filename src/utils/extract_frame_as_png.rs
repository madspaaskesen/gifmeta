use gif::DecodeOptions;
use image::{ImageBuffer, ImageEncoder, RgbaImage};
use std::fs::File;
use std::io::{BufReader, Cursor};

pub fn extract_frame_as_png(path: &str, frame_index: usize) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("File open failed: {e}"))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    //decoder.set_color_output(gif::ColorOutput::Indexed);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("GIF decode failed: {e}"))?;

    let mut frame_count = 0;
    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("Read frame error: {e}"))?
    {
        if frame_count == frame_index {
            let width = frame.width as u32;
            let height = frame.height as u32;
            let buffer = frame.buffer.clone(); // RGBA
            let img: RgbaImage = ImageBuffer::from_raw(width, height, buffer.into_owned())
                .ok_or("Failed to build image buffer")?;

            let mut png_bytes = Cursor::new(Vec::new());
            image::codecs::png::PngEncoder::new(&mut png_bytes)
                .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
                .map_err(|e| format!("PNG encode error: {e}"))?;

            return Ok(png_bytes.into_inner());
        }
        frame_count += 1;
    }

    Err(format!("Frame index {} out of bounds", frame_index))
}

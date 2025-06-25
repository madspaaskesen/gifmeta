use gif::DecodeOptions;
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::{BufReader, Cursor};

pub fn extract_frame_as_png(path: &str, frame_index: usize) -> Result<Vec<u8>, String> {
    let file = File::open(path).map_err(|e| format!("File open failed: {e}"))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("GIF decode failed: {e}"))?;

    // Get canvas size from logical screen
    let canvas_width = reader.width() as u32;
    let canvas_height = reader.height() as u32;

    let mut canvas = RgbaImage::from_pixel(canvas_width, canvas_height, Rgba([0, 0, 0, 0]));

    let mut frame_count = 0;
    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("Read frame error: {e}"))?
    {
        // Composite frame onto canvas
        //let buffer = &frame.buffer;
        for y in 0..frame.height {
            for x in 0..frame.width {
                let i = (y as usize * frame.width as usize + x as usize) * 4;
                if i + 3 >= frame.buffer.len() {
                    continue;
                }
                let px = &frame.buffer[i..i + 4];
                let rgba = Rgba([px[0], px[1], px[2], px[3]]);

                let cx = frame.left as u32 + x as u32;
                let cy = frame.top as u32 + y as u32;
                if cx < canvas_width && cy < canvas_height {
                    canvas.put_pixel(cx, cy, rgba);
                }
            }
        }

        if frame_count == frame_index {
            // Return snapshot of canvas at this point
            let mut buf = Vec::new();
            canvas
                .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)
                .map_err(|e| format!("PNG encode error: {e}"))?;
            return Ok(buf);
        }

        frame_count += 1;
    }

    Err(format!("Frame index {} out of bounds", frame_index))
}

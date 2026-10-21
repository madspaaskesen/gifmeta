//! Disposal-aware frame renderer.
//!
//! Replays the animation onto an RGBA canvas the same way browsers do, so a frame of a
//! delta-optimized GIF looks the way it does when the GIF plays.

use gif::{DecodeOptions, DisposalMethod};
use image::{Rgba, RgbaImage};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// How a single frame should be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewMode {
    /// What the viewer sees: all earlier frames composited with correct disposal and
    /// transparency ("pretty" view).
    #[default]
    Composited,
    /// Only the pixels stored in this frame, at its position on an otherwise transparent
    /// canvas ("raw" view) — shows what a delta frame actually contains.
    Raw,
}

const CLEAR: Rgba<u8> = Rgba([0, 0, 0, 0]);

/// Draws `frame` (RGBA buffer) onto `canvas` at its offset, skipping transparent pixels.
fn draw(canvas: &mut RgbaImage, frame: &gif::Frame) {
    let (cw, ch) = canvas.dimensions();
    let fw = frame.width as usize;
    for y in 0..frame.height as u32 {
        let cy = frame.top as u32 + y;
        if cy >= ch {
            break;
        }
        for x in 0..frame.width as u32 {
            let cx = frame.left as u32 + x;
            if cx >= cw {
                break;
            }
            let i = (y as usize * fw + x as usize) * 4;
            let Some(px) = frame.buffer.get(i..i + 4) else {
                continue;
            };
            // Transparent pixels let the canvas below show through.
            if px[3] == 0 {
                continue;
            }
            canvas.put_pixel(cx, cy, Rgba([px[0], px[1], px[2], px[3]]));
        }
    }
}

/// Clears the frame's rectangle on the canvas (disposal `Background`).
fn clear_rect(canvas: &mut RgbaImage, frame: &gif::Frame) {
    let (cw, ch) = canvas.dimensions();
    let right = (frame.left as u32 + frame.width as u32).min(cw);
    let bottom = (frame.top as u32 + frame.height as u32).min(ch);
    for y in frame.top as u32..bottom {
        for x in frame.left as u32..right {
            canvas.put_pixel(x, y, CLEAR);
        }
    }
}

/// Renders frame `frame_index` of the GIF at `path` as a full-canvas RGBA image.
pub fn render_frame<P: AsRef<Path>>(
    path: P,
    frame_index: usize,
    mode: PreviewMode,
) -> Result<RgbaImage, String> {
    let file = File::open(path).map_err(|e| format!("File open failed: {e}"))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::RGBA);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("GIF decode failed: {e}"))?;

    let mut canvas = RgbaImage::from_pixel(reader.width() as u32, reader.height() as u32, CLEAR);

    let mut index = 0;
    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("Read frame error: {e}"))?
    {
        if index == frame_index {
            return Ok(match mode {
                PreviewMode::Composited => {
                    draw(&mut canvas, frame);
                    canvas
                }
                PreviewMode::Raw => {
                    let (w, h) = canvas.dimensions();
                    let mut raw = RgbaImage::from_pixel(w, h, CLEAR);
                    draw(&mut raw, frame);
                    raw
                }
            });
        }

        // Only composited mode needs the history, but it is cheap enough to always track.
        let snapshot = (frame.dispose == DisposalMethod::Previous).then(|| canvas.clone());
        draw(&mut canvas, frame);
        match frame.dispose {
            DisposalMethod::Background => clear_rect(&mut canvas, frame),
            DisposalMethod::Previous => {
                if let Some(s) = snapshot {
                    canvas = s;
                }
            }
            DisposalMethod::Any | DisposalMethod::Keep => {}
        }

        index += 1;
    }

    Err(format!("Frame index {} out of bounds", frame_index))
}

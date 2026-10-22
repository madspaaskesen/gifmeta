//! Encodes a full-canvas RGBA image as a GIF frame with its own (local) palette.
//!
//! Used when a frame has to be written as a full image ("re-keyed"), e.g. after deleting the
//! frame it depended on.

use std::borrow::Cow;
use std::collections::HashMap;

use gif::{DisposalMethod, Frame};
use image::RgbaImage;

/// Result of encoding a full frame.
pub struct EncodedFrame {
    pub frame: Frame<'static>,
    /// `true` when the image had too many colours and was colour-reduced (NeuQuant).
    pub quantized: bool,
}

/// Encodes `image` as a full-canvas frame.
///
/// * ≤ 256 colours (incl. one slot for transparency if needed) → exact local palette, lossless.
/// * otherwise → NeuQuant to 255 colours + a reserved transparent index, so opaque pixels can
///   never accidentally become transparent.
///
/// Pixels with alpha 0 become transparent; all other pixels are treated as opaque.
pub fn encode_full_frame(image: &RgbaImage, delay: u16, dispose: DisposalMethod) -> EncodedFrame {
    let (w, h) = image.dimensions();
    let pixels: &[u8] = image.as_raw();

    let has_transparent = pixels.chunks_exact(4).any(|p| p[3] == 0);
    let max_colors = if has_transparent { 255 } else { 256 };

    // Try an exact palette first.
    let mut lookup: HashMap<[u8; 3], u8> = HashMap::new();
    let mut palette: Vec<u8> = Vec::new();
    let mut exact = true;
    for p in pixels.chunks_exact(4) {
        if p[3] == 0 {
            continue;
        }
        let rgb = [p[0], p[1], p[2]];
        if !lookup.contains_key(&rgb) {
            if lookup.len() == max_colors {
                exact = false;
                break;
            }
            lookup.insert(rgb, lookup.len() as u8);
            palette.extend_from_slice(&rgb);
        }
    }

    let (buffer, palette, transparent) = if exact {
        let transparent_index = has_transparent.then(|| {
            let idx = (palette.len() / 3) as u8;
            palette.extend_from_slice(&[0, 0, 0]);
            idx
        });
        let buffer: Vec<u8> = pixels
            .chunks_exact(4)
            .map(|p| {
                if p[3] == 0 {
                    transparent_index.unwrap_or(0)
                } else {
                    lookup[&[p[0], p[1], p[2]]]
                }
            })
            .collect();
        (buffer, palette, transparent_index)
    } else {
        // Quantize opaque pixels only; index 255 is reserved for transparency.
        let opaque: Vec<u8> = pixels
            .chunks_exact(4)
            .filter(|p| p[3] != 0)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect();
        let colors = if has_transparent { 255 } else { 256 };
        let nq = color_quant::NeuQuant::new(10, colors, &opaque);
        let mut palette = nq.color_map_rgb();
        let transparent_index = has_transparent.then(|| {
            palette.truncate(255 * 3);
            palette.extend_from_slice(&[0, 0, 0]);
            255u8
        });
        let buffer: Vec<u8> = pixels
            .chunks_exact(4)
            .map(|p| {
                if p[3] == 0 {
                    255
                } else {
                    nq.index_of(&[p[0], p[1], p[2], 255]) as u8
                }
            })
            .collect();
        (buffer, palette, transparent_index)
    };

    EncodedFrame {
        frame: Frame {
            width: w as u16,
            height: h as u16,
            left: 0,
            top: 0,
            delay,
            dispose,
            transparent,
            palette: Some(palette),
            buffer: Cow::Owned(buffer),
            ..Frame::default()
        },
        quantized: !exact,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn decode_pixel(f: &Frame, x: u32, y: u32) -> Option<[u8; 3]> {
        let i = f.buffer[(y * f.width as u32 + x) as usize];
        if Some(i) == f.transparent {
            return None;
        }
        let pal = f.palette.as_ref().unwrap();
        let o = i as usize * 3;
        Some([pal[o], pal[o + 1], pal[o + 2]])
    }

    #[test]
    fn exact_palette_is_lossless() {
        let mut img = RgbaImage::from_pixel(4, 2, Rgba([10, 20, 30, 255]));
        img.put_pixel(1, 0, Rgba([200, 100, 50, 255]));
        img.put_pixel(3, 1, Rgba([0, 0, 0, 0]));
        let e = encode_full_frame(&img, 7, DisposalMethod::Keep);
        assert!(!e.quantized);
        assert_eq!(e.frame.delay, 7);
        assert_eq!(decode_pixel(&e.frame, 0, 0), Some([10, 20, 30]));
        assert_eq!(decode_pixel(&e.frame, 1, 0), Some([200, 100, 50]));
        assert_eq!(decode_pixel(&e.frame, 3, 1), None);
    }

    #[test]
    fn black_opaque_is_not_transparent() {
        let mut img = RgbaImage::from_pixel(2, 1, Rgba([0, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([0, 0, 0, 0]));
        let e = encode_full_frame(&img, 1, DisposalMethod::Keep);
        assert_eq!(decode_pixel(&e.frame, 0, 0), Some([0, 0, 0]));
        assert_eq!(decode_pixel(&e.frame, 1, 0), None);
    }

    #[test]
    fn many_colors_are_quantized_and_keep_transparency() {
        let mut img = RgbaImage::new(40, 20);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgba([(x * 6) as u8, (y * 12) as u8, ((x + y) * 3) as u8, 255]);
        }
        img.put_pixel(0, 0, Rgba([0, 0, 0, 0]));
        let e = encode_full_frame(&img, 1, DisposalMethod::Keep);
        assert!(e.quantized);
        assert_eq!(e.frame.transparent, Some(255));
        assert_eq!(decode_pixel(&e.frame, 0, 0), None);
        // No opaque pixel may map to the transparent index.
        let opaque_transparent = e.frame.buffer.iter().skip(1).filter(|&&i| i == 255).count();
        assert_eq!(opaque_transparent, 0);
    }
}

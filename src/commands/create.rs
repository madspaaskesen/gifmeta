// src/commands/create.rs — build a new animated GIF from still images.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use gif::{DisposalMethod, Encoder, Repeat};
use uuid::Uuid;

use crate::gifmeta_structs::LoopMode;
use crate::utils::encode_frame::encode_full_frame;
use crate::utils::load_image::{image_dimensions, load_rgba_sized};

/// Default frame delay for [`create_gif`]: 10 cs = 100 ms (10 fps).
pub const DEFAULT_CREATE_DELAY: u16 = 10;

/// Settings for [`create_gif`]. Build it with `..Default::default()`:
///
/// ```
/// use gifmeta::{CreateOptions, LoopMode};
/// let opts = CreateOptions {
///     delay: Some(25),
///     loop_mode: Some(LoopMode::Times(3)),
///     ..Default::default()
/// };
/// assert_eq!(opts.delay, Some(25));
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreateOptions {
    /// Where to write the GIF. `None` → a temp file `$TMPDIR/<uuid>.gif`.
    pub output: Option<PathBuf>,
    /// Delay for every frame in centiseconds. `None` → 10 (100 ms).
    pub delay: Option<u16>,
    /// Per-frame delay overrides (0-based image index → centiseconds).
    pub delays: Option<HashMap<usize, u16>>,
    /// How often the GIF plays. `None` → loops forever.
    pub loop_mode: Option<LoopMode>,
}

/// What [`create_gif`] did.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CreateReport {
    /// The file that was written.
    pub output: PathBuf,
    /// Number of frames.
    pub frames: usize,
    /// Canvas size.
    pub width: u32,
    pub height: u32,
    /// Images (0-based) that had > 256 colours and were colour-reduced.
    pub quantized: Vec<usize>,
}

/// Builds an animated GIF from at least two still images (PNG, JPEG or WebP).
///
/// All images must have the same width × height. Each image becomes one full frame with its own
/// palette — exact when it has ≤ 256 colours, otherwise colour-reduced (NeuQuant). Pixels with
/// alpha 0 become transparent (each frame shows only its own image, nothing shines through);
/// semi-transparent pixels are drawn opaque.
pub fn create(images: &[PathBuf], opts: &CreateOptions) -> Result<CreateReport, String> {
    if images.len() < 2 {
        return Err("Need at least 2 images to make an animated GIF".into());
    }
    if let Some(LoopMode::Times(0)) = opts.loop_mode {
        return Err("Loop count must be at least 1 (use LoopMode::Infinite for endless)".into());
    }
    if let Some(&i) = opts
        .delays
        .iter()
        .flat_map(|m| m.keys())
        .find(|&&i| i >= images.len())
    {
        return Err(format!(
            "Delay for frame {} but there are only {} images (0–{})",
            i,
            images.len(),
            images.len() - 1
        ));
    }

    // Check every size first (header only), so a wrong file fails before anything is written.
    let (width, height) = image_dimensions(&images[0])?;
    let mismatched: Vec<String> = images
        .iter()
        .map(|p| image_dimensions(p).map(|d| (p, d)))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(_, d)| *d != (width, height))
        .map(|(p, (w, h))| format!("{} ({}×{})", p.display(), w, h))
        .collect();
    if !mismatched.is_empty() {
        return Err(format!(
            "All images must be {}×{} (like {}). Different size: {}",
            width,
            height,
            images[0].display(),
            mismatched.join(", ")
        ));
    }
    if width > u16::MAX as u32 || height > u16::MAX as u32 {
        return Err(format!(
            "Images are {}×{} — GIF supports at most 65535×65535",
            width, height
        ));
    }

    let out_path = opts
        .output
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("{}.gif", Uuid::new_v4())));
    let file = File::create(&out_path).map_err(|e| format!("Failed to create output: {}", e))?;
    let mut encoder = Encoder::new(BufWriter::new(file), width as u16, height as u16, &[])
        .map_err(|e| format!("Encoder init error: {}", e))?;
    let repeat = match opts.loop_mode.unwrap_or(LoopMode::Infinite) {
        LoopMode::Infinite => Repeat::Infinite,
        LoopMode::Times(n) => Repeat::Finite(n - 1),
    };
    encoder
        .set_repeat(repeat)
        .map_err(|e| format!("Failed to write loop setting: {}", e))?;

    let mut report = CreateReport {
        output: out_path,
        width,
        height,
        ..Default::default()
    };
    // Every frame is a full image. A frame is cleared after it is shown (disposal `Background`)
    // when it — or the next frame — has transparent pixels, so nothing shows through. Frames are
    // written one behind, because the next image decides how the current one is disposed.
    let mut pending: Option<gif::Frame<'static>> = None;
    for (i, path) in images.iter().enumerate() {
        let img = load_rgba_sized(path, width, height)?;
        let delay = opts
            .delays
            .as_ref()
            .and_then(|m| m.get(&i).copied())
            .or(opts.delay)
            .unwrap_or(DEFAULT_CREATE_DELAY);
        let transparent = img.pixels().any(|p| p[3] == 0);
        let dispose = if transparent {
            DisposalMethod::Background
        } else {
            DisposalMethod::Keep
        };
        let encoded = encode_full_frame(&img, delay, dispose);
        if encoded.quantized {
            report.quantized.push(i);
        }
        if let Some(mut prev) = pending.replace(encoded.frame) {
            if transparent {
                prev.dispose = DisposalMethod::Background;
            }
            encoder
                .write_frame(&prev)
                .map_err(|e| format!("Frame write error: {}", e))?;
        }
        report.frames += 1;
    }
    if let Some(last) = pending {
        encoder
            .write_frame(&last)
            .map_err(|e| format!("Frame write error: {}", e))?;
    }
    let mut out = encoder
        .into_inner()
        .map_err(|e| format!("Failed to finish GIF: {}", e))?;
    out.flush()
        .map_err(|e| format!("Failed to write GIF: {}", e))?;
    Ok(report)
}

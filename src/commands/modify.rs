// src/commands/modify.rs

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use gif::{DecodeOptions, Encoder, Repeat};
use uuid::Uuid;

use crate::gifmeta_structs::LoopMode;
use crate::utils::loop_count;

/// What to change in a GIF. Build it with struct update syntax so new options can be added
/// later without breaking your code:
///
/// ```
/// use gifmeta::{LoopMode, ModOptions};
/// let opts = ModOptions {
///     loop_mode: Some(LoopMode::Infinite),
///     delay_all: Some(10),
///     ..Default::default()
/// };
/// assert!(!opts.is_empty());
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModOptions {
    /// Where to write the result. `None` → a temp file `$TMPDIR/<uuid>.mod.gif`.
    pub output: Option<PathBuf>,
    /// New loop setting. `None` keeps the original (incl. "no loop block").
    pub loop_mode: Option<LoopMode>,
    /// Delay for all frames, in centiseconds.
    pub delay_all: Option<u16>,
    /// Per-frame delay overrides (0-based frame index → centiseconds). Applied after `delay_all`.
    pub delays: Option<HashMap<usize, u16>>,
}

impl ModOptions {
    /// `true` when no modification is requested.
    pub fn is_empty(&self) -> bool {
        self.loop_mode.is_none()
            && self.delay_all.is_none()
            && self.delays.as_ref().map_or(true, |m| m.is_empty())
    }
}

/// Re-encodes `input` with the requested modifications. Frames are copied as indexed
/// `gif::Frame`s, so palettes, offsets, disposal and transparency are preserved.
///
/// Returns the path that was written.
pub fn apply(input: &PathBuf, opts: &ModOptions) -> Result<PathBuf, String> {
    if let Some(LoopMode::Times(0)) = opts.loop_mode {
        return Err("Loop count must be at least 1 (use LoopMode::Infinite for endless)".into());
    }
    // Keep the original loop setting unless a new one is given.
    let loop_mode = match opts.loop_mode {
        Some(m) => m,
        None => loop_count::read_loop_mode(input)?,
    };

    let file = File::open(input).map_err(|e| format!("Failed to open input: {}", e))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    decoder.allow_unknown_blocks(true);
    let mut reader = decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("Decode error: {}", e))?;

    let out_path = opts
        .output
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("{}.mod.gif", Uuid::new_v4())));
    let out_file =
        File::create(&out_path).map_err(|e| format!("Failed to create output: {}", e))?;
    let mut writer = BufWriter::new(out_file);

    let mut encoder = Encoder::new(
        &mut writer,
        reader.width(),
        reader.height(),
        reader.global_palette().unwrap_or(&[]),
    )
    .map_err(|e| format!("Encoder init error: {}", e))?;

    // `Repeat::Finite(0)` makes the gif crate write no loop block → plays once.
    let repeat = match loop_mode {
        LoopMode::Infinite => Repeat::Infinite,
        LoopMode::Times(n) => Repeat::Finite(n - 1),
    };
    encoder
        .set_repeat(repeat)
        .map_err(|e| format!("Failed to write loop setting: {}", e))?;

    let mut index = 0;
    while let Some(frame) = reader
        .read_next_frame()
        .map_err(|e| format!("Frame read error: {}", e))?
    {
        let mut owned = frame.clone();
        if let Some(d) = opts.delay_all {
            owned.delay = d;
        }
        if let Some(d) = opts.delays.as_ref().and_then(|m| m.get(&index)) {
            owned.delay = *d;
        }
        encoder
            .write_frame(&owned)
            .map_err(|e| format!("Frame write error: {}", e))?;
        index += 1;
    }

    Ok(out_path)
}

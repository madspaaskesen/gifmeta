// src/commands/modify.rs

use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use gif::{DecodeOptions, Encoder, Frame, Repeat};
use uuid::Uuid;

use crate::gifmeta_structs::LoopMode;
use crate::utils::encode_frame::encode_full_frame;
use crate::utils::loop_count;
use crate::utils::render::Compositor;

/// What happens to the delay of a deleted frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DeleteMode {
    /// Add the deleted frame's delay to the previous kept frame (or the next one when deleting
    /// at the start) — the animation keeps its total duration.
    #[default]
    MergeDelay,
    /// Drop the delay — the animation gets shorter.
    DropDelay,
}

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
    /// Per-frame delay overrides (0-based **original** frame index → centiseconds).
    /// Applied after `delay_all`.
    pub delays: Option<HashMap<usize, u16>>,
    /// Frames to delete (0-based **original** indices). Frames that depended on a deleted
    /// frame are automatically re-written as full images, so the GIF still looks right.
    pub delete_frames: Option<Vec<usize>>,
    /// What happens to the delay of deleted frames (default: merge into the previous frame).
    pub delete_mode: DeleteMode,
    /// Frames to convert to full images ("keyframes") — original indices.
    pub keyframes: Option<Vec<usize>>,
    /// Convert **every** frame to a full image (un-optimize the GIF).
    pub unoptimize: bool,
}

impl ModOptions {
    /// `true` when no modification is requested.
    pub fn is_empty(&self) -> bool {
        self.loop_mode.is_none()
            && self.delay_all.is_none()
            && self.delays.as_ref().map_or(true, |m| m.is_empty())
            && !self.has_frame_edits()
    }

    /// `true` when frames are deleted or re-keyed (needs the full re-render pipeline).
    pub fn has_frame_edits(&self) -> bool {
        self.unoptimize
            || self.delete_frames.as_ref().map_or(false, |v| !v.is_empty())
            || self.keyframes.as_ref().map_or(false, |v| !v.is_empty())
    }
}

/// What [`apply`] did.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModReport {
    /// The file that was written.
    pub output: PathBuf,
    /// Frames in the original / written file.
    pub frames_in: usize,
    pub frames_out: usize,
    /// Original indices of deleted frames.
    pub deleted: Vec<usize>,
    /// Original indices of frames written as full images (requested or needed after a delete).
    pub rekeyed: Vec<usize>,
    /// Re-keyed frames that had > 256 colours and were colour-reduced (small quality loss).
    pub quantized: Vec<usize>,
    /// Frames whose transparent areas can't be reproduced exactly (earlier pixels may show
    /// through where the original was transparent). Rare; only transparent GIFs.
    pub transparency_warnings: Vec<usize>,
}

impl ModReport {
    /// Human-readable warnings (empty when everything is exact).
    pub fn warnings(&self) -> Vec<String> {
        let mut w = Vec::new();
        if !self.quantized.is_empty() {
            w.push(format!(
                "frames {:?} had more than 256 colours and were colour-reduced",
                self.quantized
            ));
        }
        if !self.transparency_warnings.is_empty() {
            w.push(format!(
                "frames {:?}: transparent areas may show pixels from earlier frames",
                self.transparency_warnings
            ));
        }
        w
    }
}

fn open_reader(
    input: &PathBuf,
    color: gif::ColorOutput,
) -> Result<gif::Decoder<BufReader<File>>, String> {
    let file = File::open(input).map_err(|e| format!("Failed to open input: {}", e))?;
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(color);
    decoder.allow_unknown_blocks(true);
    decoder
        .read_info(BufReader::new(file))
        .map_err(|e| format!("Decode error: {}", e))
}

/// Re-encodes `input` with the requested modifications and returns what was done.
///
/// Unchanged frames are copied as indexed `gif::Frame`s (palettes, offsets, disposal and
/// transparency preserved, lossless). When frames are deleted or re-keyed, the original and the
/// new animation are replayed side by side ("shadow simulation"): a kept frame is copied as-is
/// when the canvas before it is identical to the original, otherwise it is written as a full
/// image of how it originally looked.
pub fn apply_with_report(input: &PathBuf, opts: &ModOptions) -> Result<ModReport, String> {
    if let Some(LoopMode::Times(0)) = opts.loop_mode {
        return Err("Loop count must be at least 1 (use LoopMode::Infinite for endless)".into());
    }
    // Keep the original loop setting unless a new one is given.
    let loop_mode = match opts.loop_mode {
        Some(m) => m,
        None => loop_count::read_loop_mode(input)?,
    };

    let mut idx_reader = open_reader(input, gif::ColorOutput::Indexed)?;
    let (width, height) = (idx_reader.width(), idx_reader.height());

    // --- validate frame edits ---------------------------------------------------------
    let frame_edits = opts.has_frame_edits();
    let deleted: BTreeSet<usize> = opts.delete_frames.iter().flatten().copied().collect();
    let forced: BTreeSet<usize> = opts.keyframes.iter().flatten().copied().collect();
    let frame_count = if frame_edits {
        crate::commands::info::get_metadata(input, false)?.frame_count as usize
    } else {
        0
    };
    if frame_edits {
        if let Some(&i) = deleted
            .iter()
            .chain(forced.iter())
            .find(|&&i| i >= frame_count)
        {
            return Err(format!(
                "Frame index {} out of range (the GIF has {} frames, 0–{})",
                i,
                frame_count,
                frame_count.saturating_sub(1)
            ));
        }
        if deleted.len() >= frame_count {
            return Err("Can't delete all frames — at least one must remain".into());
        }
        if let Some(i) = deleted.intersection(&forced).next() {
            return Err(format!(
                "Frame {} is both deleted and marked as keyframe",
                i
            ));
        }
    }

    // --- output ---------------------------------------------------------------------
    let out_path = opts
        .output
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("{}.mod.gif", Uuid::new_v4())));
    let out_file =
        File::create(&out_path).map_err(|e| format!("Failed to create output: {}", e))?;
    let mut writer = BufWriter::new(out_file);
    let mut encoder = Encoder::new(
        &mut writer,
        width,
        height,
        idx_reader.global_palette().unwrap_or(&[]),
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

    let delay_for = |index: usize, original: u16| -> u16 {
        opts.delays
            .as_ref()
            .and_then(|m| m.get(&index).copied())
            .or(opts.delay_all)
            .unwrap_or(original)
    };

    let mut report = ModReport {
        output: out_path.clone(),
        deleted: deleted.iter().copied().collect(),
        ..Default::default()
    };

    // --- fast path: only loop / delays --------------------------------------------------
    if !frame_edits {
        let mut index = 0;
        while let Some(frame) = idx_reader
            .read_next_frame()
            .map_err(|e| format!("Frame read error: {}", e))?
        {
            let mut owned = frame.clone();
            owned.delay = delay_for(index, frame.delay);
            encoder
                .write_frame(&owned)
                .map_err(|e| format!("Frame write error: {}", e))?;
            index += 1;
        }
        report.frames_in = index;
        report.frames_out = index;
        return Ok(report);
    }

    // --- frame edits: shadow simulation ---------------------------------------------------
    let mut rgba_reader = open_reader(input, gif::ColorOutput::RGBA)?;
    let mut original = Compositor::new(width as u32, height as u32);
    let mut output = Compositor::new(width as u32, height as u32);

    // The last kept frame is held back so delays of following deleted frames can be merged.
    let mut pending: Option<Frame<'static>> = None;
    // Delay of deleted frames before the first kept frame (merged into the next kept frame).
    let mut carry: u16 = 0;
    let mut written = 0usize;

    let mut index = 0usize;
    loop {
        let Some(fi) = idx_reader
            .read_next_frame()
            .map_err(|e| format!("Frame read error: {}", e))?
            .cloned()
        else {
            break;
        };
        let fr = rgba_reader
            .read_next_frame()
            .map_err(|e| format!("Frame read error: {}", e))?
            .ok_or("Frame count mismatch between decoders")?
            .clone();
        let delay = delay_for(index, fi.delay);

        // Canvas before this frame, in the original and in our output.
        let states_match = output.canvas == original.canvas;
        let snapshot = original.draw(&fr);

        if deleted.contains(&index) {
            if opts.delete_mode == DeleteMode::MergeDelay {
                match pending.as_mut() {
                    Some(prev) => prev.delay = prev.delay.saturating_add(delay),
                    None => carry = carry.saturating_add(delay),
                }
            }
        } else {
            let mut out_frame: Frame<'static>;
            if states_match && !opts.unoptimize && !forced.contains(&index) {
                // Lossless copy of the original frame.
                out_frame = fi.into_owned_static();
                let snap = output.draw(&fr);
                output.dispose(&fr, snap);
            } else {
                // Write the frame as a full image of how it originally looked.
                let encoded = encode_full_frame(&original.canvas, delay, fr.dispose);
                if encoded.quantized {
                    report.quantized.push(index);
                }
                report.rekeyed.push(index);
                out_frame = encoded.frame;

                // Simulate the full frame on our output canvas.
                let full = Frame {
                    width,
                    height,
                    dispose: fr.dispose,
                    buffer: std::borrow::Cow::Owned(original.canvas.as_raw().clone()),
                    ..Frame::default()
                };
                let conflict = output
                    .canvas
                    .pixels()
                    .zip(original.canvas.pixels())
                    .any(|(o, want)| want[3] == 0 && o[3] != 0);
                if conflict {
                    report.transparency_warnings.push(index);
                }
                let snap = output.draw(&full);
                output.dispose(&full, snap);
            }
            out_frame.delay = delay.saturating_add(std::mem::take(&mut carry));

            if let Some(prev) = pending.replace(out_frame) {
                encoder
                    .write_frame(&prev)
                    .map_err(|e| format!("Frame write error: {}", e))?;
                written += 1;
            }
        }

        original.dispose(&fr, snapshot);
        index += 1;
    }

    if let Some(prev) = pending.take() {
        encoder
            .write_frame(&prev)
            .map_err(|e| format!("Frame write error: {}", e))?;
        written += 1;
    }

    report.frames_in = index;
    report.frames_out = written;
    Ok(report)
}

/// Re-encodes `input` with the requested modifications. Returns the path that was written.
pub fn apply(input: &PathBuf, opts: &ModOptions) -> Result<PathBuf, String> {
    apply_with_report(input, opts).map(|r| r.output)
}

/// Helper: turn a borrowed decoder frame into an owned `'static` frame.
trait IntoOwnedStatic {
    fn into_owned_static(self) -> Frame<'static>;
}

impl IntoOwnedStatic for Frame<'_> {
    fn into_owned_static(self) -> Frame<'static> {
        Frame {
            buffer: std::borrow::Cow::Owned(self.buffer.into_owned()),
            palette: self.palette,
            delay: self.delay,
            dispose: self.dispose,
            transparent: self.transparent,
            needs_user_input: self.needs_user_input,
            top: self.top,
            left: self.left,
            width: self.width,
            height: self.height,
            interlaced: self.interlaced,
        }
    }
}

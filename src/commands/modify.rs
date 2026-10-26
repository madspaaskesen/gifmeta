// src/commands/modify.rs

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use gif::{DecodeOptions, DisposalMethod, Encoder, Frame, Repeat};
use image::RgbaImage;
use uuid::Uuid;

use crate::gifmeta_structs::LoopMode;
use crate::utils::encode_frame::encode_full_frame;
use crate::utils::load_image::load_rgba_sized;
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

/// An image (PNG, JPEG or WebP) to insert as a new frame.
///
/// ```
/// use gifmeta::FrameInsert;
/// // Insert logo.png before original frame 3, shown for 50 cs.
/// let ins = FrameInsert::new(3, "logo.png").with_delay(50);
/// assert_eq!(ins.before, 3);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInsert {
    /// Insert before this **original** frame index. Use the frame count to append at the end.
    pub before: usize,
    /// Image file. Must have the same width × height as the GIF.
    pub image: PathBuf,
    /// Delay in centiseconds. `None` → the delay of the frame it is inserted before
    /// (or of the last frame when appending).
    pub delay: Option<u16>,
}

impl FrameInsert {
    pub fn new(before: usize, image: impl Into<PathBuf>) -> Self {
        Self {
            before,
            image: image.into(),
            delay: None,
        }
    }

    pub fn with_delay(mut self, delay: u16) -> Self {
        self.delay = Some(delay);
        self
    }
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
    /// Images to insert as new frames (same size as the GIF). Inserting before frame N and
    /// deleting frame N replaces it (its delay is not merged into the new frame).
    pub inserts: Option<Vec<FrameInsert>>,
    /// Frames to duplicate (original indices). Each copy is placed right after its frame;
    /// list an index twice for two copies.
    pub duplicates: Option<Vec<usize>>,
    /// New frame order: every original index exactly once, e.g. `[2, 0, 1]`.
    /// Can't be combined with `delete_frames`, `inserts` or `duplicates`.
    /// Holds all frames in memory while writing.
    pub order: Option<Vec<usize>>,
}

impl ModOptions {
    /// `true` when no modification is requested.
    pub fn is_empty(&self) -> bool {
        self.loop_mode.is_none()
            && self.delay_all.is_none()
            && self.delays.as_ref().map_or(true, |m| m.is_empty())
            && !self.has_frame_edits()
    }

    /// `true` when frames are added, removed, moved or re-keyed (needs the full re-render
    /// pipeline).
    pub fn has_frame_edits(&self) -> bool {
        self.unoptimize
            || self.delete_frames.as_ref().is_some_and(|v| !v.is_empty())
            || self.keyframes.as_ref().is_some_and(|v| !v.is_empty())
            || self.inserts.as_ref().is_some_and(|v| !v.is_empty())
            || self.duplicates.as_ref().is_some_and(|v| !v.is_empty())
            || self.order.is_some()
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
    /// Number of inserted images.
    pub inserted: usize,
    /// Original indices that got a copy (once per copy).
    pub duplicated: Vec<usize>,
    /// `true` when the frames were re-ordered.
    pub reordered: bool,
    /// Inserted images that had > 256 colours and were colour-reduced.
    pub quantized_inserts: Vec<PathBuf>,
    /// Inserted images whose transparent areas may show pixels from the frame before.
    pub insert_transparency_warnings: Vec<PathBuf>,
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
        for p in &self.quantized_inserts {
            w.push(format!(
                "{} has more than 256 colours and was colour-reduced",
                p.display()
            ));
        }
        for p in &self.insert_transparency_warnings {
            w.push(format!(
                "{}: transparent areas may show pixels from the frame before",
                p.display()
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

/// Writes the output frames and replays them on its own canvas ("shadow simulation"), so every
/// frame can be checked against how it looked in the original.
///
/// The last frame is held back so delays of following deleted frames can be merged into it.
struct FrameWriter {
    encoder: Encoder<BufWriter<File>>,
    /// Canvas of the output animation, as a viewer would show it.
    output: Compositor,
    pending: Option<Frame<'static>>,
    /// Delay of deleted frames before the first written frame.
    carry: u16,
    written: usize,
    width: u16,
    height: u16,
}

impl FrameWriter {
    fn push(&mut self, mut frame: Frame<'static>) -> Result<(), String> {
        frame.delay = frame.delay.saturating_add(std::mem::take(&mut self.carry));
        if let Some(prev) = self.pending.replace(frame) {
            self.encoder
                .write_frame(&prev)
                .map_err(|e| format!("Frame write error: {}", e))?;
            self.written += 1;
        }
        Ok(())
    }

    /// Adds a deleted frame's delay to the previous written frame (or the next one at the start).
    fn merge_delay(&mut self, delay: u16) {
        match self.pending.as_mut() {
            Some(prev) => prev.delay = prev.delay.saturating_add(delay),
            None => self.carry = self.carry.saturating_add(delay),
        }
    }

    /// Writes original frame `index`.
    ///
    /// * `fi` / `fr` – the frame decoded as indexed / RGBA.
    /// * `after` – the original canvas after drawing the frame (how it looks when shown).
    /// * `states_match` – the output canvas before this frame equals the original canvas
    ///   before it. Then the frame is copied unchanged (lossless); otherwise it is written as a
    ///   full image of `after`.
    #[allow(clippy::too_many_arguments)]
    fn original(
        &mut self,
        index: usize,
        fi: &Frame<'static>,
        fr: &Frame<'static>,
        after: &RgbaImage,
        delay: u16,
        states_match: bool,
        force_full: bool,
        report: &mut ModReport,
    ) -> Result<(), String> {
        if states_match && !force_full {
            let mut out = fi.clone();
            out.delay = delay;
            let snap = self.output.draw(fr);
            self.output.dispose(fr, snap);
            return self.push(out);
        }

        let encoded = encode_full_frame(after, delay, fr.dispose);
        if encoded.quantized {
            report.quantized.push(index);
        }
        report.rekeyed.push(index);
        if !self.make_room_for(after) {
            report.transparency_warnings.push(index);
        }
        self.simulate_full(after, fr.dispose);
        self.push(encoded.frame)
    }

    /// Writes an image as a new full frame. Returns `(quantized, transparency_conflict)`.
    fn image(&mut self, img: &RgbaImage, delay: u16) -> Result<(bool, bool), String> {
        // A transparent image clears itself afterwards, so the next frame starts clean.
        let dispose = if img.pixels().any(|p| p[3] == 0) {
            DisposalMethod::Background
        } else {
            DisposalMethod::Keep
        };
        let encoded = encode_full_frame(img, delay, dispose);
        let conflict = !self.make_room_for(img);
        self.simulate_full(img, dispose);
        self.push(encoded.frame)?;
        Ok((encoded.quantized, conflict))
    }

    /// Makes sure a full frame showing `want` can be drawn exactly. GIF can only draw on top,
    /// so where `want` is transparent the canvas must already be clear. If it isn't, the previous
    /// frame is switched to disposal `Background` (cleared after it is shown) when that is safe.
    /// Returns `false` when pixels from earlier frames would still show through.
    fn make_room_for(&mut self, want: &RgbaImage) -> bool {
        if !self.would_show_through(want) {
            return true;
        }
        let (w, h) = (self.width, self.height);
        if let Some(prev) = self.pending.as_mut() {
            // With Keep/Any the canvas after disposal equals what the previous frame showed, so
            // we can decide now how it is disposed.
            if matches!(prev.dispose, DisposalMethod::Keep | DisposalMethod::Any) {
                let full_canvas =
                    prev.left == 0 && prev.top == 0 && prev.width == w && prev.height == h;
                if !full_canvas {
                    // A small (delta) frame only clears its own rect → write it as a full image
                    // of what it showed, so clearing it clears everything.
                    let shown = encode_full_frame(
                        &self.output.canvas,
                        prev.delay,
                        DisposalMethod::Background,
                    );
                    *prev = shown.frame;
                }
                prev.dispose = DisposalMethod::Background;
                self.output.canvas = RgbaImage::new(w as u32, h as u32);
            }
        }
        !self.would_show_through(want)
    }

    /// GIF can only draw on top: pixels transparent in `want` but opaque on the output canvas
    /// would show the frame below.
    fn would_show_through(&self, want: &RgbaImage) -> bool {
        self.output
            .canvas
            .pixels()
            .zip(want.pixels())
            .any(|(o, w)| w[3] == 0 && o[3] != 0)
    }

    fn simulate_full(&mut self, img: &RgbaImage, dispose: DisposalMethod) {
        let full = Frame {
            width: self.width,
            height: self.height,
            dispose,
            buffer: Cow::Borrowed(img.as_raw()),
            ..Frame::default()
        };
        let snap = self.output.draw(&full);
        self.output.dispose(&full, snap);
    }

    /// Writes the held-back frame and the GIF trailer. Returns how many frames were written.
    fn finish(mut self) -> Result<usize, String> {
        if let Some(prev) = self.pending.take() {
            self.encoder
                .write_frame(&prev)
                .map_err(|e| format!("Frame write error: {}", e))?;
            self.written += 1;
        }
        let written = self.written;
        let mut out = self
            .encoder
            .into_inner()
            .map_err(|e| format!("Failed to finish GIF: {}", e))?;
        std::io::Write::flush(&mut out).map_err(|e| format!("Failed to write GIF: {}", e))?;
        Ok(written)
    }
}

fn read_frame_pair(
    idx_reader: &mut gif::Decoder<BufReader<File>>,
    rgba_reader: &mut gif::Decoder<BufReader<File>>,
) -> Result<Option<(Frame<'static>, Frame<'static>)>, String> {
    let Some(fi) = idx_reader
        .read_next_frame()
        .map_err(|e| format!("Frame read error: {}", e))?
        .cloned()
    else {
        return Ok(None);
    };
    let fr = rgba_reader
        .read_next_frame()
        .map_err(|e| format!("Frame read error: {}", e))?
        .ok_or("Frame count mismatch between decoders")?
        .clone();
    Ok(Some((fi.into_owned_static(), fr.into_owned_static())))
}

fn out_of_range(i: usize, frame_count: usize) -> String {
    format!(
        "Frame index {} out of range (the GIF has {} frames, 0–{})",
        i,
        frame_count,
        frame_count.saturating_sub(1)
    )
}

/// Re-encodes `input` with the requested modifications and returns what was done.
///
/// Unchanged frames are copied as indexed `gif::Frame`s (palettes, offsets, disposal and
/// transparency preserved, lossless). When frames are added, removed, moved or re-keyed, the
/// original and the new animation are replayed side by side ("shadow simulation"): a frame is
/// copied as-is when the canvas before it is identical to the original, otherwise it is written
/// as a full image of how it originally looked.
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
    let duplicates: Vec<usize> = opts.duplicates.iter().flatten().copied().collect();
    let insert_list: &[FrameInsert] = opts.inserts.as_deref().unwrap_or(&[]);
    let mut inserts_at: BTreeMap<usize, Vec<(RgbaImage, &FrameInsert)>> = BTreeMap::new();

    if frame_edits {
        let frame_count = crate::commands::info::get_metadata(input, false)?.frame_count as usize;
        if let Some(&i) = deleted
            .iter()
            .chain(forced.iter())
            .chain(duplicates.iter())
            .find(|&&i| i >= frame_count)
        {
            return Err(out_of_range(i, frame_count));
        }
        if deleted.len() >= frame_count && insert_list.is_empty() {
            return Err("Can't delete all frames — at least one must remain".into());
        }
        if let Some(i) = deleted.intersection(&forced).next() {
            return Err(format!(
                "Frame {} is both deleted and marked as keyframe",
                i
            ));
        }
        if let Some(i) = duplicates.iter().find(|i| deleted.contains(i)) {
            return Err(format!("Frame {} is both deleted and duplicated", i));
        }
        if let Some(order) = &opts.order {
            if !deleted.is_empty() || !insert_list.is_empty() || !duplicates.is_empty() {
                return Err(
                    "A new frame order can't be combined with delete, insert or \
                     duplicate — do them in two steps"
                        .into(),
                );
            }
            if let Some(&i) = order.iter().find(|&&i| i >= frame_count) {
                return Err(out_of_range(i, frame_count));
            }
            let unique: BTreeSet<usize> = order.iter().copied().collect();
            if order.len() != frame_count || unique.len() != frame_count {
                return Err(format!(
                    "The new order must list every frame exactly once (0–{}); got {} entries",
                    frame_count.saturating_sub(1),
                    order.len()
                ));
            }
        }
        // Load inserted images up front, so a bad file fails before anything is written.
        for ins in insert_list {
            if ins.before > frame_count {
                return Err(format!(
                    "Can't insert before frame {} (the GIF has {} frames; use {} to append)",
                    ins.before, frame_count, frame_count
                ));
            }
            let img = load_rgba_sized(&ins.image, width as u32, height as u32)?;
            inserts_at.entry(ins.before).or_default().push((img, ins));
        }
    }

    // --- output ---------------------------------------------------------------------
    let out_path = opts
        .output
        .clone()
        .unwrap_or_else(|| std::env::temp_dir().join(format!("{}.mod.gif", Uuid::new_v4())));
    let out_file =
        File::create(&out_path).map_err(|e| format!("Failed to create output: {}", e))?;
    let global_palette = idx_reader.global_palette().unwrap_or(&[]).to_vec();
    let mut encoder = Encoder::new(BufWriter::new(out_file), width, height, &global_palette)
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
    let mut writer = FrameWriter {
        encoder,
        output: Compositor::new(width as u32, height as u32),
        pending: None,
        carry: 0,
        written: 0,
        width,
        height,
    };
    let force = |index: usize| opts.unoptimize || forced.contains(&index);

    if let Some(order) = &opts.order {
        // Frames are used out of sequence → keep every frame and its canvases in memory.
        struct Stored {
            fi: Frame<'static>,
            fr: Frame<'static>,
            before: RgbaImage,
            after: RgbaImage,
            delay: u16,
        }
        let mut frames: Vec<Stored> = Vec::new();
        while let Some((fi, fr)) = read_frame_pair(&mut idx_reader, &mut rgba_reader)? {
            let delay = delay_for(frames.len(), fi.delay);
            let before = original.canvas.clone();
            let snapshot = original.draw(&fr);
            let after = original.canvas.clone();
            original.dispose(&fr, snapshot);
            frames.push(Stored {
                fi,
                fr,
                before,
                after,
                delay,
            });
        }
        for &i in order {
            let f = &frames[i];
            let states_match = writer.output.canvas == f.before;
            writer.original(
                i,
                &f.fi,
                &f.fr,
                &f.after,
                f.delay,
                states_match,
                force(i),
                &mut report,
            )?;
        }
        report.reordered = order.iter().enumerate().any(|(pos, &i)| pos != i);
        report.frames_in = frames.len();
        report.frames_out = writer.finish()?;
        return Ok(report);
    }

    let mut copies: HashMap<usize, usize> = HashMap::new();
    for &i in &duplicates {
        *copies.entry(i).or_default() += 1;
    }

    let insert = |writer: &mut FrameWriter,
                  report: &mut ModReport,
                  img: &RgbaImage,
                  ins: &FrameInsert,
                  default_delay: u16|
     -> Result<(), String> {
        let (quantized, conflict) = writer.image(img, ins.delay.unwrap_or(default_delay))?;
        report.inserted += 1;
        if quantized {
            report.quantized_inserts.push(ins.image.clone());
        }
        if conflict {
            report.insert_transparency_warnings.push(ins.image.clone());
        }
        Ok(())
    };

    let mut index = 0usize;
    let mut last_delay = 10u16;
    while let Some((fi, fr)) = read_frame_pair(&mut idx_reader, &mut rgba_reader)? {
        let delay = delay_for(index, fi.delay);

        // New images go before this frame. If this frame is also deleted, the images replace
        // it, so its delay isn't merged into them.
        let new_images = inserts_at.remove(&index).unwrap_or_default();
        let replaced = !new_images.is_empty();
        for (img, ins) in new_images {
            insert(&mut writer, &mut report, &img, ins, delay)?;
        }

        // Canvas before this frame, in the original and in our output.
        let states_match = writer.output.canvas == original.canvas;
        let n_copies = copies.get(&index).copied().unwrap_or(0);
        let before = (n_copies > 0).then(|| original.canvas.clone());
        let snapshot = original.draw(&fr);

        if deleted.contains(&index) {
            if opts.delete_mode == DeleteMode::MergeDelay && !replaced {
                writer.merge_delay(delay);
            }
        } else {
            let after = &original.canvas;
            writer.original(
                index,
                &fi,
                &fr,
                after,
                delay,
                states_match,
                force(index),
                &mut report,
            )?;
            if let Some(before) = &before {
                for _ in 0..n_copies {
                    let states_match = writer.output.canvas == *before;
                    writer.original(
                        index,
                        &fi,
                        &fr,
                        after,
                        delay,
                        states_match,
                        force(index),
                        &mut report,
                    )?;
                    report.duplicated.push(index);
                }
            }
        }

        original.dispose(&fr, snapshot);
        last_delay = delay;
        index += 1;
    }

    // Images appended after the last frame.
    for (img, ins) in std::mem::take(&mut inserts_at).into_values().flatten() {
        insert(&mut writer, &mut report, &img, ins, last_delay)?;
    }

    report.frames_in = index;
    report.frames_out = writer.finish()?;
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

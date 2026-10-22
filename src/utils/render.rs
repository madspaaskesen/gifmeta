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
/// Works row by row on the raw buffers (no per-pixel bounds checks).
pub(crate) fn draw(canvas: &mut RgbaImage, frame: &gif::Frame) {
    let (cw, ch) = canvas.dimensions();
    let (left, top) = (frame.left as u32, frame.top as u32);
    if left >= cw || top >= ch {
        return;
    }
    let fw = frame.width as usize;
    let cols = (frame.width as u32).min(cw - left) as usize;
    let rows = (frame.height as u32).min(ch - top) as usize;
    let canvas_stride = cw as usize * 4;
    let buf: &mut [u8] = canvas.as_mut();

    for y in 0..rows {
        let src_start = y * fw * 4;
        let Some(src) = frame.buffer.get(src_start..src_start + cols * 4) else {
            break;
        };
        let dst_start = (top as usize + y) * canvas_stride + left as usize * 4;
        let dst = &mut buf[dst_start..dst_start + cols * 4];
        for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            // Transparent pixels let the canvas below show through.
            if s[3] != 0 {
                d.copy_from_slice(s);
            }
        }
    }
}

/// Clears the frame's rectangle on the canvas (disposal `Background`).
pub(crate) fn clear_rect(canvas: &mut RgbaImage, frame: &gif::Frame) {
    let (cw, ch) = canvas.dimensions();
    let (left, top) = (frame.left as u32, frame.top as u32);
    if left >= cw || top >= ch {
        return;
    }
    let cols = (frame.width as u32).min(cw - left) as usize;
    let rows = (frame.height as u32).min(ch - top) as usize;
    let canvas_stride = cw as usize * 4;
    let buf: &mut [u8] = canvas.as_mut();
    for y in 0..rows {
        let start = (top as usize + y) * canvas_stride + left as usize * 4;
        buf[start..start + cols * 4].fill(0);
    }
}

/// Minimal canvas that replays frames exactly like a viewer (used for shadow simulation when
/// deleting / re-keying frames). Frames must be RGBA-decoded.
pub(crate) struct Compositor {
    pub canvas: RgbaImage,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            canvas: RgbaImage::from_pixel(width, height, CLEAR),
        }
    }

    /// Draws `frame`; returns a snapshot of the canvas *before* drawing when the frame uses
    /// disposal `Previous` (pass it to [`Compositor::dispose`]).
    pub fn draw(&mut self, frame: &gif::Frame) -> Option<RgbaImage> {
        let snapshot = (frame.dispose == DisposalMethod::Previous).then(|| self.canvas.clone());
        draw(&mut self.canvas, frame);
        snapshot
    }

    /// Applies the frame's disposal method (prepares the canvas for the next frame).
    pub fn dispose(&mut self, frame: &gif::Frame, snapshot: Option<RgbaImage>) {
        match frame.dispose {
            DisposalMethod::Background => clear_rect(&mut self.canvas, frame),
            DisposalMethod::Previous => {
                if let Some(s) = snapshot {
                    self.canvas = s;
                }
            }
            DisposalMethod::Any | DisposalMethod::Keep => {}
        }
    }
}

/// One rendered frame from [`FrameRenderer`].
#[derive(Debug, Clone)]
pub struct RenderedFrame {
    /// 0-based frame index.
    pub index: usize,
    /// Frame delay in centiseconds (as stored in the file).
    pub delay_cs: u16,
    /// Full-canvas RGBA image.
    pub image: RgbaImage,
}

/// Renders all frames of a GIF in **one pass** (frame *n* builds on the canvas of frame *n-1*),
/// instead of replaying the animation from the start for every frame.
///
/// It is an iterator, so frames can be processed one at a time without holding all of
/// them in memory:
///
/// ```
/// use gifmeta::{FrameRenderer, PreviewMode};
/// let renderer = FrameRenderer::open("tests/testdata/optimize/moving-square-optimized.gif", PreviewMode::Composited).unwrap();
/// for frame in renderer {
///     let frame = frame.unwrap();
///     assert_eq!(frame.image.dimensions(), (100, 100));
/// }
/// ```
pub struct FrameRenderer {
    reader: gif::Decoder<BufReader<File>>,
    canvas: RgbaImage,
    mode: PreviewMode,
    index: usize,
    done: bool,
}

impl FrameRenderer {
    /// Opens a GIF for rendering in the given mode.
    pub fn open<P: AsRef<Path>>(path: P, mode: PreviewMode) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("File open failed: {e}"))?;
        let mut decoder = DecodeOptions::new();
        decoder.set_color_output(gif::ColorOutput::RGBA);
        let reader = decoder
            .read_info(BufReader::new(file))
            .map_err(|e| format!("GIF decode failed: {e}"))?;
        let canvas = RgbaImage::from_pixel(reader.width() as u32, reader.height() as u32, CLEAR);
        Ok(Self {
            reader,
            canvas,
            mode,
            index: 0,
            done: false,
        })
    }

    /// Canvas (logical screen) size.
    pub fn dimensions(&self) -> (u32, u32) {
        self.canvas.dimensions()
    }
}

impl FrameRenderer {
    /// Decodes the next frame and updates the canvas. Only builds an output image when
    /// `emit` is true — skipped frames cost no extra copies (used by `nth`).
    fn advance(&mut self, emit: bool) -> Option<Result<RenderedFrame, String>> {
        if self.done {
            return None;
        }
        let frame = match self.reader.read_next_frame() {
            Ok(Some(frame)) => frame,
            Ok(None) => {
                self.done = true;
                return None;
            }
            Err(e) => {
                self.done = true;
                return Some(Err(format!("Read frame error: {e}")));
            }
        };

        let image = match self.mode {
            // Raw view doesn't depend on earlier frames → skipped frames need no work at all.
            PreviewMode::Raw => emit.then(|| {
                let (w, h) = self.canvas.dimensions();
                let mut raw = RgbaImage::from_pixel(w, h, CLEAR);
                draw(&mut raw, frame);
                raw
            }),
            PreviewMode::Composited => {
                // Snapshot before drawing, for disposal `Previous`.
                let snapshot =
                    (frame.dispose == DisposalMethod::Previous).then(|| self.canvas.clone());
                draw(&mut self.canvas, frame);
                let out = emit.then(|| self.canvas.clone());
                // Prepare the canvas for the next frame.
                match frame.dispose {
                    DisposalMethod::Background => clear_rect(&mut self.canvas, frame),
                    DisposalMethod::Previous => {
                        if let Some(s) = snapshot {
                            self.canvas = s;
                        }
                    }
                    DisposalMethod::Any | DisposalMethod::Keep => {}
                }
                out
            }
        };

        let index = self.index;
        self.index += 1;
        Some(Ok(RenderedFrame {
            index,
            delay_cs: frame.delay,
            image: image.unwrap_or_default(),
        }))
    }
}

impl Iterator for FrameRenderer {
    type Item = Result<RenderedFrame, String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance(true)
    }

    /// Skips `n` frames cheaply (no image copies), then renders the next one.
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        for _ in 0..n {
            match self.advance(false)? {
                Ok(_) => {}
                Err(e) => return Some(Err(e)),
            }
        }
        self.advance(true)
    }
}

/// Renders every frame of the GIF in one pass. Holds all frames in memory — for large GIFs
/// prefer iterating a [`FrameRenderer`].
pub fn render_all_frames<P: AsRef<Path>>(
    path: P,
    mode: PreviewMode,
) -> Result<Vec<RenderedFrame>, String> {
    FrameRenderer::open(path, mode)?.collect()
}

/// Renders frame `frame_index` of the GIF at `path` as a full-canvas RGBA image.
pub fn render_frame<P: AsRef<Path>>(
    path: P,
    frame_index: usize,
    mode: PreviewMode,
) -> Result<RgbaImage, String> {
    // Raw mode doesn't need earlier frames composited, but they still have to be decoded
    // to reach the frame — the renderer skips the compositing work for them.
    let mut renderer = FrameRenderer::open(path, mode)?;
    match renderer.nth(frame_index) {
        Some(frame) => frame.map(|f| f.image),
        None => Err(format!("Frame index {} out of bounds", frame_index)),
    }
}

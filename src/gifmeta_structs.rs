use serde::Serialize;

// Define a basic GifError type if not already defined elsewhere
#[derive(Debug)]
pub struct GifError(pub String);

/// Metadata summary for a GIF file.
#[derive(Debug, PartialEq, Serialize)]
pub struct GifMetadata {
    pub width: u16,
    pub height: u16,
    pub frame_count: u32,
    pub total_duration_cs: u32, // centiseconds
    pub loop_count: u16,
    pub frames: Vec<FrameMeta>,
    pub has_global_palette: bool,
    pub global_palette_size: Option<usize>,
    pub uses_transparency: bool,
    /// Number of frames that do not depend on earlier frames (see [`FrameMeta::is_keyframe`]).
    pub keyframe_count: u32,
    /// `true` when at least one frame is a delta frame (only stores changes since earlier frames).
    pub is_delta_optimized: bool,
}

/// What happens to a frame's area before the next frame is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Disposal {
    /// Unspecified — viewers treat it like `Keep`.
    Any,
    /// Leave the frame on the canvas; the next frame is drawn on top.
    Keep,
    /// Clear the frame's area (to transparent) before the next frame.
    Background,
    /// Restore the canvas to how it was before this frame.
    Previous,
}

impl From<gif::DisposalMethod> for Disposal {
    fn from(d: gif::DisposalMethod) -> Self {
        match d {
            gif::DisposalMethod::Any => Disposal::Any,
            gif::DisposalMethod::Keep => Disposal::Keep,
            gif::DisposalMethod::Background => Disposal::Background,
            gif::DisposalMethod::Previous => Disposal::Previous,
        }
    }
}

/// Metadata for an individual frame.
#[derive(Debug, PartialEq, Serialize)]
pub struct FrameMeta {
    pub index: usize,
    pub delay_cs: u16,
    pub transparent_index: Option<u8>,
    /// Frame rectangle on the logical screen.
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub disposal: Disposal,
    /// `true` when at least one pixel actually uses the transparent index.
    pub uses_transparent_pixels: bool,
    /// `true` when the frame is independent of earlier frames (a full image),
    /// `false` for a delta frame that relies on what is already on the canvas.
    pub is_keyframe: bool,
}

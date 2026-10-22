use serde::Serialize;

// Define a basic GifError type if not already defined elsewhere
#[derive(Debug)]
pub struct GifError(pub String);

/// How often a GIF plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    /// Loop forever.
    Infinite,
    /// Play exactly this many times (`Times(1)` = once, no loop block in the file).
    Times(u16),
}

impl LoopMode {
    /// Converts the user-facing number used by `loop_count` (`0` = infinite, `n` = n times).
    pub fn from_count(count: u16) -> Self {
        match count {
            0 => LoopMode::Infinite,
            n => LoopMode::Times(n),
        }
    }

    /// User-facing number: `0` for infinite, otherwise how many times it plays.
    pub fn to_count(self) -> u16 {
        match self {
            LoopMode::Infinite => 0,
            LoopMode::Times(n) => n,
        }
    }

    pub fn is_infinite(self) -> bool {
        self == LoopMode::Infinite
    }
}

impl std::fmt::Display for LoopMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopMode::Infinite => write!(f, "∞ infinite"),
            LoopMode::Times(1) => write!(f, "1× (plays once)"),
            LoopMode::Times(n) => write!(f, "{}×", n),
        }
    }
}

/// Metadata summary for a GIF file.
///
/// `#[non_exhaustive]`: new fields may be added in minor releases.
#[derive(Debug, PartialEq, Serialize)]
#[non_exhaustive]
pub struct GifMetadata {
    pub width: u16,
    pub height: u16,
    pub frame_count: u32,
    pub total_duration_cs: u32, // centiseconds
    /// How many times the GIF plays: `0` = infinite, `1` = once, `n` = n times.
    pub loop_count: u16,
    /// `true` when the GIF loops forever (same as `loop_count == 0`, but explicit).
    pub loop_infinite: bool,
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
///
/// `#[non_exhaustive]`: new fields may be added in minor releases.
#[derive(Debug, PartialEq, Serialize)]
#[non_exhaustive]
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

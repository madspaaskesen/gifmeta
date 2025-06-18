// Define a basic GifError type if not already defined elsewhere
#[derive(Debug)]
pub struct GifError(pub String);

/// Metadata summary for a GIF file.
#[derive(Debug, PartialEq)]
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
}

/// Metadata for an individual frame.
#[derive(Debug, PartialEq)]
pub struct FrameMeta {
    pub index: usize,
    pub delay_cs: u16,
    pub transparent_index: Option<u8>,
}

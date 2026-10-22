pub mod commands;

use std::{collections::HashMap, path::PathBuf};

pub mod gifmeta_structs;
pub mod utils;

/// Prints detailed metadata about the provided GIF file.
///
/// This includes dimensions, frame count, duration, and loop count.
///
/// # Arguments
///
/// * `path` - Path to the `.gif` file to inspect.
///
/// # Example
/// ```
/// use gifmeta::get_metadata;
/// get_metadata(&std::path::PathBuf::from("tests/testdata/loop/1.gif"), false);
/// ```
pub fn get_metadata(
    path: &PathBuf,
    show_frames: bool,
) -> Result<gifmeta_structs::GifMetadata, String> {
    match commands::info::get_metadata(path, show_frames) {
        Ok(meta) => Ok(meta),
        Err(e) => Err(e),
    }
}

pub use commands::modify::ModOptions;
pub use gifmeta_structs::LoopMode;

/// Modifies a GIF: loop setting, delay for all frames and/or per-frame delays.
///
/// Returns the path that was written (`opts.output`, or a temp file when `None`).
///
/// # Example
/// ```
/// use gifmeta::{modify_gif, LoopMode, ModOptions};
/// let out = std::env::temp_dir().join("gifmeta-doc-modify.gif");
/// modify_gif(
///     &"tests/testdata/loop/loop-once.gif".into(),
///     ModOptions {
///         output: Some(out.clone()),
///         loop_mode: Some(LoopMode::Times(3)),
///         delay_all: Some(10),
///         ..Default::default()
///     },
/// ).unwrap();
/// assert_eq!(gifmeta::get_metadata(&out, false).unwrap().loop_count, 3);
/// ```
pub fn modify_gif(input: &PathBuf, opts: ModOptions) -> Result<PathBuf, String> {
    if opts.is_empty() {
        return Err("No modification parameters provided. Use at least one of: \
             --loop-count, --infinite, --delay or --delays"
            .into());
    }
    commands::modify::apply(input, &opts)
}

/// Old API. `loop_count`: `0` = infinite, `n` = plays n times.
#[deprecated(
    since = "0.5.0",
    note = "use `modify_gif(input, ModOptions { .. })` with `LoopMode`"
)]
pub fn mod_gif(
    input: &PathBuf,
    output: Option<PathBuf>,
    loop_count: Option<u16>,
    delay_all: Option<u16>,
    delays: Option<HashMap<usize, u16>>,
) -> Result<(), String> {
    let opts = ModOptions {
        output,
        loop_mode: loop_count.map(LoopMode::from_count),
        delay_all,
        delays,
    };
    match modify_gif(input, opts) {
        Ok(out) => {
            println!("✅ Modifications applied → {}", out.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            Err(e)
        }
    }
}

/// Returns the loop setting of a GIF as a [`LoopMode`].
/// A GIF without a loop block plays once (`LoopMode::Times(1)`).
///
/// # Example
/// ```
/// use gifmeta::{get_loop_mode, LoopMode};
/// let mode = get_loop_mode(&"tests/testdata/loop/loop-once.gif".into()).unwrap();
/// assert_eq!(mode, LoopMode::Times(1));
/// ```
pub fn get_loop_mode(path: &PathBuf) -> Result<LoopMode, String> {
    utils::loop_count::read_loop_mode(path)
}

/// Sets the loop setting of a GIF. If `output` is `None`, the input file is overwritten.
pub fn set_loop_mode(
    path: &PathBuf,
    mode: LoopMode,
    output: Option<PathBuf>,
) -> Result<(), String> {
    utils::loop_count::set_loop_mode(path, mode, output)
}

/// Prints and returns how many times the GIF plays (`0` = infinite, `1` = once).
///
/// # Example
/// ```
/// use gifmeta::get_loop_count;
/// get_loop_count(&std::path::PathBuf::from("tests/testdata/loop/2.gif"));
/// ```
pub fn get_loop_count(path: &PathBuf) -> Result<u16, String> {
    let mode = utils::loop_count::read_loop_mode(path)?;
    println!("Loop count: {}", mode);
    Ok(mode.to_count())
}

/// Sets a fixed frame delay (in centiseconds) for all frames in the GIF.
///
/// Currently a placeholder (not yet implemented).
///
/// # Arguments
///
/// * `path` - Path to the input `.gif` file.
/// * `delay` - Desired delay in centiseconds (e.g., `10` = 100ms).
/// * `output` - Optional path for the output file. If `None`, overwrites original.
///
/// # Example
/// ```
/// use gifmeta::set_frame_delay;
/// set_frame_delay(&"tests/testdata/1.gif".into(), 10, None);
/// ```
pub fn set_frame_delay(path: &PathBuf, delay: u16, output: Option<PathBuf>) -> Result<u16, String> {
    let output_clone = output.clone();
    println!(
        "(stub) Setting delay {} for: {:?} → {:?}",
        delay, path, output
    );
    match utils::set_frame_delay::set_frame_delay(path, delay, output) {
        Ok(_) => {
            println!("New delay: {} saved to: {:?}", delay, output_clone);
            Ok(delay)
        }
        Err(e) => {
            eprintln!("Failed to set delay: {}", e);
            Err(e)
        }
    }
}

/// Sets how many times the GIF plays: `0` = infinite, `1` = once, `n` = n times.
/// If `output` is `None`, the input file is overwritten.
///
/// Before 0.5.0 this wrote the raw file value (so `3` played 4 times) — it now matches
/// `mod --loop-count` and `info`.
///
/// # Example
/// ```
/// use gifmeta::set_loop_count;
/// let out = std::env::temp_dir().join("gifmeta-doc-loop.gif");
/// set_loop_count(&"tests/testdata/loop/2.gif".into(), 3, Some(out.clone())).unwrap();
/// assert_eq!(gifmeta::get_metadata(&out, false).unwrap().loop_count, 3);
/// ```
pub fn set_loop_count(path: &PathBuf, count: u16, output: Option<PathBuf>) -> Result<u16, String> {
    utils::loop_count::set_loop_count(path, count, output)?;
    Ok(count)
}

/// Displays the delay (in centiseconds) of each frame in the GIF.
///
/// Useful for inspecting frame timing before editing or modifying.
/// Prints one line per frame to stdout.
///
/// # Arguments
///
/// * `path` - Path to the input `.gif` file
///
/// # Returns
///
/// * `Ok(())` on success
/// * `Err(String)` if the file cannot be read or decoded
///
/// # Example
/// ```
/// use gifmeta::show_frame_delays;
/// show_frame_delays(&"tests/testdata/loop/2.gif".into());
/// ```
pub fn show_frame_delays(path: &PathBuf) {
    println!("show_frame_delays {:?}", path);
}

/// Sets custom delays for specific frames in the GIF.
///
/// Only the specified frame indices are updated with new delay values.
/// All other frames retain their original timing.
/// Delays are measured in centiseconds (1cs = 10ms).
///
/// # Arguments
///
/// * `input` - Path to the input `.gif` file
/// * `frame_numbers` - A list of 0-based frame indices to modify
/// * `delay_values` - A list of matching delays in centiseconds
/// * `output` - Optional path to save the modified `.gif`
///             If `None`, will overwrite the original input file
///
/// # Returns
///
/// * `Ok(())` on success
/// * `Err(String)` if the frame count doesn't match, or writing fails
///
/// # Example
/// ```
/// use gifmeta::set_selected_frame_delays;
/// set_selected_frame_delays(
///     &"tests/testdata/loop/2.gif".into(),
///     vec![0, 2],
///     vec![5, 20],
///     Some("out.gif".into())
/// );
/// ```
pub fn set_selected_frame_delays(
    input: &PathBuf,
    frame_numbers: Vec<usize>,
    delay_values: Vec<u16>,
    output: Option<PathBuf>,
) {
    println!(
        "set_selected_frame_delays {:?} with: {:?}, {:?} → {:?}",
        input, frame_numbers, delay_values, output
    );
}

/// Extracts a single frame from a GIF and returns it as PNG-encoded bytes.
///
/// This function loads the specified frame from the GIF file at `path`,
/// encodes it as a PNG image in memory, and returns the result as a `Vec<u8>`.
///
/// # Arguments
///
/// * `path` - Path to the `.gif` file
/// * `frame` - Zero-based index of the frame to extract
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - PNG-encoded image data of the frame
/// * `Err(String)` - Error message if the file cannot be read, decoded, or the frame is out of bounds
///
/// # Example
///
/// ```rust
/// use gifmeta::get_frame_image;
/// let png_data = get_frame_image("tests/testdata/loop/2.gif".into(), 0).unwrap();
/// std::fs::write("tests/output/frame0.png", &png_data).unwrap();
/// ```
pub fn get_frame_image(path: String, frame: usize) -> Result<Vec<u8>, String> {
    utils::extract_frame_as_png::extract_frame_as_png(&path, frame)
}

/// Like [`get_frame_image`], but lets you choose how the frame is rendered.
///
/// * [`PreviewMode::Composited`] – "pretty" view: the frame as it looks when the GIF plays.
/// * [`PreviewMode::Raw`] – "raw" view: only the pixels stored in this frame, at its position
///   on a transparent canvas. Useful to see what a delta frame of an optimized GIF contains.
///
/// # Example
///
/// ```rust
/// use gifmeta::{get_frame_image_with_mode, PreviewMode};
/// let png = get_frame_image_with_mode(
///     "tests/testdata/optimize/moving-square-optimized.gif".into(),
///     1,
///     PreviewMode::Raw,
/// ).unwrap();
/// assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");
/// ```
pub fn get_frame_image_with_mode(
    path: String,
    frame: usize,
    mode: PreviewMode,
) -> Result<Vec<u8>, String> {
    utils::extract_frame_as_png::extract_frame_as_png_with_mode(&path, frame, mode)
}

pub use utils::render::{render_all_frames, FrameRenderer, PreviewMode, RenderedFrame};

/// Renders **all** frames in one pass and returns them as PNG bytes, in order.
///
/// Much faster than calling [`get_frame_image_with_mode`] for every frame, which replays the
/// animation from the start each time. Holds all frames in memory — for very large GIFs,
/// iterate a [`FrameRenderer`] instead.
///
/// # Example
/// ```
/// use gifmeta::{get_all_frame_images, PreviewMode};
/// let pngs = get_all_frame_images("tests/testdata/optimize/moving-square-optimized.gif".into(), PreviewMode::Composited).unwrap();
/// assert_eq!(pngs.len(), 5);
/// assert_eq!(&pngs[0][0..8], b"\x89PNG\r\n\x1a\n");
/// ```
pub fn get_all_frame_images(path: String, mode: PreviewMode) -> Result<Vec<Vec<u8>>, String> {
    FrameRenderer::open(&path, mode)?
        .map(|frame| frame.and_then(|f| utils::extract_frame_as_png::encode_png(&f.image)))
        .collect()
}

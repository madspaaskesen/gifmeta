pub mod commands;

use std::{collections::HashMap, path::PathBuf};

pub mod gifmeta_structs;
pub mod loop_count;

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
/// get_metadata(&std::path::PathBuf::from("tests/testdata/1.gif"));
/// ```
pub fn get_metadata(
    path: &PathBuf,
    show_frames: bool,
) -> Result<gifmeta_structs::GifMetadata, String> {
    match commands::info::get_metadata(path) {
        Ok(meta) => {
            println!("✅ Metadata for : {}\n", path.display());
            println!("🖼️ Dimensions   : {} × {}", meta.width, meta.height);
            println!("🖼️ Frame count  : {}", meta.frame_count);
            println!("⏱️ Duration     : {} centiseconds", meta.total_duration_cs);
            println!("🔄Loop         : {:?}", meta.loop_count);
            println!("🎨Has palette  : {:?}", meta.has_global_palette);
            println!("🎨Palette size : {:?}", meta.global_palette_size.unwrap());
            println!("🎨Transparency : {:?}", meta.uses_transparency);

            if show_frames {
                println!("\n🧩Frame delays :");
                for frame in &meta.frames {
                    println!(
                        "  • Frame {:>3}: {:>4} cs{}",
                        frame.index,
                        frame.delay_cs,
                        match frame.transparent_index {
                            Some(idx) => format!(" (transparent index: {})", idx),
                            None => "".to_string(),
                        }
                    );
                }
            }

            Ok(meta)
        }
        Err(e) => Err(e),
    }
}

pub fn mod_gif(
    input: &PathBuf,
    output: Option<PathBuf>,
    loop_count: Option<u16>,
    delay_all: Option<u16>,
    delays: Option<HashMap<usize, u16>>,
) -> Result<(), String> {
    Ok(())
}

/// Prints the loop count of the provided GIF file.
///
/// # Arguments
///
/// * `path` - Path to the `.gif` file.
///
/// # Example
/// ```
/// use gifmeta::get_loop_count;
/// get_loop_count(&std::path::PathBuf::from("tests/testdata/2.gif"));
/// ```
pub fn get_loop_count(path: &PathBuf) -> Result<u16, String> {
    match loop_count::extract_loop_count(path) {
        Ok(count) => {
            println!("Loop count: {}", count);
            Ok(count)
        }
        Err(e) => {
            eprintln!("Failed to extract loop count: {}", e);
            Err(e)
        }
    }
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
    match commands::set_frame_delay::set_frame_delay(path, delay, output) {
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

/// Sets the loop count metadata in a GIF file.
///
/// Currently a placeholder (not yet implemented).
///
/// # Arguments
///
/// * `path` - Path to the input `.gif` file.
/// * `count` - Number of loops (e.g., `0` = infinite).
/// * `output` - Optional path for the output file. If `None`, overwrites original.
///
/// # Example
/// ```
/// use gifmeta::set_loop_count;
/// set_loop_count(&"tests/testdata/2.gif".into(), 3, None);
/// ```
pub fn set_loop_count(path: &PathBuf, count: u16, output: Option<PathBuf>) -> Result<u16, String> {
    let output_clone = output.clone();
    match loop_count::set_loop_count(path, count, output) {
        Ok(_) => {
            println!("New loop count: {} saved to: {:?}", count, output_clone);
            Ok(count)
        }
        Err(e) => {
            eprintln!("Failed to extract loop count: {}", e);
            Err(e)
        }
    }
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
/// show_frame_delays(&"tests/testdata/2.gif".into()).unwrap();
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
///     &"tests/testdata/2.gif".into(),
///     vec![0, 2],
///     vec![5, 20],
///     Some("out.gif".into())
/// ).unwrap();
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

pub mod commands;

use std::path::PathBuf;

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
}

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
pub fn get_metadata(path: &PathBuf) -> Result<GifMetadata, String> {
    match commands::info::get_metadata(path) {
        Ok(meta) => {
            println!("✅ Metadata for : {}\n", path.display());
            println!("🖼️ Dimensions   : {} × {}", meta.width, meta.height);
            println!("🖼️ Frame count  : {}", meta.frame_count);
            println!("⏱️ Duration     : {} centiseconds", meta.total_duration_cs);
            Ok(meta)
        }
        Err(e) => Err(e),
    }
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
    match commands::loop_count::extract_loop_count(path) {
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
pub fn set_frame_delay(path: &PathBuf, delay: u16, output: Option<PathBuf>) {
    println!("(stub) Setting delay {} for: {:?} → {:?}", delay, path, output);
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
pub fn set_loop_count(path: &PathBuf, count: u16, output: Option<PathBuf>) {
    println!("(stub) Setting loop count {} for: {:?} → {:?}", count, path, output);
}

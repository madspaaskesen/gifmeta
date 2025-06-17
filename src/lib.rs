pub mod commands;

use std::path::PathBuf;

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
/// use gifmeta::print_info;
/// print_info(&std::path::PathBuf::from("tests/testdata/1.gif"));
/// ```
pub fn print_info(path: &PathBuf) {
    commands::info::show_info(path);
}

/// Prints the loop count of the provided GIF file.
///
/// # Arguments
///
/// * `path` - Path to the `.gif` file.
/// 
/// # Example
/// ```
/// use gifmeta::print_loop_count;
/// print_loop_count(&std::path::PathBuf::from("tests/testdata/2.gif"));
/// ```
pub fn print_loop_count(path: &PathBuf) {
    commands::loop_count::show_loop_count(path);
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

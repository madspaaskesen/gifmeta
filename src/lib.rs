pub mod commands;

use std::path::PathBuf;

/// Print metadata about the GIF file
pub fn print_info(path: &PathBuf) {
    commands::info::show_info(path);
}

pub fn print_loop_count(path: &PathBuf) {
    commands::get_loop::print_loop_count(path);
}

/// Set a fixed delay for all frames in the GIF
pub fn set_frame_delay(path: &PathBuf, delay: u16, output: Option<PathBuf>) {
    println!("(stub) Setting delay {} for: {:?} → {:?}", delay, path, output);
}

/// Set loop count metadata in the GIF
pub fn set_loop_count(path: &PathBuf, count: u16, output: Option<PathBuf>) {
    println!("(stub) Setting loop count {} for: {:?} → {:?}", count, path, output);
}

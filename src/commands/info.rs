use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use gif::DecodeOptions; 

pub fn show_info(path: &Path) {
    let file = File::open(path).expect("❌ Failed to open file");
    let mut decoder = DecodeOptions::new();
    decoder.set_color_output(gif::ColorOutput::Indexed);
    decoder.allow_unknown_blocks(true);
    let mut reader = decoder.read_info(BufReader::new(file)).expect("❌ Failed to decode");

    let mut frame_count = 0;
    let width = reader.width();
    let height = reader.height();
    let mut total_duration = 0;
    
    while let Some(frame) = reader.read_next_frame().expect("❌ Error reading frame") {
        frame_count += 1;
        total_duration += frame.delay as u32;
    }

    //let loop_count = crate::commands::loop_count::extract_loop_count(path).ok();

    println!("✅ Metadata for : {}\n", path.display());
    println!("🖼️ Dimensions   : {} × {}", width, height);
    println!("🖼️ Frame count  : {}", frame_count);
    println!("⏱️ Duration     : {} centiseconds", total_duration);
    //println!("🔁 Loop count   : {}", loop_count.unwrap_or(1));
}

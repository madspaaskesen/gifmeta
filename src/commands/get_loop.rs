use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn extract_loop_count<P: AsRef<Path>>(path: P) -> Result<u16, String> {
    let mut file = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let mut i = 0;
    while i < buffer.len() {
        if buffer[i] == 0x21 && buffer[i + 1] == 0xFF {
            let block_size = buffer[i + 2] as usize;
            if &buffer[i + 3..i + 3 + 11] == b"NETSCAPE2.0" {
                let loop_data_index = i + 3 + 11 + 2;
                let loop_count = u16::from_le_bytes([buffer[loop_data_index], buffer[loop_data_index + 1]]);
                return Ok(loop_count);
            }
        }
        i += 1;
    }

    Err("Loop count not found".into())
}
pub fn print_loop_count(path: &Path) {
    match extract_loop_count(path) {
        Ok(count) => println!("Loop count: {}", count),
        Err(e) => eprintln!("Failed to extract loop count: {}", e),
    }
}
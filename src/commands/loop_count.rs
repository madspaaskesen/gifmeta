use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::path::PathBuf;

pub fn extract_loop_count<P: AsRef<Path>>(path: P) -> Result<u16, String> {
    let mut file = BufReader::new(File::open(path).map_err(|e| e.to_string())?);
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

    let mut i = 0;
    while i < buffer.len() {
        if buffer[i] == 0x21 && buffer[i + 1] == 0xFF {
            let _block_size = buffer[i + 2] as usize;
            if &buffer[i + 3..i + 3 + 11] == b"NETSCAPE2.0" {
                let loop_data_index = i + 3 + 11 + 2;
                let loop_count =
                    u16::from_le_bytes([buffer[loop_data_index], buffer[loop_data_index + 1]]);
                return Ok(loop_count);
            }
        }
        i += 1;
    }

    Err("Loop count not found".into())
}

/// Sets the loop count metadata in a GIF file.
/// If NETSCAPE2.0 loop extension exists, it will be modified.
/// If not found, it will be inserted right after the GIF header.
pub fn set_loop_count(path: &PathBuf, count: u16, output: Option<PathBuf>) -> Result<(), String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let netscape_sig: &[u8] = b"NETSCAPE2.0";
    let mut i = 0;
    let mut found = false;

    while i + 19 < data.len() {
        // Look for the NETSCAPE2.0 signature inside an application extension block
        if data[i] == 0x21
            && data[i + 1] == 0xFF
            && data[i + 2] == 0x0B
            && &data[i + 3..i + 14] == netscape_sig
            && data[i + 14] == 0x03
            && data[i + 15] == 0x01
        {
            // Replace loop count
            let count_le = count.to_le_bytes();
            data[i + 16] = count_le[0];
            data[i + 17] = count_le[1];
            found = true;
            break;
        }
        i += 1;
    }

    if !found {
        // Insert a new loop block after the header
        // Find the end of the header: typically after the first image descriptor or GCT
        // For simplicity, we insert after logical screen descriptor (at offset 13 + GCT length)
        let gct_flag = data[10] & 0b1000_0000 != 0;
        let gct_size = if gct_flag {
            3 * (2_usize.pow(((data[10] & 0b0000_0111) + 1) as u32))
        } else {
            0
        };
        let insert_pos = 13 + gct_size;

        let mut new_block = vec![
            0x21, 0xFF, 0x0B, // Extension Introducer + Application Label
        ];
        new_block.extend_from_slice(netscape_sig); // "NETSCAPE2.0"
        new_block.extend_from_slice(&[
            0x03,
            0x01,
            count as u8,
            (count >> 8) as u8,
            0x00, // Block terminator
        ]);

        data.splice(insert_pos..insert_pos, new_block);
    }

    let out_path = output.unwrap_or_else(|| path.clone());
    let mut out_file =
        File::create(out_path).map_err(|e| format!("Failed to write file: {}", e))?;
    out_file
        .write_all(&data)
        .map_err(|e| format!("Failed to save GIF: {}", e))?;

    Ok(())
}

//! Reading and writing the loop setting (NETSCAPE2.0 application extension).
//!
//! How GIFs store it:
//!
//! | In the file        | Plays     | [`LoopMode`]       |
//! |--------------------|-----------|--------------------|
//! | no NETSCAPE block  | once      | `Times(1)`         |
//! | block, value `0`   | forever   | `Infinite`         |
//! | block, value `N`   | `N+1` ×   | `Times(N + 1)`     |

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;

use crate::gifmeta_structs::LoopMode;

const NETSCAPE: &[u8] = b"NETSCAPE2.0";
/// Total size of a NETSCAPE2.0 block incl. introducer and terminator.
const BLOCK_LEN: usize = 19;

/// Finds the byte offset of the NETSCAPE2.0 block, if any.
fn find_block(data: &[u8]) -> Option<usize> {
    (0..data.len().saturating_sub(BLOCK_LEN - 1)).find(|&i| {
        data[i] == 0x21
            && data[i + 1] == 0xFF
            && data[i + 2] == 0x0B
            && &data[i + 3..i + 14] == NETSCAPE
            && data[i + 14] == 0x03
            && data[i + 15] == 0x01
    })
}

fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, String> {
    let mut data = Vec::new();
    File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?
        .read_to_end(&mut data)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(data)
}

/// Reads the **raw** value stored in the NETSCAPE2.0 block (`0` = infinite, `N` = N extra
/// repeats). Returns `Ok(None)` when the GIF has no loop block (it plays once).
pub fn extract_raw_loop_value<P: AsRef<Path>>(path: P) -> Result<Option<u16>, String> {
    let data = read_file(path)?;
    Ok(find_block(&data).map(|i| u16::from_le_bytes([data[i + 16], data[i + 17]])))
}

/// Reads the loop setting of a GIF as a [`LoopMode`]. A GIF without a loop block plays once.
pub fn read_loop_mode<P: AsRef<Path>>(path: P) -> Result<LoopMode, String> {
    Ok(match extract_raw_loop_value(path)? {
        None => LoopMode::Times(1),
        Some(0) => LoopMode::Infinite,
        Some(n) => LoopMode::Times(n.saturating_add(1)),
    })
}

/// Writes `mode` into the GIF, patching, inserting or removing the NETSCAPE2.0 block.
///
/// * `Infinite` → block with value 0
/// * `Times(1)` → no block (plays once)
/// * `Times(n)` → block with value `n - 1`
///
/// If `output` is `None`, the input file is overwritten.
pub fn set_loop_mode(
    path: &PathBuf,
    mode: LoopMode,
    output: Option<PathBuf>,
) -> Result<(), String> {
    let mut data = read_file(path)?;
    if data.len() < 13 {
        return Err("File too small to be a GIF".into());
    }

    let raw: Option<u16> = match mode {
        LoopMode::Infinite => Some(0),
        LoopMode::Times(0) => return Err("Loop count must be at least 1".into()),
        LoopMode::Times(1) => None,
        LoopMode::Times(n) => Some(n - 1),
    };

    match (find_block(&data), raw) {
        (Some(i), Some(value)) => {
            let le = value.to_le_bytes();
            data[i + 16] = le[0];
            data[i + 17] = le[1];
        }
        (Some(i), None) => {
            data.drain(i..i + BLOCK_LEN);
        }
        (None, Some(value)) => {
            // Insert right after the logical screen descriptor + global colour table.
            let gct_flag = data[10] & 0b1000_0000 != 0;
            let gct_size = if gct_flag {
                3 * (1usize << ((data[10] & 0b0000_0111) + 1))
            } else {
                0
            };
            let insert_pos = (13 + gct_size).min(data.len());
            let mut block = vec![0x21, 0xFF, 0x0B];
            block.extend_from_slice(NETSCAPE);
            block.extend_from_slice(&[0x03, 0x01]);
            block.extend_from_slice(&value.to_le_bytes());
            block.push(0x00);
            data.splice(insert_pos..insert_pos, block);
        }
        (None, None) => {}
    }

    let out_path = output.unwrap_or_else(|| path.clone());
    File::create(out_path)
        .map_err(|e| format!("Failed to write file: {}", e))?
        .write_all(&data)
        .map_err(|e| format!("Failed to save GIF: {}", e))?;
    Ok(())
}

/// Reads the raw loop value. Errors when the GIF has no loop block.
#[deprecated(
    since = "0.5.0",
    note = "use `read_loop_mode` (a missing block means the GIF plays once)"
)]
pub fn extract_loop_count<P: AsRef<Path>>(path: P) -> Result<u16, String> {
    extract_raw_loop_value(path)?.ok_or_else(|| "Loop count not found".into())
}

/// Sets the loop setting using the user-facing convention (`0` = infinite, `n` = plays n times).
pub fn set_loop_count(path: &PathBuf, count: u16, output: Option<PathBuf>) -> Result<(), String> {
    set_loop_mode(path, LoopMode::from_count(count), output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncated_file_does_not_panic() {
        assert_eq!(find_block(&[0x21]), None);
        assert_eq!(find_block(&[0x21, 0xFF, 0x0B, b'N']), None);
        assert_eq!(find_block(&[]), None);
    }
}

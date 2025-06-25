#[cfg(test)]
mod tests {
    use gifmeta::utils::extract_frame_as_png::extract_frame_as_png;
    use image::ImageReader;
    use std::{io::Cursor, path::Path};

    #[test]
    fn test_preview_dimensions_match_gif() {
        let path = "tests/testdata/loop/2.gif";

        // Get expected dimensions from metadata once
        let original_meta = gifmeta::get_metadata(&Path::new(path).to_path_buf(), false).unwrap();
        let expected_w = original_meta.width as u32;
        let expected_h = original_meta.height as u32;

        // Loop over first 2 frames
        for frame_index in 0..=1 {
            let png_bytes =
                extract_frame_as_png(path, frame_index).expect("Failed to extract preview");

            let cursor = Cursor::new(png_bytes);
            let decoded = ImageReader::new(cursor)
                .with_guessed_format()
                .expect("Guess format failed")
                .decode()
                .expect("PNG decode failed");

            let w = decoded.width();
            let h = decoded.height();

            assert_eq!(w, expected_w, "Width mismatch on frame {}", frame_index);
            assert_eq!(h, expected_h, "Height mismatch on frame {}", frame_index);
        }
    }
}

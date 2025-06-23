use gifmeta::get_frame_image;
use std::fs;
use std::path::Path;

#[test]
fn test_extract_single_frame_as_png() {
    let path = "tests/testdata/loop/1.gif";
    let frame_index = 0;

    let result = get_frame_image(path.into(), frame_index);
    assert!(result.is_ok(), "Expected OK, got error: {:?}", result);

    let png_bytes = result.unwrap();
    // PNG starts with the 8-byte magic number: 89 50 4E 47 0D 0A 1A 0A
    assert_eq!(&png_bytes[0..8], b"\x89PNG\r\n\x1a\n", "Invalid PNG header");

    // Optional: Save to temp file for debugging
    let debug_path = Path::new("tests/output/test_frame0.png");
    fs::create_dir_all(debug_path.parent().unwrap()).unwrap();
    fs::write(debug_path, png_bytes).expect("Failed to write debug image");
}

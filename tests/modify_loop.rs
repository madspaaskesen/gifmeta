use std::fs;
use std::path::Path;
// use gifmeta::parse_csv; // Removed because there is no parse_csv in the root
use gifmeta::mod_gif;
use gifmeta::utils::parse_csv::parse_keyval_csv;

#[test]
fn test_set_loop_variants() {
    use gifmeta::utils::loop_count::{extract_loop_count, set_loop_count};

    let input_path = Path::new("tests/testdata/loop/loop-once.gif");

    for &loop_value in &[0u16, 1, 2] {
        let output_path_str = format!("tests/testdata/loop/modified-loop-{}.gif", loop_value);
        let output_path = Path::new(&output_path_str);

        // Clean up before test
        if output_path.exists() {
            fs::remove_file(output_path).unwrap();
        }

        // Act
        let result = set_loop_count(
            &input_path.to_path_buf(),
            loop_value,
            Some(output_path.to_path_buf()),
        );
        assert!(result.is_ok(), "Failed to set loop count {}", loop_value);

        // Assert
        let actual = extract_loop_count(output_path).unwrap();
        assert_eq!(
            actual, loop_value,
            "Loop count did not match for value {}",
            loop_value
        );
        assert!(
            output_path.exists(),
            "Output file was not created for loop count {}",
            loop_value
        );
    }
}

#[test]
fn test_set_loop_and_frame_delays() {
    // Arrange
    let input_path = Path::new("tests/testdata/loop/loop-once.gif");
    let output_path = Path::new("tests/testdata/loop/modified-loop-test.gif");

    // Ensure clean output file
    if output_path.exists() {
        fs::remove_file(output_path).unwrap();
    }

    // Act – call your CLI logic directly or through library
    let input_pathbuf = input_path.to_path_buf();
    let output_pathbuf = output_path.to_path_buf();

    let delays = Some("1=15".to_string());
    let delays_map = delays
        .as_ref()
        .map(|s| parse_keyval_csv(s))
        .transpose()
        .unwrap_or(None);

    let result = mod_gif(
        &input_pathbuf,
        Some(output_pathbuf.clone()),
        Some(3),
        Some(4),
        delays_map,
    );
    assert!(result.is_ok());

    // Optionally: check that file exists
    assert!(output_path.exists(), "Output file should be created");

    // Assert – verify loop count
    let output_result = gifmeta::get_metadata(&output_pathbuf.clone(), false);
    let output_data = output_result.ok();
    let loop_count = output_data.unwrap().loop_count;
    assert_eq!(loop_count, 3, "Expected loop count to be 0 (infinite)");
}

#[test]
fn test_mod_preserves_loop_count_if_not_specified() {
    // Arrange
    let input_path = Path::new("tests/testdata/loop/1.gif");
    let output_path = Path::new("tests/testdata/loop/loop-infinite-untouched.gif");

    // Clean output
    if output_path.exists() {
        fs::remove_file(output_path).unwrap();
    }

    // Get original loop count
    let original_meta = gifmeta::get_metadata(&input_path.to_path_buf(), false).unwrap();
    let original_loop = original_meta.loop_count;

    // Act – only change delay, not loop count
    let result = gifmeta::mod_gif(
        &input_path.to_path_buf(),
        Some(output_path.to_path_buf()),
        None,    // <-- no loop count
        Some(5), // set uniform delay
        None,    // no delay map
    );
    assert!(result.is_ok());

    // Assert
    let modified_meta = gifmeta::get_metadata(&output_path.to_path_buf(), false).unwrap();
    let modified_loop = modified_meta.loop_count;

    assert_eq!(
        original_loop, modified_loop,
        "Loop count should remain unchanged"
    );
}

use gifmeta::gifmeta_structs::Disposal;
use std::path::PathBuf;

fn meta(path: &str) -> gifmeta::gifmeta_structs::GifMetadata {
    gifmeta::get_metadata(&PathBuf::from(path), true).expect("metadata")
}

#[test]
fn full_frame_gif_has_only_keyframes() {
    let m = meta("tests/testdata/optimize/moving-square-full.gif");
    assert_eq!(m.frame_count, 5);
    assert_eq!(m.keyframe_count, 5);
    assert!(!m.is_delta_optimized);
    assert!(m.frames.iter().all(|f| f.is_keyframe));
}

#[test]
fn optimized_gif_has_delta_frames() {
    let m = meta("tests/testdata/optimize/moving-square-optimized.gif");
    assert_eq!(m.frame_count, 5);
    assert!(m.is_delta_optimized);
    assert_eq!(m.keyframe_count, 1);
    assert!(m.frames[0].is_keyframe);
    for f in &m.frames[1..] {
        assert!(!f.is_keyframe, "frame {} should be a delta frame", f.index);
        assert!(
            f.width < m.width,
            "delta frame {} should be a sub-rectangle",
            f.index
        );
    }
}

#[test]
fn offset_frame_is_delta() {
    let m = meta("tests/testdata/layout/offset-frames.gif");
    assert!(m.frames[0].is_keyframe);
    assert!(!m.frames[1].is_keyframe);
    assert_eq!((m.frames[1].left, m.frames[1].top), (20, 20));
}

#[test]
fn self_clearing_frames_are_keyframes() {
    // Every frame uses Background disposal, so each one is drawn on an empty canvas.
    let m = meta("tests/testdata/transparency/stars-transparent.gif");
    assert!(m.frames.iter().all(|f| f.disposal == Disposal::Background));
    assert_eq!(m.keyframe_count, m.frame_count);
    assert!(!m.is_delta_optimized);
}

#[test]
fn summary_is_computed_without_show_frames() {
    let m = gifmeta::get_metadata(
        &PathBuf::from("tests/testdata/optimize/moving-square-optimized.gif"),
        false,
    )
    .unwrap();
    assert!(m.frames.is_empty());
    assert_eq!(m.keyframe_count, 1);
    assert!(m.is_delta_optimized);
}

#[test]
fn json_contains_new_fields() {
    let m = meta("tests/testdata/optimize/moving-square-optimized.gif");
    let json = serde_json::to_value(&m).unwrap();
    assert_eq!(json["is_delta_optimized"], true);
    assert_eq!(json["frames"][1]["is_keyframe"], false);
    assert_eq!(json["frames"][1]["disposal"], "keep");
}

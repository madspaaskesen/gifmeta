use gifmeta::utils::render::{render_frame, PreviewMode};

const FULL: &str = "tests/testdata/optimize/moving-square-full.gif";
const OPTIMIZED: &str = "tests/testdata/optimize/moving-square-optimized.gif";

#[test]
fn composited_optimized_frames_match_full_frames() {
    for i in 0..5 {
        let full = render_frame(FULL, i, PreviewMode::Composited).unwrap();
        let opt = render_frame(OPTIMIZED, i, PreviewMode::Composited).unwrap();
        assert_eq!(full.dimensions(), opt.dimensions());
        let diff = full
            .pixels()
            .zip(opt.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(diff, 0, "frame {i}: {diff} pixels differ");
        // Background is opaque everywhere → no "ghost" transparent holes.
        assert!(opt.pixels().all(|p| p[3] == 255), "frame {i} has holes");
    }
}

#[test]
fn raw_delta_frame_only_contains_its_own_pixels() {
    let raw = render_frame(OPTIMIZED, 1, PreviewMode::Raw).unwrap();
    assert_eq!(raw.dimensions(), (100, 100));
    // Outside the frame rect (5,40 41×21) everything is transparent.
    assert_eq!(raw.get_pixel(0, 0)[3], 0);
    assert_eq!(raw.get_pixel(99, 99)[3], 0);
    // Most of the canvas is transparent in raw view of a delta frame.
    let opaque = raw.pixels().filter(|p| p[3] != 0).count();
    assert!(
        opaque > 0 && opaque < 100 * 100 / 4,
        "opaque pixels: {opaque}"
    );
}

#[test]
fn raw_and_composited_are_equal_for_first_frame() {
    let a = render_frame(OPTIMIZED, 0, PreviewMode::Raw).unwrap();
    let b = render_frame(OPTIMIZED, 0, PreviewMode::Composited).unwrap();
    assert_eq!(a, b);
}

#[test]
fn background_disposal_clears_between_frames() {
    // Every frame clears itself, so composited == raw for each frame.
    let path = "tests/testdata/transparency/stars-transparent.gif";
    for i in [1, 20, 60] {
        let a = render_frame(path, i, PreviewMode::Raw).unwrap();
        let b = render_frame(path, i, PreviewMode::Composited).unwrap();
        assert_eq!(a, b, "frame {i}");
    }
}

#[test]
fn out_of_bounds_frame_errors() {
    assert!(render_frame(OPTIMIZED, 99, PreviewMode::Composited).is_err());
}

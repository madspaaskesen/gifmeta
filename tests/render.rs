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

// ---------- FrameRenderer (single pass) ----------

use gifmeta::{get_all_frame_images, render_all_frames, FrameRenderer};

const STARS: &str = "tests/testdata/transparency/stars-transparent.gif";
const OFFSET: &str = "tests/testdata/layout/offset-frames.gif";

/// The single-pass renderer must give exactly the same pixels as rendering each frame alone.
/// For long GIFs only every `step`-th frame (+ the last) is compared against the slow path.
fn assert_renderer_matches_render_frame(path: &str, mode: PreviewMode) {
    let step = if path == STARS { 10 } else { 1 };
    let all = render_all_frames(path, mode).unwrap();
    let meta = gifmeta::get_metadata(&path.into(), true).unwrap();
    assert_eq!(all.len(), meta.frame_count as usize, "{path}: frame count");
    for (i, f) in all.iter().enumerate() {
        assert_eq!(f.index, i);
        assert_eq!(
            f.delay_cs, meta.frames[i].delay_cs,
            "{path}: delay of frame {i}"
        );
        if i % step != 0 && i != all.len() - 1 {
            continue;
        }
        let single = render_frame(path, i, mode).unwrap();
        assert!(f.image == single, "{path} {mode:?}: frame {i} differs");
    }
}

#[test]
fn renderer_matches_render_frame_composited() {
    for p in [OPTIMIZED, FULL, STARS, OFFSET] {
        assert_renderer_matches_render_frame(p, PreviewMode::Composited);
    }
}

#[test]
fn renderer_matches_render_frame_raw() {
    for p in [OPTIMIZED, STARS, OFFSET] {
        assert_renderer_matches_render_frame(p, PreviewMode::Raw);
    }
}

#[test]
fn renderer_composited_matches_full_frames() {
    let opt = render_all_frames(OPTIMIZED, PreviewMode::Composited).unwrap();
    let full = render_all_frames(FULL, PreviewMode::Composited).unwrap();
    assert_eq!(opt.len(), full.len());
    for (a, b) in opt.iter().zip(&full) {
        assert!(
            a.image == b.image,
            "frame {} differs from the unoptimized source",
            a.index
        );
    }
}

#[test]
fn renderer_is_lazy_and_reports_dimensions() {
    let mut r = FrameRenderer::open(OPTIMIZED, PreviewMode::Composited).unwrap();
    assert_eq!(r.dimensions(), (100, 100));
    assert_eq!(r.next().unwrap().unwrap().index, 0);
    assert_eq!(r.by_ref().count(), 4);
    assert!(r.next().is_none(), "iterator stays exhausted");
}

#[test]
fn all_frame_images_are_pngs() {
    let pngs = get_all_frame_images(OPTIMIZED.into(), PreviewMode::Raw).unwrap();
    assert_eq!(pngs.len(), 5);
    assert!(pngs.iter().all(|p| p.starts_with(b"\x89PNG\r\n\x1a\n")));
}

#[test]
fn missing_file_errors() {
    assert!(
        FrameRenderer::open("tests/testdata/does-not-exist.gif", PreviewMode::Composited).is_err()
    );
}

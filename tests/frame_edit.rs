//! Deleting / re-keying / unoptimizing frames must keep every remaining frame looking exactly
//! like it did in the original GIF.

use gifmeta::{
    get_metadata, modify_gif_with_report, render_all_frames, DeleteMode, ModOptions, ModReport,
    PreviewMode,
};
use std::path::PathBuf;

const OPTIMIZED: &str = "tests/testdata/optimize/moving-square-optimized.gif";
const FULL: &str = "tests/testdata/optimize/moving-square-full.gif";
const STARS: &str = "tests/testdata/transparency/stars-transparent.gif";
const OFFSET: &str = "tests/testdata/layout/offset-frames.gif";

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("gifmeta-tests-edit");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn edit(input: &str, name: &str, opts: ModOptions) -> (PathBuf, ModReport) {
    let out = tmp(name);
    let report = modify_gif_with_report(
        &input.into(),
        ModOptions {
            output: Some(out.clone()),
            ..opts
        },
    )
    .expect("modify failed");
    (out, report)
}

/// Every output frame must equal the original frame it came from (composited).
fn assert_frames_match(original: &str, output: &PathBuf, kept: &[usize]) {
    let orig = render_all_frames(original, PreviewMode::Composited).unwrap();
    let out = render_all_frames(output, PreviewMode::Composited).unwrap();
    assert_eq!(out.len(), kept.len(), "frame count");
    for (o, &k) in out.iter().zip(kept) {
        let diff = o
            .image
            .pixels()
            .zip(orig[k].image.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            diff, 0,
            "output frame {} (original {}) differs in {} px",
            o.index, k, diff
        );
    }
}

fn kept(total: usize, deleted: &[usize]) -> Vec<usize> {
    (0..total).filter(|i| !deleted.contains(i)).collect()
}

#[test]
fn delete_middle_frame_rekeys_dependent_frame() {
    let (out, r) = edit(
        OPTIMIZED,
        "del-2.gif",
        ModOptions {
            delete_frames: Some(vec![2]),
            ..Default::default()
        },
    );
    assert_eq!((r.frames_in, r.frames_out), (5, 4));
    assert_eq!(r.rekeyed, vec![3]);
    assert!(r.quantized.is_empty() && r.transparency_warnings.is_empty());
    assert_frames_match(OPTIMIZED, &out, &kept(5, &[2]));
    // Same as deleting from the unoptimized source.
    assert_frames_match(FULL, &out, &kept(5, &[2]));
}

#[test]
fn deleted_delay_is_merged_into_previous_frame() {
    let (out, _) = edit(
        OPTIMIZED,
        "merge.gif",
        ModOptions {
            delete_frames: Some(vec![2]),
            ..Default::default()
        },
    );
    let m = get_metadata(&out, true).unwrap();
    let delays: Vec<u16> = m.frames.iter().map(|f| f.delay_cs).collect();
    assert_eq!(delays, vec![10, 20, 10, 10]);
    assert_eq!(m.total_duration_cs, 50, "total duration kept");
}

#[test]
fn drop_mode_shortens_animation() {
    let (out, _) = edit(
        OPTIMIZED,
        "drop.gif",
        ModOptions {
            delete_frames: Some(vec![2]),
            delete_mode: DeleteMode::DropDelay,
            ..Default::default()
        },
    );
    let m = get_metadata(&out, true).unwrap();
    assert_eq!(m.total_duration_cs, 40);
}

#[test]
fn delete_first_frame_merges_into_next_and_rekeys() {
    let (out, r) = edit(
        OPTIMIZED,
        "del-0.gif",
        ModOptions {
            delete_frames: Some(vec![0]),
            ..Default::default()
        },
    );
    assert_eq!(r.rekeyed, vec![1]);
    assert_frames_match(OPTIMIZED, &out, &kept(5, &[0]));
    let m = get_metadata(&out, true).unwrap();
    assert_eq!(m.frames[0].delay_cs, 20);
    assert!(m.frames[0].is_keyframe);
}

#[test]
fn delete_consecutive_and_last_frames() {
    for (name, del) in [
        ("del-12.gif", vec![1, 2]),
        ("del-34.gif", vec![3, 4]),
        ("del-024.gif", vec![0, 2, 4]),
    ] {
        let (out, _) = edit(
            OPTIMIZED,
            name,
            ModOptions {
                delete_frames: Some(del.clone()),
                ..Default::default()
            },
        );
        assert_frames_match(OPTIMIZED, &out, &kept(5, &del));
        assert_eq!(
            get_metadata(&out, false).unwrap().total_duration_cs,
            50,
            "{name}"
        );
    }
}

#[test]
fn self_clearing_frames_need_no_rekey() {
    let del = [0, 5, 6, 7, 40, 149];
    let (out, r) = edit(
        STARS,
        "stars-del.gif",
        ModOptions {
            delete_frames: Some(del.to_vec()),
            ..Default::default()
        },
    );
    assert_eq!(r.frames_out, 150 - del.len());
    assert!(r.rekeyed.is_empty(), "rekeyed: {:?}", r.rekeyed);
    assert_frames_match(STARS, &out, &kept(150, &del));
}

#[test]
fn offset_frames_with_transparency() {
    let (out, r) = edit(
        OFFSET,
        "offset-del0.gif",
        ModOptions {
            delete_frames: Some(vec![0]),
            ..Default::default()
        },
    );
    assert_eq!(r.frames_out, 1);
    assert_frames_match(OFFSET, &out, &[1]);
}

#[test]
fn keyframe_converts_frame_to_full_image() {
    let (out, r) = edit(
        OPTIMIZED,
        "key-3.gif",
        ModOptions {
            keyframes: Some(vec![3]),
            ..Default::default()
        },
    );
    assert_eq!(r.rekeyed, vec![3]);
    assert_frames_match(OPTIMIZED, &out, &[0, 1, 2, 3, 4]);
    let m = get_metadata(&out, true).unwrap();
    assert!(m.frames[3].is_keyframe);
    assert_eq!((m.frames[3].width, m.frames[3].height), (100, 100));
}

#[test]
fn unoptimize_makes_every_frame_a_keyframe() {
    let (out, r) = edit(
        OPTIMIZED,
        "unopt.gif",
        ModOptions {
            unoptimize: true,
            ..Default::default()
        },
    );
    assert_eq!(r.rekeyed, vec![0, 1, 2, 3, 4]);
    let m = get_metadata(&out, true).unwrap();
    assert!(!m.is_delta_optimized);
    assert_eq!(m.keyframe_count, 5);
    assert_frames_match(OPTIMIZED, &out, &[0, 1, 2, 3, 4]);
}

#[test]
fn edits_combine_with_loop_and_delay_overrides_by_original_index() {
    let mut delays = std::collections::HashMap::new();
    delays.insert(3, 30); // original index 3
    let (out, _) = edit(
        OPTIMIZED,
        "combo.gif",
        ModOptions {
            delete_frames: Some(vec![2]),
            delays: Some(delays),
            loop_mode: Some(gifmeta::LoopMode::Times(2)),
            ..Default::default()
        },
    );
    let m = get_metadata(&out, true).unwrap();
    assert_eq!(m.loop_count, 2);
    // original 3 is now output index 2
    assert_eq!(m.frames[2].delay_cs, 30);
}

#[test]
fn invalid_edits_are_rejected() {
    let bad = |opts: ModOptions| {
        modify_gif_with_report(
            &OPTIMIZED.into(),
            ModOptions {
                output: Some(tmp("bad.gif")),
                ..opts
            },
        )
        .is_err()
    };
    assert!(bad(ModOptions {
        delete_frames: Some(vec![0, 1, 2, 3, 4]),
        ..Default::default()
    }));
    assert!(bad(ModOptions {
        delete_frames: Some(vec![5]),
        ..Default::default()
    }));
    assert!(bad(ModOptions {
        keyframes: Some(vec![99]),
        ..Default::default()
    }));
    assert!(bad(ModOptions {
        delete_frames: Some(vec![1]),
        keyframes: Some(vec![1]),
        ..Default::default()
    }));
}

/// Builds a 2-frame GIF where frame 0 and frame 1 each use 200 different colours (local
/// palettes) and frame 1 only covers part of the canvas → its composite has ~400 colours.
fn many_colors_gif() -> PathBuf {
    let path = tmp("many-colors.gif");
    let mut file = std::fs::File::create(&path).unwrap();
    let mut enc = gif::Encoder::new(&mut file, 40, 20, &[]).unwrap();
    let mut pal_a = Vec::new();
    let mut pal_b = Vec::new();
    for i in 0..200u32 {
        pal_a.extend_from_slice(&[i as u8, (i / 2) as u8, 255 - i as u8]);
        pal_b.extend_from_slice(&[(255 - i) as u8, i as u8, (i / 3) as u8]);
    }
    let a: Vec<u8> = (0..40 * 20).map(|i| (i % 200) as u8).collect();
    let mut f0 = gif::Frame::from_palette_pixels(40, 20, &a, &pal_a, None);
    f0.delay = 5;
    enc.write_frame(&f0).unwrap();
    let b: Vec<u8> = (0..20 * 20).map(|i| (i % 200) as u8).collect();
    let mut f1 = gif::Frame::from_palette_pixels(20, 20, &b, &pal_b, None);
    f1.left = 20;
    f1.delay = 5;
    enc.write_frame(&f1).unwrap();
    drop(enc);
    path
}

#[test]
fn too_many_colors_are_quantized_with_warning() {
    let src = many_colors_gif();
    let (out, r) = edit(
        src.to_str().unwrap(),
        "many-colors-key.gif",
        ModOptions {
            keyframes: Some(vec![1]),
            ..Default::default()
        },
    );
    assert_eq!(r.quantized, vec![1]);
    assert_eq!(r.warnings().len(), 1);
    let m = get_metadata(&out, true).unwrap();
    assert_eq!(m.frame_count, 2);
    assert!(m.frames[1].is_keyframe);
    // Frame 0 is copied losslessly; frame 1 is colour-reduced → close, not exact.
    let orig = render_all_frames(&src, PreviewMode::Composited).unwrap();
    let new = render_all_frames(&out, PreviewMode::Composited).unwrap();
    assert!(orig[0].image == new[0].image, "frame 0 must be lossless");
    let (sum, n) =
        orig[1]
            .image
            .pixels()
            .zip(new[1].image.pixels())
            .fold((0u64, 0u64), |(s, n), (a, b)| {
                let d: u64 = (0..3)
                    .map(|c| (a[c] as i32 - b[c] as i32).unsigned_abs() as u64)
                    .sum();
                (s + d, n + 3)
            });
    let mean = sum as f64 / n as f64;
    assert!(
        mean < 12.0,
        "mean colour error per channel too high: {mean:.2}"
    );
    assert!(
        new[1].image.pixels().all(|p| p[3] == 255),
        "no opaque pixel may become transparent"
    );
}

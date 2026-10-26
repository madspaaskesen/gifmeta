//! Creating GIFs from images, inserting, duplicating and re-ordering frames. Every output frame
//! must look exactly like the frame (or image) it came from.

use gifmeta::{
    create_gif, get_loop_mode, get_metadata, modify_gif_with_report, render_all_frames,
    CreateOptions, DeleteMode, FrameInsert, LoopMode, ModOptions, ModReport, PreviewMode,
};
use image::{Rgba, RgbaImage};
use std::path::PathBuf;

const OPTIMIZED: &str = "tests/testdata/optimize/moving-square-optimized.gif"; // 100×100, 5 frames, 10 cs
const STARS: &str = "tests/testdata/transparency/stars-transparent.gif"; // self-clearing, transparent
const OFFSET: &str = "tests/testdata/layout/offset-frames.gif";

fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("gifmeta-tests-build");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// A test image: solid colour with a small marker so frames differ in more than colour.
fn image(name: &str, w: u32, h: u32, color: [u8; 4]) -> (PathBuf, RgbaImage) {
    let mut img = RgbaImage::from_pixel(w, h, Rgba(color));
    for x in 0..w.min(5) {
        img.put_pixel(x, 0, Rgba([color[2], color[0], color[1], 255]));
    }
    let path = tmp(name);
    img.save(&path).unwrap();
    (path, img)
}

/// Where an output frame should come from.
#[derive(Clone, Copy)]
enum Want<'a> {
    Orig(usize),
    Img(&'a RgbaImage),
}

fn assert_frames(original: &str, output: &PathBuf, want: &[Want]) {
    let orig = render_all_frames(original, PreviewMode::Composited).unwrap();
    let out = render_all_frames(output, PreviewMode::Composited).unwrap();
    assert_eq!(out.len(), want.len(), "frame count");
    for (o, w) in out.iter().zip(want) {
        let (expected, label) = match *w {
            Want::Orig(i) => (&orig[i].image, format!("original {i}")),
            Want::Img(img) => (img, "image".to_string()),
        };
        let diff = o
            .image
            .pixels()
            .zip(expected.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            diff, 0,
            "output frame {} ({label}) differs in {diff} px",
            o.index
        );
    }
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

fn delays(path: &PathBuf) -> Vec<u16> {
    get_metadata(path, true)
        .unwrap()
        .frames
        .iter()
        .map(|f| f.delay_cs)
        .collect()
}

// --- create ---------------------------------------------------------------------------------

#[test]
fn create_from_pngs() {
    let (a, ia) = image("c-a.png", 40, 30, [255, 0, 0, 255]);
    let (b, ib) = image("c-b.png", 40, 30, [0, 255, 0, 255]);
    let (c, ic) = image("c-c.png", 40, 30, [0, 0, 255, 255]);
    let out = tmp("create.gif");
    let r = create_gif(
        &[a, b, c],
        CreateOptions {
            output: Some(out.clone()),
            delay: Some(20),
            delays: Some([(2, 50)].into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!((r.frames, r.width, r.height), (3, 40, 30));
    assert!(r.quantized.is_empty());
    assert_frames(
        OPTIMIZED, // unused for Img
        &out,
        &[Want::Img(&ia), Want::Img(&ib), Want::Img(&ic)],
    );
    assert_eq!(delays(&out), vec![20, 20, 50]);
    assert_eq!(get_loop_mode(&out).unwrap(), LoopMode::Infinite);
}

#[test]
fn create_defaults_and_loop_times() {
    let (a, _) = image("d-a.png", 8, 8, [10, 20, 30, 255]);
    let (b, _) = image("d-b.png", 8, 8, [30, 20, 10, 255]);
    let out = tmp("create-loop.gif");
    create_gif(
        &[a, b],
        CreateOptions {
            output: Some(out.clone()),
            loop_mode: Some(LoopMode::Times(2)),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(delays(&out), vec![10, 10]);
    assert_eq!(get_loop_mode(&out).unwrap(), LoopMode::Times(2));
}

#[test]
fn create_with_transparency_and_jpeg() {
    // Transparent PNG followed by an opaque one: the transparent frame must not leave
    // anything behind, and the opaque JPEG must load.
    let mut t = RgbaImage::from_pixel(16, 16, Rgba([0, 0, 0, 0]));
    t.put_pixel(3, 3, Rgba([255, 0, 0, 255]));
    let tp = tmp("t.png");
    t.save(&tp).unwrap();
    let (a, _) = image("t-a.png", 16, 16, [0, 0, 200, 255]);
    let jpg = tmp("t-b.jpg");
    image::DynamicImage::ImageRgba8(RgbaImage::from_pixel(16, 16, Rgba([128, 128, 128, 255])))
        .to_rgb8()
        .save(&jpg)
        .unwrap();
    let out = tmp("create-transparent.gif");
    create_gif(
        &[a, tp, jpg],
        CreateOptions {
            output: Some(out.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let frames = render_all_frames(&out, PreviewMode::Composited).unwrap();
    assert_eq!(frames[1].image, t, "transparent frame shows only itself");
    assert_eq!(frames[2].image.dimensions(), (16, 16));
}

#[test]
fn create_rejects_bad_input() {
    let (a, _) = image("e-a.png", 10, 10, [1, 2, 3, 255]);
    let (b, _) = image("e-b.png", 12, 10, [1, 2, 3, 255]);
    let err = create_gif(&[a.clone(), b], CreateOptions::default()).unwrap_err();
    assert!(err.contains("e-b.png (12×10)"), "{err}");
    let err = create_gif(std::slice::from_ref(&a), CreateOptions::default()).unwrap_err();
    assert!(err.contains("at least 2"), "{err}");
    let err = create_gif(&[a.clone(), tmp("missing.png")], CreateOptions::default()).unwrap_err();
    assert!(err.contains("missing.png"), "{err}");
    let err = create_gif(
        &[a.clone(), a],
        CreateOptions {
            delays: Some([(5, 10)].into()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("frame 5"), "{err}");
}

// --- insert ---------------------------------------------------------------------------------

#[test]
fn insert_into_optimized_gif() {
    let (p, img) = image("ins.png", 100, 100, [255, 255, 0, 255]);
    let (out, r) = edit(
        OPTIMIZED,
        "insert-2.gif",
        ModOptions {
            inserts: Some(vec![FrameInsert::new(2, &p)]),
            ..Default::default()
        },
    );
    assert_eq!((r.frames_in, r.frames_out, r.inserted), (5, 6, 1));
    // Frame 2 was a delta on top of frame 1 — now it follows the new image → re-keyed.
    assert_eq!(r.rekeyed, vec![2]);
    use Want::*;
    assert_frames(
        OPTIMIZED,
        &out,
        &[Orig(0), Orig(1), Img(&img), Orig(2), Orig(3), Orig(4)],
    );
    // Default delay = delay of the frame it's inserted before.
    assert_eq!(delays(&out), vec![10, 10, 10, 10, 10, 10]);
}

#[test]
fn insert_at_start_and_end_with_delay() {
    let (p, img) = image("ins-se.png", 100, 100, [0, 128, 255, 255]);
    let (out, r) = edit(
        OPTIMIZED,
        "insert-ends.gif",
        ModOptions {
            inserts: Some(vec![
                FrameInsert::new(0, &p).with_delay(30),
                FrameInsert::new(5, &p).with_delay(70),
            ]),
            ..Default::default()
        },
    );
    assert_eq!(r.frames_out, 7);
    use Want::*;
    assert_frames(
        OPTIMIZED,
        &out,
        &[
            Img(&img),
            Orig(0),
            Orig(1),
            Orig(2),
            Orig(3),
            Orig(4),
            Img(&img),
        ],
    );
    assert_eq!(delays(&out), vec![30, 10, 10, 10, 10, 10, 70]);
}

#[test]
fn insert_and_delete_replaces_frame_without_merging_delay() {
    let (p, img) = image("replace.png", 100, 100, [200, 0, 200, 255]);
    let (out, _) = edit(
        OPTIMIZED,
        "replace-2.gif",
        ModOptions {
            inserts: Some(vec![FrameInsert::new(2, &p)]),
            delete_frames: Some(vec![2]),
            ..Default::default()
        },
    );
    use Want::*;
    assert_frames(
        OPTIMIZED,
        &out,
        &[Orig(0), Orig(1), Img(&img), Orig(3), Orig(4)],
    );
    assert_eq!(delays(&out), vec![10, 10, 10, 10, 10], "duration unchanged");
}

#[test]
fn insert_transparent_image_into_transparent_gif() {
    let (w, h) = {
        let m = get_metadata(&STARS.into(), false).unwrap();
        (m.width as u32, m.height as u32)
    };
    let mut img = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 0]));
    for x in 0..w / 2 {
        img.put_pixel(x, h / 2, Rgba([0, 255, 0, 255]));
    }
    let p = tmp("stars-ins.png");
    img.save(&p).unwrap();
    let (out, r) = edit(
        STARS,
        "stars-insert.gif",
        ModOptions {
            inserts: Some(vec![FrameInsert::new(10, &p)]),
            ..Default::default()
        },
    );
    assert!(
        r.insert_transparency_warnings.is_empty(),
        "{:?}",
        r.warnings()
    );
    let mut want: Vec<Want> = (0..150).map(Want::Orig).collect();
    want.insert(10, Want::Img(&img));
    assert_frames(STARS, &out, &want);
}

#[test]
fn insert_rejects_wrong_size_and_position() {
    let (p, _) = image("small.png", 50, 50, [1, 1, 1, 255]);
    let err = modify_gif_with_report(
        &OPTIMIZED.into(),
        ModOptions {
            output: Some(tmp("never.gif")),
            inserts: Some(vec![FrameInsert::new(1, &p)]),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("50×50") && err.contains("100×100"), "{err}");

    let (p, _) = image("ok.png", 100, 100, [1, 1, 1, 255]);
    let err = modify_gif_with_report(
        &OPTIMIZED.into(),
        ModOptions {
            output: Some(tmp("never.gif")),
            inserts: Some(vec![FrameInsert::new(6, &p)]),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("use 5 to append"), "{err}");
}

// --- duplicate ------------------------------------------------------------------------------

#[test]
fn duplicate_frames_in_optimized_gif() {
    let (out, r) = edit(
        OPTIMIZED,
        "dup.gif",
        ModOptions {
            duplicates: Some(vec![1, 3, 3]),
            ..Default::default()
        },
    );
    assert_eq!(r.frames_out, 8);
    assert_eq!(r.duplicated, vec![1, 3, 3]);
    use Want::*;
    assert_frames(
        OPTIMIZED,
        &out,
        &[
            Orig(0),
            Orig(1),
            Orig(1),
            Orig(2),
            Orig(3),
            Orig(3),
            Orig(3),
            Orig(4),
        ],
    );
}

#[test]
fn duplicate_self_clearing_and_offset_frames() {
    for (input, n, dups) in [(STARS, 150, vec![0, 75, 149]), (OFFSET, 0, vec![0, 1])] {
        let n = if n == 0 {
            get_metadata(&input.into(), false).unwrap().frame_count as usize
        } else {
            n
        };
        let (out, _) = edit(
            input,
            "dup-mixed.gif",
            ModOptions {
                duplicates: Some(dups.clone()),
                ..Default::default()
            },
        );
        let mut want = Vec::new();
        for i in 0..n {
            want.push(Want::Orig(i));
            if dups.contains(&i) {
                want.push(Want::Orig(i));
            }
        }
        assert_frames(input, &out, &want);
    }
}

#[test]
fn duplicate_and_delete_same_frame_is_an_error() {
    let err = modify_gif_with_report(
        &OPTIMIZED.into(),
        ModOptions {
            duplicates: Some(vec![2]),
            delete_frames: Some(vec![2]),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("both deleted and duplicated"), "{err}");
}

// --- order ----------------------------------------------------------------------------------

#[test]
fn reverse_optimized_gif() {
    let order = vec![4, 3, 2, 1, 0];
    let (out, r) = edit(
        OPTIMIZED,
        "reverse.gif",
        ModOptions {
            order: Some(order.clone()),
            delays: Some([(4, 40)].into()),
            ..Default::default()
        },
    );
    assert!(r.reordered);
    let want: Vec<Want> = order.iter().map(|&i| Want::Orig(i)).collect();
    assert_frames(OPTIMIZED, &out, &want);
    // Delays follow the frame (original index 4 → now first).
    assert_eq!(delays(&out), vec![40, 10, 10, 10, 10]);
}

#[test]
fn shuffle_transparent_gif() {
    let mut order: Vec<usize> = (0..150).collect();
    order.swap(0, 149);
    order[10..60].reverse();
    let (out, _) = edit(
        STARS,
        "stars-order.gif",
        ModOptions {
            order: Some(order.clone()),
            ..Default::default()
        },
    );
    let want: Vec<Want> = order.iter().map(|&i| Want::Orig(i)).collect();
    assert_frames(STARS, &out, &want);
}

#[test]
fn identity_order_copies_everything() {
    let (_, r) = edit(
        OPTIMIZED,
        "same-order.gif",
        ModOptions {
            order: Some(vec![0, 1, 2, 3, 4]),
            ..Default::default()
        },
    );
    assert!(!r.reordered);
    assert!(r.rekeyed.is_empty());
}

#[test]
fn order_must_be_a_permutation() {
    for (order, needle) in [
        (vec![0, 1, 2, 3], "exactly once"),
        (vec![0, 1, 2, 3, 3], "exactly once"),
        (vec![0, 1, 2, 3, 9], "out of range"),
    ] {
        let err = modify_gif_with_report(
            &OPTIMIZED.into(),
            ModOptions {
                order: Some(order),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert!(err.contains(needle), "{err}");
    }
    let err = modify_gif_with_report(
        &OPTIMIZED.into(),
        ModOptions {
            order: Some(vec![4, 3, 2, 1, 0]),
            delete_frames: Some(vec![1]),
            delete_mode: DeleteMode::DropDelay,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(err.contains("two steps"), "{err}");
}

#[test]
fn transparent_image_in_opaque_gif_clears_previous_frame() {
    // GIF can only draw on top. For the transparent image to show exactly, the frame before it
    // must be cleared — gifmeta switches that frame to disposal Background.
    let mut img = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 0]));
    for y in 40..60 {
        for x in 40..60 {
            img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }
    let p = tmp("hole.png");
    img.save(&p).unwrap();
    let (out, r) = edit(
        OPTIMIZED,
        "insert-transparent.gif",
        ModOptions {
            inserts: Some(vec![FrameInsert::new(3, &p)]),
            ..Default::default()
        },
    );
    assert!(r.warnings().is_empty(), "{:?}", r.warnings());
    use Want::*;
    assert_frames(
        OPTIMIZED,
        &out,
        &[Orig(0), Orig(1), Orig(2), Img(&img), Orig(3), Orig(4)],
    );
}

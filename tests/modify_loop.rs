use gifmeta::utils::loop_count::extract_raw_loop_value;
use gifmeta::utils::parse_csv::parse_keyval_csv;
use gifmeta::{get_loop_mode, get_metadata, modify_gif, set_loop_mode, LoopMode, ModOptions};
use std::path::PathBuf;

// Note: ImageMagick writes a loop block (value 0 = infinite) by default, so
// `no-loop-flag.gif` actually loops forever. `loop-once.gif` (`-loop 1`) has NO block.
const NO_BLOCK: &str = "tests/testdata/loop/loop-once.gif"; // no block → plays once
const WITH_BLOCK: &str = "tests/testdata/loop/2frame-soft.gif"; // block, value 0
const INFINITE: &str = "tests/testdata/loop/1.gif";
const DEFAULT_IM: &str = "tests/testdata/loop/no-loop-flag.gif"; // ImageMagick default

/// Output path in the system temp dir, so tests never touch tests/testdata.
fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("gifmeta-tests");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn modify(input: &str, out: &PathBuf, opts: ModOptions) {
    modify_gif(
        &input.into(),
        ModOptions {
            output: Some(out.clone()),
            ..opts
        },
    )
    .expect("modify failed");
}

#[test]
fn missing_loop_block_means_plays_once() {
    assert_eq!(get_loop_mode(&NO_BLOCK.into()).unwrap(), LoopMode::Times(1));
    let m = get_metadata(&NO_BLOCK.into(), false).unwrap();
    assert_eq!(m.loop_count, 1);
    assert!(!m.loop_infinite);
}

#[test]
fn imagemagick_default_loops_forever() {
    assert_eq!(extract_raw_loop_value(DEFAULT_IM).unwrap(), Some(0));
    assert_eq!(
        get_loop_mode(&DEFAULT_IM.into()).unwrap(),
        LoopMode::Infinite
    );
}

#[test]
fn infinite_is_reported_as_zero_and_flag() {
    let m = get_metadata(&INFINITE.into(), false).unwrap();
    assert_eq!(m.loop_count, 0);
    assert!(m.loop_infinite);
}

#[test]
fn changing_delay_keeps_plays_once() {
    // Regression: this used to turn a "plays once" GIF into an infinite loop.
    let out = tmp("no-loop-flag-delay.gif");
    modify(
        NO_BLOCK,
        &out,
        ModOptions {
            delay_all: Some(50),
            ..Default::default()
        },
    );
    assert_eq!(
        extract_raw_loop_value(&out).unwrap(),
        None,
        "no block expected"
    );
    assert_eq!(get_loop_mode(&out).unwrap(), LoopMode::Times(1));
}

#[test]
fn changing_delay_keeps_infinite() {
    let out = tmp("infinite-delay.gif");
    modify(
        INFINITE,
        &out,
        ModOptions {
            delay_all: Some(5),
            ..Default::default()
        },
    );
    assert_eq!(get_loop_mode(&out).unwrap(), LoopMode::Infinite);
}

#[test]
fn modify_loop_roundtrip() {
    for (mode, raw) in [
        (LoopMode::Infinite, Some(0)),
        (LoopMode::Times(1), None),
        (LoopMode::Times(2), Some(1)),
        (LoopMode::Times(3), Some(2)),
    ] {
        let out = tmp(&format!("mod-roundtrip-{:?}.gif", mode));
        modify(
            WITH_BLOCK,
            &out,
            ModOptions {
                loop_mode: Some(mode),
                ..Default::default()
            },
        );
        assert_eq!(get_loop_mode(&out).unwrap(), mode, "read back {:?}", mode);
        assert_eq!(
            extract_raw_loop_value(&out).unwrap(),
            raw,
            "raw for {:?}",
            mode
        );
    }
}

#[test]
fn set_loop_mode_patches_inserts_and_removes_block() {
    // Insert into a file without block.
    let a = tmp("set-insert.gif");
    set_loop_mode(&NO_BLOCK.into(), LoopMode::Times(4), Some(a.clone())).unwrap();
    assert_eq!(get_loop_mode(&a).unwrap(), LoopMode::Times(4));
    // Patch an existing block.
    let b = tmp("set-patch.gif");
    set_loop_mode(&a, LoopMode::Infinite, Some(b.clone())).unwrap();
    assert_eq!(get_loop_mode(&b).unwrap(), LoopMode::Infinite);
    // Remove it again → plays once, and the GIF still decodes.
    let c = tmp("set-remove.gif");
    set_loop_mode(&b, LoopMode::Times(1), Some(c.clone())).unwrap();
    assert_eq!(extract_raw_loop_value(&c).unwrap(), None);
    assert_eq!(get_metadata(&c, false).unwrap().frame_count, 2);
}

#[test]
fn times_zero_is_rejected() {
    let out = tmp("times-zero.gif");
    let res = modify_gif(
        &WITH_BLOCK.into(),
        ModOptions {
            output: Some(out),
            loop_mode: Some(LoopMode::Times(0)),
            ..Default::default()
        },
    );
    assert!(res.is_err());
}

#[test]
fn empty_options_are_rejected() {
    assert!(modify_gif(&WITH_BLOCK.into(), ModOptions::default()).is_err());
}

#[test]
fn loop_and_frame_delays_together() {
    let out = tmp("loop-and-delays.gif");
    modify(
        WITH_BLOCK,
        &out,
        ModOptions {
            loop_mode: Some(LoopMode::Times(3)),
            delay_all: Some(4),
            delays: Some(parse_keyval_csv("1=15").unwrap()),
            ..Default::default()
        },
    );
    let m = get_metadata(&out, true).unwrap();
    assert_eq!(m.loop_count, 3);
    assert_eq!(m.frames[0].delay_cs, 4);
    assert_eq!(m.frames[1].delay_cs, 15);
}

#[test]
#[allow(deprecated)]
fn deprecated_mod_gif_still_works() {
    // Old API: loop_count 0 = infinite, n = plays n times.
    let out = tmp("deprecated-mod-gif.gif");
    gifmeta::mod_gif(&WITH_BLOCK.into(), Some(out.clone()), Some(0), None, None).unwrap();
    assert_eq!(get_loop_mode(&out).unwrap(), LoopMode::Infinite);
    gifmeta::mod_gif(&WITH_BLOCK.into(), Some(out.clone()), Some(2), None, None).unwrap();
    assert_eq!(get_metadata(&out, false).unwrap().loop_count, 2);
}

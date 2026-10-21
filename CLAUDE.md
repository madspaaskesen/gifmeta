# gifmeta (crate + CLI)

Small Rust crate and CLI for inspecting and editing GIF metadata: loop count, frame delays,
frame preview as PNG. Published on crates.io (current: **0.4.1**), MIT, edition 2021.
Consumed by the sibling `../gifmeta-gui` Tauri app.

## Commands
```bash
cargo build
cargo test            # integration tests + doctests (all green as of 0.4.1)
cargo fmt -- --check  # CI runs this — run `cargo fmt` before committing
cargo run -- info tests/testdata/loop/10frame-rainbow.gif --show-frames [--json]
cargo run -- mod  tests/testdata/loop/loop-once.gif --loop-count 0 --delay 10 --delays 1=20 -o out.gif
cargo run -- preview tests/testdata/loop/2.gif --frame 0 -o frame0.png   # or --as-base64
./run-info-all.sh        # info on every test GIF
./run-visual-preview.sh  # HTML before/after report in tests/visual/
./run-create-tests.sh    # regenerates tests/testdata with ImageMagick `convert`
```
Note: INPUT is a **positional** arg. The README examples using `--input` are outdated.

## Layout
```
src/main.rs                 clap v4 CLI: subcommands Info, Mod, Preview
src/lib.rs                  public API (what the GUI calls) — thin wrappers, doc-tested
src/gifmeta_structs.rs      GifMetadata, FrameMeta (Serialize → JSON for CLI --json and GUI)
src/commands/info.rs        get_metadata: decode, count frames, sum delays, palette/transparency
src/commands/modify.rs      apply_modifications: decode → re-encode with new loop/delays
src/utils/loop_count.rs     raw byte scan/patch of the NETSCAPE2.0 block
src/utils/set_frame_delay.rs  older standalone "delay all" re-encoder
src/utils/extract_frame_as_png.rs  composite frames onto RGBA canvas → PNG bytes
src/utils/frame_analysis.rs   KeyframeTracker: 🔑 keyframe vs Δ delta detection (dirty-area rule)
src/utils/parse_csv.rs      "1=10,5=20" → HashMap<usize,u16>
tests/*.rs                  integration tests; fixtures in tests/testdata/{loop,color,timing,transparency,layout,optimize}
docs/FRAME_EDITING_PLAN.md  roadmap for frame editing (detect → GUI → renderer → re-key → delete → unoptimize)
```
`main.rs` declares `pub mod utils;` itself AND uses the lib — utils is compiled twice (bin + lib). Harmless, but don't add state there.

## Public API (src/lib.rs)
- `get_metadata(&PathBuf, show_frames) -> Result<GifMetadata, String>` — `frames` is only filled when `show_frames` is true.
- `mod_gif(input, output: Option<PathBuf>, loop_count, delay_all, delays: Option<HashMap<usize,u16>>)` — errors if nothing to change.
- `get_frame_image(path: String, frame) -> Result<Vec<u8>, String>` — PNG bytes, full-canvas composite.
- `get_loop_count`, `set_loop_count`, `set_frame_delay` — older helpers that also `println!`.
- `show_frame_delays`, `set_selected_frame_delays` — **stubs**, only print.

Errors are `String` everywhere (`GifError` struct exists but is unused). Lib functions print to
stdout/stderr — keep that in mind, the GUI calls them too.

## Gotchas — read before touching modify/loop code
- **Loop count semantics are inconsistent.** File stores N = extra repeats (0 = infinite).
  `info` reports user-facing `N+1` (0 stays 0). `mod --loop-count n` writes `Repeat::Finite(n-1)`
  (user-facing, matches info). But `set_loop_count` / `utils::loop_count::set_loop_count` writes the **raw** value.
- If a GIF has no NETSCAPE block, `extract_loop_count` errs → treated as 0 → `mod` will write
  **infinite** loop even though the original plays once. (`tests/testdata/loop/no-loop-flag.gif` reproduces it.)
- `mod` without `--output` writes to `$TMPDIR/<uuid>.mod.gif` — it does NOT overwrite the input.
  `set_frame_delay` with `None` DOES overwrite the input (tempfile + rename).
- Re-encoding (`modify.rs`) decodes with `ColorOutput::Indexed` and clones each `gif::Frame`, so local
  palettes, disposal, offsets and transparency are preserved. Keep it that way — never decode to RGBA for writing.
- `extract_frame_as_png` composites frames naively (no disposal-method handling, transparent pixels overwrite).
- Doctests have side effects: `set_loop_count` doctest rewrites `tests/testdata/loop/2.gif` (loop=3);
  `get_frame_image` doctest writes `tests/output/frame0.png`. Integration tests write `modified-*.gif` into testdata.
  Check `git status` after `cargo test` and don't commit those by accident.
- `loop_count::extract_loop_count` indexes `buffer[i+1]` without a bounds check — can panic on a truncated file ending in 0x21.

## Frame editing roadmap
**Read `docs/FRAME_EDITING_PLAN.md` before working on keyframes, deleting, re-keying or unoptimizing frames.**
It holds the phases, status, pitfalls (palettes >256 colours, disposal `Previous`, index semantics),
known bugs and open decisions. Update its status markers when a phase lands.
- Phase 1 (detection) is done but unreleased: `FrameMeta` has rect/disposal/`is_keyframe`,
  `GifMetadata` has `keyframe_count`/`is_delta_optimized`.
- Delete/re-key must use the disposal-aware renderer (Phase 3) + shadow simulation — never just skip frames.

## Release checklist
1. Bump `version` in Cargo.toml (Cargo.lock updates on build).
2. Add a section at the top of CHANGELOG.md (`## [x.y.z] ✨ YYYY-MM-DD - title`).
3. Update README usage.
4. `cargo fmt && cargo test`, commit `🔖 Bump to vX.Y.Z – …`.
5. `cargo publish`; GitHub release → `.github/workflows/release.yml` builds linux/macos/windows binaries.

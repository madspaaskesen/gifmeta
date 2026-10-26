# gifmeta (crate + CLI)

Small Rust crate and CLI for inspecting and editing GIF metadata: loop count, frame delays,
frame preview as PNG. Published on crates.io (current: **0.7.0**), MIT, edition 2021.
Consumed by the sibling `../gifmeta-gui` Tauri app.

## Commands
```bash
cargo build
cargo test            # unit + integration tests + doctests (~20 s; deps are optimized in dev via [profile.dev.package."*"])
cargo run --release --example bench_render -- some.gif   # single-pass vs per-frame rendering
cargo fmt -- --check  # CI runs this — run `cargo fmt` before committing
cargo run -- info tests/testdata/loop/10frame-rainbow.gif --show-frames [--json]
cargo run -- mod  tests/testdata/loop/loop-once.gif --infinite --delay 10 --delays 1=20 -o out.gif   # or --loop-count 3
cargo run -- preview tests/testdata/loop/2.gif --frame 0 -o frame0.png   # or --as-base64
./run-info-all.sh        # info on every test GIF
./run-visual-preview.sh  # HTML before/after report in tests/visual/
./run-create-tests.sh    # regenerates tests/testdata with ImageMagick `convert`
```
Note: INPUT is a **positional** arg (no `--input`).

## Layout
```
src/main.rs                 clap v4 CLI: subcommands Info, Mod, Create, Preview
src/lib.rs                  public API (what the GUI calls) — thin wrappers, doc-tested
src/gifmeta_structs.rs      GifMetadata, FrameMeta (Serialize → JSON for CLI --json and GUI)
src/commands/info.rs        get_metadata: decode, count frames, sum delays, palette/transparency
src/commands/modify.rs      ModOptions + apply_with_report(): loop/delays, delete/insert/duplicate/order/keyframe (FrameWriter = shadow simulation)
src/commands/create.rs      create(): images → new GIF (CreateOptions, CreateReport)
src/utils/load_image.rs     load PNG/JPEG/WebP as RGBA, size check
src/utils/loop_count.rs     LoopMode read/write: scan/patch/insert/remove the NETSCAPE2.0 block
src/utils/set_frame_delay.rs  older standalone "delay all" re-encoder
src/utils/render.rs           disposal-aware renderer: FrameRenderer (single-pass iterator), render_all_frames,
                              render_frame(path, i, PreviewMode::{Composited,Raw}) (uses FrameRenderer::nth)
src/utils/extract_frame_as_png.rs  render → PNG bytes (pretty by default, raw via _with_mode)
src/utils/frame_analysis.rs   KeyframeTracker: 🔑 keyframe vs Δ delta detection (dirty-area rule)
src/utils/parse_csv.rs      "1=10,5=20" → HashMap<usize,u16>
tests/*.rs                  integration tests; fixtures in tests/testdata/{loop,color,timing,transparency,layout,optimize}
docs/FRAME_EDITING_PLAN.md  roadmap for frame editing (detect → GUI → renderer → re-key → delete → unoptimize)
docs/WASM_PLAN.md           future plan: gifmeta in the browser (byte API → feature flags → gifmeta-wasm → web app)
```
`main.rs` only uses the lib (`gifmeta::…`) — never add `mod` declarations to main.rs (utils would compile twice without the rest of the crate).

## Public API (src/lib.rs)
- `get_metadata(&PathBuf, show_frames) -> Result<GifMetadata, String>` — `frames` is only filled when `show_frames` is true;
  `keyframe_count`, `is_delta_optimized`, `loop_count`, `loop_infinite` always. Structs are `#[non_exhaustive]`.
- `modify_gif(input, ModOptions { output, loop_mode, delay_all, delays, delete_frames, delete_mode, keyframes, unoptimize, inserts, duplicates, order, ..Default::default() }) -> Result<PathBuf, String>`;
  `modify_gif_with_report` → `ModReport { frames_in, frames_out, deleted, rekeyed, quantized, transparency_warnings, inserted, duplicated, reordered, quantized_inserts, insert_transparency_warnings }`.
  `FrameInsert::new(before, path).with_delay(cs)`; `order` = permutation, exclusive with delete/insert/duplicate.
  — new options go into `ModOptions` (never new positional params). Errors if nothing to change.
- `create_gif(&[PathBuf], CreateOptions { output, delay, delays, loop_mode, ..Default::default() }) -> Result<CreateReport, String>` — ≥ 2 same-size images. CLI `create IMAGES... -o out.gif`.
- `LoopMode::{Infinite, Times(n)}` — `Times(1)` = plays once = **no** NETSCAPE block. `get_loop_mode`, `set_loop_mode`.
- `get_frame_image(path: String, frame) -> Result<Vec<u8>, String>` — PNG bytes, full-canvas composite ("pretty").
- `FrameRenderer::open(path, mode)` — iterator of `RenderedFrame { index, delay_cs, image }`, one pass over the GIF.
  `render_all_frames` / `get_all_frame_images` (PNG bytes) collect it. Use these for playback/export — calling
  `render_frame` per frame is O(n²). CLI: `preview --all --output-dir DIR [--raw]`.
- `get_frame_image_with_mode(path, frame, PreviewMode)` — `Composited` (pretty) or `Raw` (only the frame's own pixels). CLI: `preview --raw`.
- Deprecated, kept for compatibility: `mod_gif` (0 = infinite), `utils::loop_count::extract_loop_count`.
- `get_loop_count`, `set_loop_count` (user-facing: 0 = ∞, n = n times), `set_frame_delay` — older helpers that also `println!`.
- `show_frame_delays`, `set_selected_frame_delays` — **stubs**, only print.

Errors are `String` everywhere (`GifError` struct exists but is unused). Some lib functions print to
stdout/stderr — keep that in mind, the GUI calls them too.

## Gotchas — read before touching modify/loop code
- **Loop semantics (one convention everywhere):** file stores N = extra repeats (0 = infinite, no block = once).
  Everything user-facing uses "plays n times": `LoopMode::Times(n)`, `loop_count` (0 = ∞ kept for JSON/compat).
  `Repeat::Finite(0)` makes the gif crate write **no** block — that's how `Times(1)` is encoded.
- **Fixtures:** ImageMagick writes a loop block (0 = infinite) by default, so `no-loop-flag.gif` loops forever
  despite its name. `loop-once.gif` is the fixture **without** a block.
- `mod` without `--output` writes to `$TMPDIR/<uuid>.mod.gif` — it does NOT overwrite the input.
  `set_frame_delay` / `set_loop_mode` with `None` DO overwrite the input.
- Re-encoding (`modify.rs`) decodes with `ColorOutput::Indexed` and clones each `gif::Frame`, so local
  palettes, disposal, offsets and transparency are preserved. Keep it that way — never decode to RGBA for writing.
- Previews go through `utils/render.rs` (handles Keep/Background/Previous, skips transparent pixels). Verified pixel-identical to ImageMagick `-coalesce`.
- Tests write outputs to `$TMPDIR/gifmeta-tests/` — never into `tests/testdata/`. Only `tests/output/` (gitignored) is used by the preview test.

## Frame editing roadmap
**Read `docs/FRAME_EDITING_PLAN.md` before working on keyframes, deleting, re-keying or unoptimizing frames.**
It holds the phases, status, pitfalls (palettes >256 colours, disposal `Previous`, index semantics),
known bugs and open decisions. Update its status markers when a phase lands.
- Phases 1, 3, 4, 5, 6 are done (0.5.0–0.6.0): detection, renderer, `--keyframe`, `--delete`, `--unoptimize`.
- Phases 7, 8 done (0.7.0): `--insert`, `--duplicate`, `--order`, `create`.
- Delete/re-key = shadow simulation in `commands/modify.rs::apply_with_report` (two `Compositor`s,
  copy when canvases match, otherwise `utils/encode_frame::encode_full_frame`). Never just skip frames.
- All frame indices in `ModOptions` are ORIGINAL indices. `ModOptions` grows — always build it with
  `..Default::default()` (the GUI too).

## Release checklist
1. Bump `version` in Cargo.toml (Cargo.lock updates on build).
2. Add a section at the top of CHANGELOG.md (`## [x.y.z] ✨ YYYY-MM-DD - title`).
3. Update README usage.
4. `cargo fmt && cargo test`, commit `🔖 Bump to vX.Y.Z – …`.
5. `cargo publish`; GitHub release → `.github/workflows/release.yml` builds linux/macos/windows binaries.

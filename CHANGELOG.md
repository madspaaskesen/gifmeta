# 📝 Changelog

All notable changes to **gifmeta** will be documented here.

---

## [0.6.2] 🌐 2026-10-26 - New homepage

- 🌐 Homepage is now [gifmeta.com](https://gifmeta.com). No code changes.

## [0.6.1] 🔎 2026-10-22 - Better crates.io description

- 🔎 New crate description, keywords (`animation`, `frames`) and categories (`multimedia::encoding`) so gifmeta is easier to find. No code changes.

## [0.6.0] ✂️ 2026-10-22 - Delete frames, keyframes & unoptimize

- ✂️ **Delete frames**: `gifmeta mod in.gif --delete 2,5 -o out.gif`
  - Frames that depended on a deleted frame (delta-optimized GIFs) are **automatically re-written as
    full images**, so every remaining frame looks exactly like before. Works with all disposal methods
    via *shadow simulation* (original and output are replayed side by side; a frame is copied losslessly
    when its canvas matches, otherwise re-keyed)
  - The deleted frame's delay is **merged into the previous frame** (total duration unchanged);
    `--delete-mode drop` makes the animation shorter instead
  - Verified on a 40-frame optimized GIF: 5 frames deleted → 35/35 frames pixel-identical, only 2 re-keyed
- 🔑 **Convert frames to full images**: `--keyframe 3,7`
- 🧱 **Unoptimize**: `--unoptimize` writes every frame as a full image (bigger file, every frame independent)
- 🎨 Re-keyed frames use an **exact local palette**; with > 256 colours they are colour-reduced (NeuQuant)
  with a reserved transparent index and a warning
- New lib API: `ModOptions { delete_frames, delete_mode, keyframes, unoptimize, .. }`, `DeleteMode`,
  `modify_gif_with_report()` → `ModReport { frames_in, frames_out, deleted, rekeyed, quantized, transparency_warnings }`
- All frame indices (delays, deletes, keyframes) refer to the **original** frame numbers
- ⚠️ `ModOptions` has new fields — construct it with `..Default::default()` (struct literals without it no longer compile)

---

## [0.5.1] ⚡ 2026-10-22 - Render all frames in one pass

- New `FrameRenderer`: an iterator that renders every frame in **one pass** (frame *n* builds on
  frame *n-1*) instead of replaying the animation from the start for each frame
  - forge-preview.gif (40 frames, 800×516): **2.29 s → 0.15 s**
  - stars-transparent.gif (150 frames): **31.4 s → 0.56 s** (release build)
- New lib API: `FrameRenderer::open(path, mode)`, `render_all_frames(path, mode)`,
  `get_all_frame_images(path, mode)` (PNG bytes), `RenderedFrame { index, delay_cs, image }`
- `render_frame` now uses the same renderer (one implementation) and skips frames without copies
- Faster compositing (row-based copies instead of per-pixel writes)
- CLI: `gifmeta preview in.gif --all --output-dir frames/` exports every frame as `frame_000.png`, … (works with `--raw`)
- Dev: dependencies are optimized in debug/test builds → `cargo test` is much faster
- `examples/bench_render.rs` to compare single-pass vs. per-frame rendering

---

## [0.5.0] 🧩 2026-10-22 - Delta-frame detection, correct previews & clear loop settings

### 🔄 Loop settings
- **Fixed:** a GIF without a loop block plays **once**, but was reported as `0` (infinite) — and
  `mod` without `--loop-count` (e.g. only `--delay`) silently turned it into an **infinite** loop
- New `--infinite` flag; `--loop-count N` now means "plays N times" (1 = once). Can't be combined
- `info` shows `🔄 Loop : ∞ infinite`, `1× (plays once)` or `3×`; JSON gets `loop_infinite`
- New lib API: `LoopMode::{Infinite, Times(n)}`, `modify_gif(input, ModOptions { .. })`,
  `get_loop_mode`, `set_loop_mode`
- `--delays` with an invalid value now errors instead of being silently ignored
- `extract_loop_count` no longer panics on truncated files

### ⚠️ Migration / behavior changes
- `info`/JSON `loop_count` is now `1` (not `0`) for GIFs without a loop block
- `--loop-count 0` still works but is deprecated → use `--infinite`
- `mod_gif(…)` is deprecated → `modify_gif(input, ModOptions { loop_mode, delay_all, delays, output, ..Default::default() })`
- `set_loop_count(path, n, …)` now uses the same meaning as `mod` (`n` = plays n times, `0` = infinite);
  before it wrote the raw file value (`3` played 4 times)
- `utils::loop_count::extract_loop_count` is deprecated → `read_loop_mode` / `extract_raw_loop_value`
- `commands::modify::apply_modifications` was removed → `modify_gif`
- `GifMetadata` / `FrameMeta` are `#[non_exhaustive]` (fields can be added without breaking)
- Deprecated items will be removed in a later release

### 🧩 Frames

- `info` now detects **frame-optimized GIFs** (frames that only store changes since earlier frames)
- New summary line: `🧩 Optimized : yes (1 keyframes / 5 frames)`
- `--show-frames` marks each frame 🔑 keyframe / Δ delta and shows its rect + disposal method
- New JSON fields: `keyframe_count`, `is_delta_optimized`, and per frame `left`, `top`, `width`,
  `height`, `disposal`, `uses_transparent_pixels`, `is_keyframe` (additive, non-breaking)
- Fixed panic in `info` for GIFs without a global palette
- 🖼️ **Correct frame previews for optimized GIFs**: new disposal-aware renderer composites earlier
  frames (Keep / Background / Previous) and no longer lets transparent pixels punch holes —
  verified pixel-identical to ImageMagick `-coalesce`
- `preview --raw`: show only the pixels stored in a frame (what a delta frame actually contains)
- New lib API: `get_frame_image_with_mode(path, frame, PreviewMode::{Composited, Raw})`
- New test fixtures in `tests/testdata/optimize/`
- Plan for frame editing: `docs/FRAME_EDITING_PLAN.md`

---

## [0.4.1] ✨ 2025-06-25 - Hotfix keep dimenstion for transparent images

- Fixed preview frame rendering for transparent images 🧙‍♂️✨
- Preview images now preserve full canvas dimensions with proper composite layering
- Verified with CLI and test suite

---

## [0.4.0] ✨ gifmeta v0.4.0 – Frame by Frame

This release introduces the ability to **extract any frame from a GIF** as a PNG — with support for:

* ✅ Output to file (`--output frame.png`)
* ✅ Output as base64 (`--as-base64`) for web previews or GUI tools
* ✅ Fully in-memory processing (no temp files)
* ✅ Doc-tested + integration-tested for accuracy and safety

### ✅ Also includes
* Updated README with usage examples
* Feature added to the `--help` CLI
* Internal helper: `extract_frame_as_png(path, frame)`
* Doc test now verifies PNG header and writes file
* Version bumped to `0.4.0` on crates.io 🎉

---

## [0.3.0] ✨ gifmeta v0.3.0 — Sacred CLI for GIF Metadata

This release marks a meaningful evolution of `gifmeta`, now with test coverage, visual previews, and multi-platform binaries.

### 🌟 Highlights
- 🌀 Modify GIF loop count and frame delays with a single command
- 🧠 Smart delay editing using frame-index maps
- 🧪 CI-integrated test suite with HTML previews
- 📦 Prebuilt binaries for macOS, Windows, and Linux
- ⚙️ New `--json` flag for structured output

---

## [0.2.0] – 2025-06-14

✨ First meaningful release

### Added
- `info` command: show frame count, size, and total duration
- `get-loop`: extract loop count (NETSCAPE2.0 extension)
- `set-loop`: modify or insert loop count (supports infinite loop via `0`)
- `set-delay`: apply fixed delay to all frames
- `show-frame-delays`: display delay values for every frame
- `set-frame-delay`: define custom delays for selected frames (frame index + delay pairs)
- Hybrid CLI and library design (usable via command line or Rust)
- Clear `clap v4` CLI with long options like `--input`, `--output`, etc.
- `parse_csv` helper to support user-friendly comma-separated input

### Changed
- CLI now uses named flags (`--input`, `--count`, `--output`) for clarity and future flexibility

### Notes
- Batch processing and WebAssembly are intentionally left out.
- GIF preview and GUI planned for future versions.

---

## [0.1.0] – *Skipped / unreleased stub version*
This version was never published and is skipped to avoid ghost crates on crates.io.

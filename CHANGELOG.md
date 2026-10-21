# 📝 Changelog

All notable changes to **gifmeta** will be documented here.

---

## [Unreleased] 🧩 Delta-frame detection

- `info` now detects **frame-optimized GIFs** (frames that only store changes since earlier frames)
- New summary line: `🧩 Optimized : yes (1 keyframes / 5 frames)`
- `--show-frames` marks each frame 🔑 keyframe / Δ delta and shows its rect + disposal method
- New JSON fields: `keyframe_count`, `is_delta_optimized`, and per frame `left`, `top`, `width`,
  `height`, `disposal`, `uses_transparent_pixels`, `is_keyframe` (additive, non-breaking)
- Fixed panic in `info` for GIFs without a global palette
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

# 📝 Changelog

All notable changes to **gifmeta** will be documented here.

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

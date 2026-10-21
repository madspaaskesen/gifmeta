# 🧩 Frame editing plan — detect, re-key, delete, unoptimize

Living plan for adding frame-level editing to **gifmeta** (crate + CLI) and **gifmeta-gui**.
Status markers: ⬜ todo · 🟨 in progress · ✅ done

---

## Background: delta-optimized GIFs

Many GIFs are *frame-optimized* (gifsicle `-O`, ImageMagick `-layers Optimize`). Each frame only
stores what changed since the previous one. This is **not** interlacing — interlacing is just the
row order inside a single image.

Three techniques, often combined:

| Technique | How it shows up in the file |
|---|---|
| Sub-rectangle frames | frame `left/top/width/height` smaller than the logical screen |
| Transparency optimization | unchanged pixels set to the frame's transparent index, so the previous canvas shows through |
| Disposal method | what happens to a frame's area *before the next frame is drawn*: `Any/Keep` (leave it), `Background` (clear to transparent/background), `Previous` (restore canvas to how it was before this frame) |

Consequence: a frame can depend on every frame before it. Deleting or reordering a frame changes
what later frames are drawn on top of → visual glitches.

### Keyframe rule (implemented in `src/utils/frame_analysis.rs`)
We track a conservative bounding box of the canvas area that still holds content from earlier
frames (the *dirty area*). A frame is a **keyframe** (independent of earlier frames) when:
1. the canvas is empty — always true for frame 0, **or**
2. the frame's rect covers the whole dirty area **and** none of its pixels use the transparent index.

After each frame the dirty area is updated from its disposal:
`Any/Keep` → union with the frame rect · `Background` → empty if the frame rect covered all earlier
content, otherwise unchanged · `Previous` → unchanged (restored).

Everything else is a **delta frame**. The rule may call an independent frame "delta" in rare
cases (bounding boxes are coarse) but never calls a dependent frame a keyframe — the safe direction
for deleting/re-keying.

Why not just "full canvas = keyframe"? `tests/testdata/transparency/stars-transparent.gif` uses
small frames that each clear themselves (`Background`), so every frame is drawn on an empty canvas.
The naive rule called 149 of 150 frames delta; the dirty-area rule correctly finds 150 keyframes.

Note: "restore to background" — browsers clear to transparent, not to the background colour index.
We follow browsers.

---

## Phases

### Phase 1 — Detection in the crate ✅ (unreleased)
Pure analysis, no file changes. `commands/info.rs` feeds each frame into
`utils::frame_analysis::KeyframeTracker` (indexed decode → transparent-pixel check is free).

- `FrameMeta` gains: `left`, `top`, `width`, `height`, `disposal` (own `Disposal` enum, serialised
  lowercase), `uses_transparent_pixels`, `is_keyframe`.
- `GifMetadata` gains: `keyframe_count`, `is_delta_optimized` (≥ 1 delta frame). Always computed,
  even without `--show-frames`.
- CLI `info`: summary line `🧩 Optimized : yes (3 keyframes / 10 frames)`; `--show-frames` lists
  🔑 / Δ per frame plus rect + disposal.
- Tests: unit tests in `frame_analysis.rs`; `tests/frame_analysis.rs` covers
  `optimize/moving-square-full.gif` (5/5 keyframes), `optimize/moving-square-optimized.gif`
  (1/5, made with `-layers Optimize`, added to `run-create-tests.sh`), `layout/offset-frames.gif`
  and the self-clearing `stars-transparent.gif`.
- Also fixed: `info` panicked on GIFs without a global palette.
- Additive JSON change only → no breaking change for existing consumers.

### Phase 2 — Show it in the GUI ⬜
- Bump GUI dep (while developing: temporary `gifmeta = { path = "../../gifmeta" }`, do not commit).
- Frame list: small 🔑 / Δ badge per frame.
- Status bar: `Optimized: 3/10 keyframes`.
- FrameViewer toggle **Pretty / Raw** (new Tauri command param `raw: bool` → `get_frame_image_with_mode`);
  raw view on a checkerboard so transparency is visible. Cache images per mode.
- FrameViewer: rect (`left,top width×height`), disposal, keyframe yes/no.

### Phase 3 — Correct canvas renderer ✅ (unreleased) (prerequisite for 4–6)
A single disposal-aware compositor in the crate that replays the animation and yields the true
RGBA canvas for every frame. Reused by preview, re-keying, delete and unoptimize.
- Handle `Keep/Any`, `Background` (clear only the frame's rect), `Previous` (snapshot/restore).
- Transparent pixels must **not** overwrite the canvas (current `extract_frame_as_png` copies them).
- Replace the naive loop in `utils/extract_frame_as_png.rs` → previews of optimized GIFs become correct.
- Tests: compare rendered frames of an optimized fixture against the un-optimized source
  (`convert -coalesce`) pixel-for-pixel.
- Done: `src/utils/render.rs` (`render_frame(path, index, PreviewMode)`), `tests/render.rs`.
  Also verified on the 40-frame `forge-preview.gif` (800×516, 5 MB — too big for the repo):
  0 differing pixels vs ImageMagick `-coalesce` on frames 1, 20, 39.
- Bonus: `PreviewMode::Raw` / `preview --raw` shows only the pixels stored in a frame.
  GUI: add a "Pretty / Raw" toggle in FrameViewer (Phase 2), raw view on a checkerboard background.

### Phase 4 — Convert a frame to a full image ("re-key") ⬜
- Crate: `mod --keyframe 3,7` / lib `keyframes: Option<Vec<usize>>`.
- Writes the rendered canvas of that frame as a full-canvas frame; all other frames are copied unchanged.
- Keep the frame's original disposal. Because the canvas after the re-keyed frame equals the original
  canvas, and the canvas before it is untouched, `Background` and `Previous` disposal still produce
  the same result for later frames. (Verify with a test per disposal method.)
- GUI: checkbox per frame "Convert to full image".

### Phase 5 — Delete frames ⬜
- Crate: `mod --delete 2,5` (`parse_csv::<usize>`) / lib `delete_frames: Option<Vec<usize>>`.
- All indices (delays, keyframes, deletes) refer to **original** frame indices.
- Algorithm — *shadow simulation*: replay original and new output side by side with the Phase 3
  renderer. Before writing each kept frame, compare the new canvas with the original canvas at
  the same point. Identical → copy the frame unchanged (cheap, lossless). Different → write the
  frame's fully rendered canvas instead (auto re-key). Usually only the frame right after a
  deleted block gets re-keyed; the simulation also catches `Previous` disposal chains, consecutive
  deletes and deleting frame 0 without special cases.
- Delay policy (decision): `drop` (animation gets shorter) vs `merge` (add the deleted delay to the
  previous kept frame, total duration unchanged). Proposed default: `merge`, flag `--delete-mode drop`.
- Refuse deleting all frames; error on out-of-range indices.
- Extend `mod_gif`'s "no modifications" check.
- GUI: checkbox per frame "Delete frame"; deleted frames dimmed/struck through and excluded from
  status-bar duration + frame count; send `deleteFrames` to `save_modified_gif`.

### Phase 6 — Unoptimize everything ("lazy mode") ⬜
- Crate/CLI: `mod --unoptimize` (alias `--coalesce`) → every frame written as a full keyframe.
- Same machinery as Phase 4 applied to all frames. Bigger files, but every frame is independent
  and safe to edit in any tool.
- Maybe later: the reverse (`--optimize`) — out of scope for now, gifsicle does it well.

### Release plan (proposal)
- **0.5.0** — Phase 1 (+ Phase 3 if ready) → GUI can ship the markers (Phase 2).
- **0.6.0** — Phases 4 + 5.
- **0.7.0** — Phase 6.
Each release: Cargo.toml version, CHANGELOG, README usage, `cargo fmt && cargo test`, publish, GitHub release.

---

## ⚠️ Pitfalls

- **More than 256 colours in a re-keyed frame.** A rendered canvas can mix colours from several
  local palettes. If every pixel colour exists in the global palette (or one palette) → map exactly,
  lossless. Otherwise we must quantize that frame (`gif::Frame::from_rgba_speed`, NeuQuant) → small
  quality loss on that frame only. Decision: quantize with a warning, or error out (first version).
- **Transparency in a re-keyed frame.** Genuinely transparent canvas pixels need a transparent index
  in the new frame's palette, reserving one of the 256 slots.
- **File size.** Re-keyed/full frames are larger than deltas. Expected and fine; mention it in docs.
- **`Previous` disposal.** Restores to the state before the frame — if that state included a deleted
  frame, later frames break. Handled by shadow simulation (Phase 5), not by special-casing.
- **Loop restart.** Viewers clear the canvas when the animation loops, so frame 0 is drawn on an
  empty canvas. Deleting the last frame is therefore safe; deleting frame 0 forces a re-key of the new first frame.
- **Background colour vs transparent.** The spec says "restore to background colour"; browsers clear
  to transparent. Follow browsers, document it.
- **Index semantics.** Mixing original and new indices is an easy bug — always original indices in the API/GUI.
- **Never decode to RGBA for writing unchanged frames.** `modify.rs` clones indexed `gif::Frame`s,
  which preserves palettes, rects, disposal and transparency. Only re-keyed frames are re-encoded.

## 🐞 Existing issues found during analysis

- **Loop flag lost/changed:** a GIF without a NETSCAPE2.0 block (plays once) becomes **infinite**
  after `mod` without `--loop-count`. Reproduced with `tests/testdata/loop/no-loop-flag.gif`.
  Cause: `extract_loop_count` errors → treated as `0` → `Repeat::Infinite`. Fix: return `Option`,
  and when absent don't write a NETSCAPE block at all.
- **Loop-count semantics are inconsistent:** `info`/`mod` use user-facing counts (N+1), while
  `set_loop_count` writes the raw value.
- `extract_loop_count` reads `buffer[i+1]` / `buffer[i+3..i+14]` without bounds checks → can panic on truncated files.
- ~~`extract_frame_as_png` ignores disposal and lets transparent pixels overwrite the canvas → wrong
  previews for optimized GIFs~~ — fixed in Phase 3.
- `mod` without `--output` writes to `$TMPDIR/<uuid>.mod.gif` (not documented); `set_frame_delay`
  without output overwrites the input — inconsistent.
- README examples use `--input`, but INPUT is positional.
- ~~`info` panics when the GIF has no global palette (`global_palette_size.unwrap()`)~~ — fixed in Phase 1.
- `cargo test` doctests modify fixtures (`tests/testdata/loop/2.gif`) and write `modified-*.gif` into testdata.
- `show_frame_delays` / `set_selected_frame_delays` in `lib.rs` are stubs.
- GUI: path handling with `split("/")` breaks on Windows; `SettingsTab.applyToAllChange` references
  undefined `delay_cs` (dead code); some JSX uses `class=`; save feedback via `alert()`.

## ❓ Open decisions

1. Delete delay policy default: `merge` or `drop`?
2. >256 colours on re-key: quantize with warning, or error in the first version?
3. Release 0.5.0 with detection only, or wait for the renderer (Phase 3)?
4. Fix the loop-flag bug in 0.5.0? (Recommended — small and user-visible.)

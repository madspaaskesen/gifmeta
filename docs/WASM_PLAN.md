# 🌐 WASM plan — gifmeta in the browser

Future plan for running **gifmeta** (crate) directly in the browser on **gifmeta.com**, as
WebAssembly. Nothing started yet. Status markers: ⬜ todo · 🟨 in progress · ✅ done

---

## Why WASM (and not a Rust backend)

| | WASM in the browser | Rust backend (server) |
|---|---|---|
| Privacy | ✅ the file never leaves the user's computer | ❌ upload needed (GDPR, trust) |
| Cost / ops | ✅ none, gifmeta.com stays a static Astro site | ❌ server, queue, limits, abuse (huge GIFs) |
| Speed | ~1.5–2× slower than native, no upload/download | native speed, but upload + download time |
| Memory | browser limit (~2–4 GB, wasm32) | as much as the server has |
| Offline | ✅ works once loaded | ❌ |

**Decision (proposal):** WASM first. A backend only if we ever want a public API or batch jobs —
it can use the exact same crate, so it isn't either/or.

Selling point for the website: *"Edit GIFs in your browser — nothing is uploaded."*
Matches the local-first idea of gifmeta-gui and Flixlar.

Note: the README says "Batch processing and WASM are intentionally skipped". That was right for a
CLI tool; with gifmeta.com it's worth revisiting → update the README when this starts.

---

## Where we are (0.7.0, static review — not yet compiled to wasm)

- ✅ **Core dependencies are pure Rust** and should build for `wasm32-unknown-unknown`: `gif`,
  `color_quant`, `image` (0.7.0 trimmed it to `png`, `jpeg` (zune-jpeg), `webp` (image-webp) —
  rayon/avif/exr are gone, which were the usual wasm troublemakers), `serde`, `serde_json`, `base64`.
- ❌ **The API is file-path based.** Every public function takes `PathBuf`/`String` paths and the
  internals open `File`s (`info.rs`, `modify.rs`, `create.rs`, `render.rs`, `loop_count.rs`,
  `load_image.rs`, `set_frame_delay.rs`). The browser has no file system → main refactor.
- ❌ **Temp files:** `modify`/`create` default to `$TMPDIR/<uuid>.gif` (`uuid` + `std::env::temp_dir`),
  `set_frame_delay` uses `tempfile`. `uuid` v4 needs `getrandom` with the `js` feature on wasm.
- ❌ **CLI-only deps in the lib:** `clap`, `tempfile`, `uuid` are always compiled.
- ⚠️ **Printing:** 9 `println!`/`eprintln!` in lib code (mostly old helpers). Harmless on wasm
  (goes nowhere) but should not be in a library anyway.
- ✅ `loop_count.rs` already patches the NETSCAPE block on raw bytes → easy to make byte-based.
- ✅ `modify` decodes the input twice (indexed + RGBA) — works the same from a byte slice
  (`Cursor<&[u8]>` twice).

---

## Phases

### Phase W0 — Spike: does it compile? ⬜
- `rustup target add wasm32-unknown-unknown`
- `cargo check --lib --target wasm32-unknown-unknown` → list what breaks (expected: `uuid`/`getrandom`,
  `tempfile`). Note the results here.
- Quick size check of a release build (`wasm-opt -Oz`).

### Phase W1 — Byte-based core API ⬜ (useful even without wasm)
- Each operation gets a bytes version; the path functions become thin wrappers:
  - `get_metadata_from_bytes(&[u8], show_frames) -> GifMetadata`
  - `modify_gif_bytes(&[u8], &ModOptions) -> (Vec<u8>, ModReport)`
  - `create_gif_from_bytes(&[ImageBytes], &CreateOptions) -> (Vec<u8>, CreateReport)`
  - `FrameRenderer::from_bytes(&[u8], PreviewMode)`, `get_all_frame_images_from_bytes`
- Internally: decoders take `impl Read` (`Cursor` for bytes, `BufReader<File>` for paths), the
  encoder writes to `impl Write` (`Vec<u8>` or `BufWriter<File>`).
- `FrameInsert` needs the image as bytes too: e.g. `enum ImageSource { Path(PathBuf), Bytes(Vec<u8>) }`
  (decided in W1; keep `FrameInsert::new(before, path)` working).
- Bonus for **gifmeta-gui**: Tauri can pass bytes directly instead of writing temp files.
- No behaviour change → all existing tests must stay green; add a few `_bytes` tests that compare
  with the path versions.

### Phase W2 — Feature flags ⬜
- `[features] default = ["cli"]`, `cli = ["dep:clap"]`; the binary gets `required-features = ["cli"]`.
- Temp-file defaults (`output: None` → `$TMPDIR/<uuid>`) only in the path API, behind a `fs` feature
  (default on) — or replace `uuid` with a timestamp + counter name and drop the dependency.
- `set_frame_delay` (`tempfile`) is old/duplicate functionality → deprecate or move behind `fs`.
- Remove `println!` from lib functions (keep messages in `main.rs`).
- CI: add `cargo check --lib --no-default-features --target wasm32-unknown-unknown`.

### Phase W3 — `gifmeta-wasm` crate ⬜
- New crate (workspace member or separate repo) with `wasm-bindgen`; built with `wasm-pack build --target web`.
- Thin JS-facing API (bytes in, bytes + JSON report out):
  `info(bytes) → JSON`, `modify(bytes, optionsJson) → { gif, report }`,
  `create(images[], optionsJson) → { gif, report }`, `frames(bytes, raw) → PNG[]` (or an iterator).
- Options as JSON (serde) so the JS side mirrors `ModOptions` / `CreateOptions`.
- `console_error_panic_hook` for readable errors in dev.
- Publish to npm? Optional — first just ship it inside gifmeta-frontend-astro.

### Phase W4 — Web app on gifmeta.com ⬜
- Page `/app` (or `/edit`) in **gifmeta-frontend-astro**: drop a GIF → frame list (🔑/Δ badges),
  preview, delays, loop, delete / insert / duplicate / drag-to-reorder, Save (download).
- "Make a GIF": drop images → sortable list → delay/loop → download.
- Run gifmeta in a **Web Worker** so the page never freezes; lazy-load the `.wasm` only when a file
  is dropped (keeps the landing page fast).
- Size guard: warn above e.g. 50 MB / large width × height × frames (see pitfalls).
- Nothing is uploaded — say it clearly in the UI. Analytics only counts events, never file data.
- SEO: "GIF editor online", "make GIF from images", "delete frames from GIF", "change GIF speed",
  "make GIF loop forever" — one small landing section per task linking to the app.

### Phase W5 — Later / maybe ⬜
- Threads (rayon via `wasm-bindgen-rayon`, needs COOP/COEP headers for `SharedArrayBuffer`) — only if
  speed becomes a problem.
- Phase 9 "Optimize" in the browser (shrink file size) — big search term, pairs well with W4.
- Public Rust backend / API using the same crate — only if there is demand.

---

## ⚠️ Pitfalls

- **Memory.** wasm32 = max 4 GB address space, browsers often less. A rendered frame is
  `width × height × 4` bytes. `--order` keeps every frame + 2 canvases in memory (≈ 3 × that per
  frame); `render_all_frames` keeps all frames. 800×516 × 40 frames ≈ 66 MB per copy (fine), but
  1920×1080 × 300 frames ≈ 2.5 GB per copy (not fine) → size guard + streaming where possible.
- **Copies across the JS/wasm boundary.** Every `Uint8Array` ↔ `Vec<u8>` is a copy. Pass the GIF once,
  return PNGs/frames lazily (per frame or as one transferable buffer) instead of many small calls.
- **Main thread.** Even 1 s of work freezes the page → always a Web Worker.
- **Bundle size.** Expect ~0.5–1 MB gzipped (image decoders are the biggest part). Lazy-load;
  consider a feature without `jpeg`/`webp` for an "edit GIF only" build.
- **Randomness / time.** `uuid`, `getrandom`, `std::time::SystemTime` panic or fail on
  `wasm32-unknown-unknown` without extra features → keep them out of the wasm build (W2).
- **Errors are `String`.** Fine for JS (`throw new Error(msg)`), but a proper error enum would
  allow translated UI messages later.

## ❓ Open decisions

1. Workspace (`gifmeta/` + `gifmeta-wasm/` in one repo) or a separate `gifmeta-wasm` repo?
2. `ImageSource` enum vs separate `_bytes` functions for inserts/create.
3. Drop `uuid` entirely (temp name from timestamp + counter) or keep it behind the `fs` feature?
4. Publish `gifmeta-wasm` to npm, or only use it on gifmeta.com?
5. Release order: W1 + W2 as a normal crate release (e.g. 0.8.0) before any web work?

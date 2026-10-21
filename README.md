# 🦀 gifmeta

**gifmeta** is a small, focused tool for inspecting and modifying GIF metadata.  
It’s designed for developers, artists, and anyone needing **loop control** and **timing tweaks** — fast and reliably.

> ❤️ Built with Rust. Sacredly simple. Zero bloat.

[![Crates.io](https://img.shields.io/crates/v/gifmeta.svg)](https://crates.io/crates/gifmeta)
[![Downloads](https://img.shields.io/crates/d/gifmeta.svg)](https://crates.io/crates/gifmeta)
[![License](https://img.shields.io/crates/l/gifmeta.svg)](https://crates.io/crates/gifmeta)
[![CI](https://img.shields.io/github/actions/workflow/status/madspaaskesen/gifmeta/ci.yml)](https://github.com/madspaaskesen/gifmeta)
![Tests](https://github.com/madspaaskesen/gifmeta/actions/workflows/ci.yml/badge.svg)

---

## 🧍‍♂️ Why I Built This

Because changing loop count or frame delays in GIFs was a pain.

Most tools were too heavy, too raw, or lacked reliable preview/testing.  
`gifmeta` was born to fix that — with precise CLI commands, minimalism, and full test coverage.

![Rust](https://img.shields.io/badge/built_with-rust-orange)
![FFmpeg](https://img.shields.io/badge/rendered_by-ffmpeg-blue)
[![SacredAI](https://img.shields.io/badge/powered%20by-%F0%9F%95%8A%EF%B8%8F%20Sacred%20AI-lightgrey)](https://sacre-ai.com)
[![mySiteChart](https://badges.mysitechart.com/github-project/madspaaskesen/gifmeta?site=COM)](https://mysitechart.com/In?github-project=madspaaskesen%2Fgifmeta)

---

## 🔧 Features

- 🔁 Set infinite or fixed loop count (`--loop-count`)
- ⏱️ Apply fixed delay to all frames (`--delay`)
- 🎯 Set specific frame delays (`--delays 1=30,4=60`)
- 📤 Output JSON metadata (`--json`)
- 🖼️ Extract any frame as PNG (to file or base64)
- 🧩 Detect frame-optimized GIFs (🔑 keyframes vs Δ delta frames)
- 🧪 CLI + doc tests for core functions
- 🖼️ Side-by-side **visual preview reports** via `run-visual-preview.sh`

---

## 🙋‍♂️ Why Not Use gifsicle or gif-rs?

You can — and should, if you need full encoding pipelines.

But `gifmeta` wraps just the things most of us need *quickly*:
- 🔁 Loop control (infinite or fixed)
- ⏱ Frame delay override (global + selective)
- ✅ Verified, minimal changes
- 🔍 Human-friendly reports
- ⚙️ Ideal for future GUI integration or scripting

---

## ⚠️ What It’s Not

- ❌ Not a full GIF encoder
- ❌ Not for image content editing
- ❌ Not a general-purpose wrapper

`gifmeta` lives in a small, useful space — metadata tuning and clarity. Nothing more.

---

## 🧠 Why This Exists

GIFs are still everywhere — especially in messaging, memes, and UI loaders. But modifying timing or loop behavior often requires heavy tools.

`gifmeta` lets you do it instantly, with:

* 🦀 Native Rust speed
* 🧼 Zero bloat
* 🖥️ CLI-first ergonomics

---

## 📦 Installation

Install via [crates.io](https://crates.io/)!:
```bash
cargo install gifmeta
```

Or use directly with:

```bash
cargo run -- <command>
```

Install via GitHub
```bash
git clone https://github.com/madspaaskesen/gifmeta.git
cd gifmeta
cargo build --release
./target/release/gifmeta --help
```

Absolutely. Here’s a clean and helpful section you can add to your `README.md` under the **Installation** section:

---

### 🧰 Installation

Download the binary for your platform from the [latest release](https://github.com/madspaaskesen/gifmeta/releases).

#### ✅ macOS Users

If you see a warning when trying to run `gifmeta`, macOS may have quarantined the file because it's unsigned.

**To allow execution:**

```bash
xattr -d com.apple.quarantine gifmeta
chmod +x gifmeta
```

Then run it with:

```bash
./gifmeta info yourfile.gif
```

> You only need to do this once per download. After this, `gifmeta` will run normally.

---

Let me know if you want the same clarity for Windows (e.g., SmartScreen warnings) or Linux permissions 💛


---

## 🛠️ Usage

### Info options

- `--json`: Get output in json format

### Modify (mod) options

You can combine any of the following:

- `--loop-count <n>`: Set loop behavior (0 = infinite, 1 = once, 2 = twice…)
- `--delay <n>`: Set same delay for all frames. Unit is **centiseconds** (1 = 10ms).
- `--delays <csv>`: Override specific frames. Format: `1=15,3=50` (in centiseconds).

### Show metadata
```bash
gifmeta info --input tests/testdata/loop/10frame-rainbow.gif
```

### Show metadata output as json
```bash
gifmeta info --input tests/testdata/loop/10frame-rainbow.gif --json
```

### Detect delta-optimized frames
```bash
gifmeta info tests/testdata/optimize/moving-square-optimized.gif --show-frames
```
Shows `🧩 Optimized : yes (1 keyframes / 5 frames)` and marks every frame 🔑 (keyframe, a full
image on its own) or Δ (delta frame that only stores changes on top of earlier frames).

### Modify loop count
```bash
gifmeta mod --input tests/testdata/loop/loop-once.gif --loop-count 10 --output tests/testdata/loop/loop-once-modified.gif
```

### Modify delay on all frames to 100ms or 10 centiseconds
```bash
gifmeta mod --input tests/testdata/timing/zero-delay.gif --delay 10 --output tests/testdata/timing/zero-delay-modified.gif
```

### Modify frame delays using centiseconds (1 = 10ms): frame 0 → 1, frame 1 → 20
```bash
gifmeta mod --input tests/testdata/timing/zero-delay.gif --delays "0=1,1=20" --output tests/testdata/timing/zero-delay-modified2.gif
```

### Modify delay on all frames to 150ms, set specific frame 0 to 20ms and frame 1 to 200ms.
```bash
gifmeta mod --input tests/testdata/timing/zero-delay.gif --delay 15 --delays "0=2,1=20" --output tests/testdata/timing/zero-delay-modified3.gif
```

### Modify all values at once example
```bash
gifmeta mod --input tests/testdata/timing/zero-delay.gif --loop-count 0 --delay 15 --delays "0=2,1=20" --output tests/testdata/timing/zero-delay-modified4.gif
```

### 🖼️ Preview a GIF Frame as PNG

You can now extract a specific frame from any GIF and output it as a PNG image — either as a file or directly as base64 for web previews:

#### 🔹 Output frame 0 as PNG to a file

```bash
gifmeta preview --input path/to/input.gif --frame 0 --output frame0.png
```

#### 🔹 Output frame 3 as base64 (for GUI, web, etc.)

```bash
gifmeta preview --input path/to/input.gif --frame 3 --as-base64
```

> If `--frame` is omitted, it defaults to the first frame (`0`).
> You must specify either `--as-base64` or `--output`.

---

## 🛠️ Commands

| Command   | Description                                       |
| --------- | ------------------------------------------------- |
| `info`    | Display GIF metadata (dimensions, loop, delays)   |
| `mod`     | Apply metadata modifications (loop/delays/output) |
| `preview` | Extract a single frame as PNG (file or base64)    |

---

## 📊 Visual Preview Report

You can run:

```bash
./run-visual-preview.sh
```

It will:

* Apply a set of metadata modifications
* Generate side-by-side comparisons
* Output a beautiful `tests/visual/report.html`

This is useful for testing transparency, timing, or human-visible playback changes.

---

## 🛤️ Roadmap

- [x] Set loop count (v0.2.0)
- [x] CLI structure with `clap v4`
- [x] Set per-frame delays (v0.3.0)
- [ ] Frame editing with duration visuals (experimental)
- [ ] GUI Companion (Tauri, optional)

⚠️ Batch processing and WASM are intentionally skipped.  
These are best handled externally with scripting or shell tooling.

---

## 👩‍💻 Contributing

Wanna help? Fork, clone, and PRs welcome. You can also suggest Codex tasks or open issues 💛

---

## 🔗 Links

- [Crates.io](https://crates.io/crates/gifmeta)
- [Documentation (coming soon)](https://docs.rs/gifmeta)
- [GitHub](https://github.com/madspaaskesen/gifmeta)

---

## 🛠 Built With

- [gif](https://docs.rs/gif/)
- Rust 2021
- Love, frustration, and sacred formatting

---

## 🕊️ License

MIT — do good things with it.

---

## 🌟 Author

Built by [Mads Paaskesen](https://github.com/madspaaskesen) with Rust and a soft spot for retro formats.

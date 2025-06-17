# 🦀 gifmeta

**gifmeta** is a small, fast, and sacred tool for inspecting and editing GIF metadata.  
Use it to extract loop counts, view frame delays, and set precise timing — all from the command line or via Rust.

> ❤️ Built with love and clarity. No unnecessary bloat.

---

## ✨ Features

- 🔍 Inspect frame count, size, and duration
- 🔁 View and modify loop counts
- 🕰️ Show per-frame delays (preview timing)
- 🎯 Set a fixed delay across all frames
- 🎯 Prepare for per-frame delay editing (coming in v0.3.0)

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
````

---

## 🛠️ Usage

### Show metadata
```bash
gifmeta info --input input.gif
```

### Get loop count
```bash
gifmeta get-loop --input input.gif
```

### Set loop count
```bash
gifmeta set-loop --input input.gif --count 3 --output out.gif
```

### Show all frame delays
```bash
gifmeta show-frame-delays --input input.gif
```

### Set delay for all frames
```bash
gifmeta set-delay --input input.gif --delay 10 --output out.gif
```

### Set frame specific delays
```bash
gifmeta set-frame-delay --input input.gif --frame-numbers 1,5,80 --delay-values 10,20,100 --output out.gif
```

---

## 🛠 Commands

| Command             | Description                               |
| ------------------- | ----------------------------------------- |
| `info`              | Show basic metadata (frames, size, delay) |
| `get-loop`          | Get the loop count from a GIF             |
| `set-loop`          | Set the loop count (use `0` for infinite) |
| `show-frame-delays` | Display the delay of every frame          |
| `set-delay`         | Apply a fixed delay to all frames         |
| `set-frame-delay`   | Set custom delays for specific frames     |

---

## 🧠 Why This Exists

GIFs are still everywhere — especially in messaging, memes, and UI loaders. But modifying timing or loop behavior often requires heavy tools.

`gifmeta` lets you do it instantly, with:

* 🦀 Native Rust speed
* 🧼 Zero bloat
* 🖥️ CLI-first ergonomics

---

## 🛤️ Roadmap

- [x] Set loop count (v0.2.0)
- [x] CLI structure refactored with `clap v4`
- [ ] Set per-frame delays via frame/index lists (v0.3.0)
- [ ] GIF preview tools
- [ ] Tauri GUI companion

---

## 👩‍💻 Contributing

Wanna help? Fork, clone, and PRs welcome. You can also suggest Codex tasks or open issues 💛

---

## 🔗 Links

- [Crates.io](https://crates.io/crates/gifmeta)
- [Documentation (coming soon)](https://docs.rs/gifmeta)
- [GitHub](https://github.com/madspaaskesen/gifmeta)

---

## 🕊️ License

MIT — do good things with it.

---

## 🌟 Author

Built by [Mads Paaskesen](https://github.com/madspaaskesen) with Rust and a soft spot for retro formats.

Yes please! 🫶 Here’s a polished **`README.md` version 1** for `gifmeta`, crafted with love and clarity — perfect for crates.io and GitHub.

---

````md
# 🎞️ gifmeta

> Inspect and edit GIF metadata from the command line.

`gifmeta` is a simple, fast CLI tool written in Rust to read and modify basic GIF metadata such as:

- ✅ Frame count
- ✅ Dimensions
- ✅ Loop count (including infinite)
- ✅ Frame delay (duration per frame)

This tool is ideal for automation, batch fixes, or inspecting media metadata in GIF workflows.

---

## 📦 Installation

Coming soon via [crates.io](https://crates.io/)! For now:

```bash
git clone https://github.com/madspaaskesen/gifmeta.git
cd gifmeta
cargo build --release
./target/release/gifmeta --help
````

---

## 🚀 Usage

```bash
gifmeta info mygif.gif
```

Prints metadata about the GIF.

```bash
gifmeta set-delay mygif.gif 10 -o new.gif
```

Sets each frame’s delay to 100ms (10 × 10ms), outputs to `new.gif`.

```bash
gifmeta set-loop mygif.gif 3 -o loop3.gif
```

Sets the loop count to 3 (or use `0` for infinite).

---

## 🛠 Commands

| Command     | Description                         |
| ----------- | ----------------------------------- |
| `info`      | Show metadata (frames, size, delay) |
| `set-delay` | Set a fixed delay for all frames    |
| `set-loop`  | Set loop count (0 = infinite)       |

---

## 🧠 Why This Exists

GIFs are still everywhere — especially in messaging, memes, and UI loaders. But modifying timing or loop behavior often requires heavy tools.

`gifmeta` lets you do it instantly, with:

* 🦀 Native Rust speed
* 🧼 Zero bloat
* 🖥️ CLI-first ergonomics

---

## 🧩 Roadmap

* [x] CLI with `clap`
* [x] Print metadata
* [ ] Modify individual frame delays
* [ ] Batch processing
* [ ] WebAssembly version?
* [ ] GUI wrapper?

---

## 👩‍💻 Contributing

Wanna help? Fork, clone, and PRs welcome. You can also suggest Codex tasks or open issues 💛

---

## 🪪 License

Licensed under **MIT OR Apache-2.0** — pick what works for you.

---

## 🌟 Author

Built by [Mads Paaskesen](https://github.com/madspaaskesen) with Rust and a soft spot for retro formats.


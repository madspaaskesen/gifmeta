# 🧪 GIFMeta Test Suite

This folder contains handcrafted test GIFs used for validating and developing the `gifmeta` tool.  
Each file is designed to represent a specific feature, edge case, or metadata quirk in the GIF format.

These files are created using `create-tests.sh`, and are grouped by purpose.

---

## 📂 loop/

| Filename               | Description                                  |
|------------------------|----------------------------------------------|
| `1.gif`                | Basic 2-frame infinite loop (original test)  |
| `2.gif`                | Multi-frame infinite loop (original test)    |
| `2frame-soft.gif`      | Lavender ↔ MistyRose soft alternating loop   |
| `10frame-rainbow.gif`  | 10-frame loop using HSL spectrum colors      |
| `loop-once.gif`        | Loops exactly once (`loop=1`)                |
| `no-loop-flag.gif`     | No loop metadata; depends on viewer default  |

---

## 📂 color/

| Filename                   | Description                             |
|----------------------------|-----------------------------------------|
| `grayscale.gif`            | Gradient from black to white            |
| `reduced-palette-16.gif`   | Gradient reduced to 16 colors           |
| `full-palette-256.gif`     | Full 256-color horizontal gradient      |

---

## 📂 timing/

| Filename               | Description                                 |
|------------------------|---------------------------------------------|
| `variable-delays.gif`  | 3 frames with different delays (20/100/50)  |
| `zero-delay.gif`       | 2 frames with zero delay                    |

---

## 📂 transparency/

| Filename                   | Description                              |
|----------------------------|------------------------------------------|
| `transparent-circle.gif`   | Transparent canvas with golden circle    |
| `stars-transparent.gif`    | Transparent background stars animation   |

---

## 📂 layout/

| Filename                 | Description                                |
|--------------------------|--------------------------------------------|
| `offset-frames.gif`      | Frames drawn with canvas offset           |
| `odd-dimension-wide.gif` | 800px wide, 1px tall — extreme size test  |

---

## 📝 TODO — Advanced Test Files (manual or external source)

These files represent rare or tool-specific behavior and will be added later:

| Intended Filename                   | Description                                      |
|-------------------------------------|--------------------------------------------------|
| `compression/compressed_photoshop.gif` | Exported from Photoshop (unique block metadata) |
| `compression/compressed_gimp.gif`      | Exported from GIMP (different compression hints) |
| `layout/reused_frame_variants.gif`     | Frame diff optimization (same image, new delay)  |

---

## 🛠 How to Regenerate All Test Files

Run the script from the root folder:

```bash
./create-tests.sh
```

This will safely regenerate all automatable test files without overwriting external ones.

---

🕊️ *Crafted with care as part of the Sacred Tools Initiative.*

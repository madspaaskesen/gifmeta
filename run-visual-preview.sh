#!/bin/bash
set -e

echo "✨ Generating GIFMeta Visual Preview Report..."

# CONFIG
INPUT="tests/testdata/loop/2frame-soft.gif"
OUTPUT="tests/visual/output/2frame-soft-modified.gif"
META_DIR="tests/visual/metadata"
CMD_LOG="tests/visual/commands.txt"
REPORT="tests/visual/report.html"

# Ensure folders exist
mkdir -p "$(dirname "$OUTPUT")"
mkdir -p "$META_DIR"

# Command to run
CMD="cargo run --quiet -- mod $INPUT --loop-count 0 --delay 10 --delays 1=20 --output $OUTPUT"

# Run command
eval "$CMD"
echo "✅ Mod done: $OUTPUT"

# Save command
echo "$CMD" > "$CMD_LOG"

# Save metadata
cargo run --quiet -- info $INPUT --json > "$META_DIR/original.json"
cargo run --quiet -- info $OUTPUT --json > "$META_DIR/modified.json"

# Generate report
cat <<EOF > $REPORT
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>GIFMeta Visual Report</title>
  <style>
    body { font-family: sans-serif; padding: 2em; }
    img { border: 1px solid #ccc; margin: 1em 0; max-width: 200px; }
    pre { background: #f9f9f9; padding: 1em; overflow: auto; }
    .side-by-side { display: flex; gap: 2em; }
    h2 { margin-top: 2em; }
  </style>
</head>
<body>

<h1>🧪 GIFMeta Visual Test Report</h1>

<h2>🔧 Command</h2>
<pre>$(cat "$CMD_LOG")</pre>

<div class="side-by-side">
  <div>
    <h2>🖼️ Original</h2>
    <img src="../../testdata/loop/2frame-soft.gif" alt="Original">
    <pre>$(cat "$META_DIR/original.json")</pre>
  </div>

  <div>
    <h2>🎨 Modified</h2>
    <img src="output/2frame-soft-modified.gif" alt="Modified">
    <pre>$(cat "$META_DIR/modified.json")</pre>
  </div>
</div>

</body>
</html>
EOF

echo "✅ HTML report generated at: $REPORT"

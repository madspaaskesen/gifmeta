#!/bin/bash

set -e

echo "🌱 Creating sacred test data for GIFMeta..."

# Root folder
TESTDIR="tests/testdata"
mkdir -p "$TESTDIR"

# Subfolders
mkdir -p "$TESTDIR"/{loop,color,timing,transparency,layout}

# Loop: 2-frame soft loop
convert -size 100x100 xc:lavender "$TESTDIR/loop/frame1.png"
convert -size 100x100 xc:mistyrose "$TESTDIR/loop/frame2.png"
convert -loop 0 -delay 100 "$TESTDIR/loop/frame1.png" "$TESTDIR/loop/frame2.png" "$TESTDIR/loop/2frame-soft.gif"
rm "$TESTDIR/loop/frame1.png" "$TESTDIR/loop/frame2.png"

# Loop: 10-frame rainbow loop
for i in {1..10}; do
  convert -size 120x120 xc:"hsl($((i * 36)),100%,75%)" "$TESTDIR/loop/rainbow_$i.png"
done
convert -loop 0 -delay 50 "$TESTDIR/loop"/rainbow_*.png "$TESTDIR/loop/10frame-rainbow.gif"
rm "$TESTDIR/loop"/rainbow_*.png

# Color: 16-color palette
convert -size 120x120 gradient:yellow-red -colors 16 "$TESTDIR/color/reduced-palette-16.gif"

# Color: Grayscale
convert -size 120x120 gradient:black-white "$TESTDIR/color/grayscale.gif"

# Timing: Variable delays
convert -delay 20 xc:lightblue -delay 100 xc:lightgreen -delay 50 xc:lightpink "$TESTDIR/timing/variable-delays.gif"

# Timing: Zero delay
convert -delay 0 xc:white -delay 0 xc:gray "$TESTDIR/timing/zero-delay.gif"

# Transparency: Binary transparent circle
convert -size 120x120 xc:none -fill gold -draw "circle 60,60 60,20" -alpha on "$TESTDIR/transparency/transparent-circle.gif"

# Layout: Offset frames
convert -size 100x100 xc:none -draw "rectangle 10,10 60,60" "$TESTDIR/layout/frame1.png"
convert -size 100x100 xc:none -draw "rectangle 30,30 80,80" "$TESTDIR/layout/frame2.png"
convert -loop 0 -delay 100 -page +0+0 "$TESTDIR/layout/frame1.png" -page +20+20 "$TESTDIR/layout/frame2.png" "$TESTDIR/layout/offset-frames.gif"
rm "$TESTDIR/layout/frame1.png" "$TESTDIR/layout/frame2.png"

# Loop: loop once
convert -loop 1 -delay 100 xc:plum -delay 100 xc:pink "$TESTDIR/loop/loop-once.gif"

# Loop: no loop flag (default depends on viewer)
convert -delay 100 xc:gold -delay 100 xc:orange "$TESTDIR/loop/no-loop-flag.gif"

# Color: Full 256-color palette gradient
convert -size 256x1 gradient: -colors 256 -scale 256x120 "$TESTDIR/color/full-palette-256.gif"

# Layout: odd dimension wide (1x800)
convert -size 800x1 xc:deepskyblue "$TESTDIR/layout/odd-dimension-wide.gif"

echo "✅ Sacred test GIFs created under $TESTDIR"

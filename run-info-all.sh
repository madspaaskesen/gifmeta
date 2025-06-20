#!/bin/bash

echo "🧪 Running gifmeta info on all test GIFs..."

for file in tests/testdata/**/*.gif; do
  echo "🔍 Inspecting: $file"
  cargo run --quiet -- info "$file" --show-frames
  echo ""
done

#!/usr/bin/env bash
# Extracts vanilla textures, models, sounds, lang and other assets from the
# official client.jar into ./assets (never commit the jar itself).
# Usage: scripts/extract_assets.sh path/to/client.jar
set -euo pipefail

CLIENT_JAR="${1:-client.jar}"
OUT_DIR="${2:-assets}"

if [ ! -f "$CLIENT_JAR" ]; then
  echo "client jar not found: $CLIENT_JAR" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
unzip -q -o "$CLIENT_JAR" 'assets/*' -d "$OUT_DIR"
echo "assets extracted to $OUT_DIR/assets"

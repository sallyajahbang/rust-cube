#!/usr/bin/env bash
# Menyiapkan data Minecraft untuk Rust Cube:
#   1. download client.jar dan server.jar dari server resmi Mojang
#   2. verifikasi SHA-1
#   3. ekstrak assets/minecraft/ ke ./assets
#   4. jalankan data generator (server.jar) ke ./generated
#   5. buat src/world/blocks_generated.rs dari generated/reports/blocks.json
#
# Pemakaian: ./scripts/setup-assets.sh [versi]   (default: 26.3)
set -euo pipefail

VERSION="${1:-26.3}"
MANIFEST_URL="https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DL="$ROOT/downloads"

for cmd in curl jq unzip sha1sum java python3; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Butuh '$cmd' tapi belum terpasang." >&2
    exit 1
  fi
done

mkdir -p "$DL"
cd "$DL"

echo "==> Mengambil manifest versi"
curl -fsSL "$MANIFEST_URL" -o manifest.json
VERSION_URL=$(jq -r --arg v "$VERSION" '.versions[] | select(.id==$v) | .url' manifest.json)
if [ -z "$VERSION_URL" ] || [ "$VERSION_URL" = "null" ]; then
  echo "Versi $VERSION tidak ditemukan di manifest." >&2
  exit 1
fi
curl -fsSL "$VERSION_URL" -o "$VERSION.json"

# fetch <key> <file>: download kalau belum ada atau SHA-1 tidak cocok, lalu verifikasi.
fetch() {
  local key=$1 file=$2 url sha
  url=$(jq -r ".downloads.$key.url" "$VERSION.json")
  sha=$(jq -r ".downloads.$key.sha1" "$VERSION.json")
  if [ -f "$file" ] && echo "$sha  $file" | sha1sum -c --status 2>/dev/null; then
    echo "==> $file sudah ada dan valid, dilewati"
    return
  fi
  echo "==> Download $file"
  curl -fL "$url" -o "$file"
  if ! echo "$sha  $file" | sha1sum -c --status; then
    echo "SHA-1 $file tidak cocok, file dihapus." >&2
    rm -f "$file"
    exit 1
  fi
}

fetch client client.jar
fetch server server.jar

echo "==> Ekstrak assets/minecraft/"
rm -rf "$ROOT/assets"
unzip -q client.jar 'assets/minecraft/*' -d "$ROOT"

echo "==> Jalankan data generator (butuh Java 25 untuk 26.x)"
rm -rf "$DL/generated" "$ROOT/generated"
java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --all --output "$DL/generated"
mv "$DL/generated" "$ROOT/generated"

echo "==> Generate src/world/blocks_generated.rs"
python3 "$ROOT/tools/gen_blocks.py" \
  "$ROOT/generated/reports/blocks.json" \
  "$ROOT/src/world/blocks_generated.rs"

echo "Selesai. Data Minecraft $VERSION siap dipakai."

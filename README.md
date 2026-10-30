# Rust Cube

Klien Minecraft Java (single player) yang ditulis ulang di Rust, target Android arm64.
Target versi: **Minecraft 26.3**.

## Setup data game

Data Minecraft tidak ikut di repo (file milik Mojang). Jalankan sekali:

```bash
./scripts/setup-assets.sh        # atau: ./scripts/setup-assets.sh 26.3
```

Butuh: `curl`, `jq`, `unzip`, `sha1sum`, `python3`, dan Java 25 (untuk data generator 26.x).

Script ini membuat:
- `assets/minecraft/` dari client jar (texture, model, blockstate)
- `generated/` dari data generator server (blok, item, recipe, dan lainnya)
- `src/world/blocks_generated.rs` dari `generated/reports/blocks.json`

## Test dan build

```bash
cargo test
cargo build --release
```

CI di `.github/workflows/ci.yml` menjalankan test di Linux dan build Android arm64 dengan `cargo-ndk`.

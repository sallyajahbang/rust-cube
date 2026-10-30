# SallyMC (Rust, Android)

Rebuild of Minecraft Java Edition 26.3 in Rust for Android 10 (API 29), Vulkan renderer.
Test device: OPPO K3, Snapdragon 710, Adreno 616 (Vulkan 1.1).

## Layout
- `src/lib.rs`: android_main entry point and window lifecycle
- `src/render/`: Vulkan backend (stub for now)
- `scripts/extract_assets.sh`: pulls assets from the official client.jar
- `.github/workflows/build-android.yml`: builds the arm64 APK on GitHub Actions

## Build
Push to GitHub; download the `sallymc-arm64-debug-test` artifact and install the APK on the device.

## Phases
0. Window lifecycle + CI build (this commit)
1. Vulkan init: instance, surface, swapchain, clear color
2. Asset loading (blocks, textures, atlas)
3. Chunk meshing and rendering
4. World load/save (Anvil format), server protocol (26.3)
5. Entities, input, UI

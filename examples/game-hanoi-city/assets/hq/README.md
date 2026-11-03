# Hanoi City asset batch 01

This is a **technical integration candidate, not final production artwork**. The
five assets replace the corresponding stand-ins through `manifest.json`; the
game's existing loader reads this folder directly.

| Asset | LOD0 / LOD1 / LOD2 triangles |
| --- | --- |
| Minh | 23,294 / 7,856 / 1,913 |
| Cau The Huc | 58,206 / 14,716 / 2,332 |
| Cho co | 7,810 / 2,434 / - |
| Trom 01 | 24,210 / 7,853 / 1,912 |
| Xa cu | 7,836 / 2,908 / billboard |

All 13 GLBs embed their PBR images. `textures/` also contains the editable PNG
maps. ORM stores baked ambient occlusion in R, roughness in G and metallic in B.
Character maps are at most 1024 pixels; other maps are at most 2048 pixels.
The files use metres, +Y up and +Z forward; the bridge's north end points -Z.

The two characters retain the supplied 65-joint skeleton, joint order, hierarchy
and inverse bind matrices. Their animations come from the game's existing
`anim.glb`; those 43 clips are not duplicated here. Both dog LODs contain `Idle`,
`Walk`, `Run`, `Sit` and `Bark`.

## Evidence

- [Overview](previews/overview.jpg) and [individual previews](previews/).
- [Manifest](manifest.json): dimensions, LODs, tools, license and per-asset notes.
- [Structural checks](reports/spec-checks.json): triangle budgets, sizes and hashes.
- [Browser checks](reports/browser-checks.json): Chrome/Three.js loading of every
  GLB and 268 animation/LOD samples with finite deformation values.
- `reports/*.validator.json`: Khronos glTF Validator reports, zero errors and zero
  warnings for all 13 GLBs.
- [Provenance](SOURCES.md) and [CC0 license](LICENSE.txt).

## Remaining work

The shape, clothing, hair/fur and bark need artistic refinement to reach the
requested realism. Dog animations are basic integration clips. The bridge and
tree atlases do not provide a uniform 4 pixels/cm; repeating textures and
additional UV work are needed. These models are not site scans.

The tree billboard is padded to preserve its wide canopy. Its intended plane
size and pivot are in `manifest.json` as `billboard_plane_size_m` and
`billboard_pivot_uv`. The current `src/hq.rs` loader uses `size_m` instead and
does not consume those fields, so its distant billboard needs an integration
adjustment before visual approval.

Verification used Chrome/Three.js WebGL and a 390 x 844 mobile viewport emulator.
The assets have not been run or profiled in Bevy/WebAssembly/WebGPU or on a
physical phone. Binding and finite-value checks of all 43 clips do not constitute
frame-by-frame visual approval.

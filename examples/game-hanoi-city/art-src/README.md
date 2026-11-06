# Editable Hanoi City asset sources

These scripts produce draft assets for review. They do not modify the game or
publish a release. The cage and audio are not yet approved in-game.

## Cage

Use Python 3.11 with `sand3d==1.6.2`, its Blender 4.5 `bpy` dependency,
Pillow, NumPy and `rectpack`. Download the 1K diffuse JPG, OpenGL normal PNG
and ARM PNG from the CC0 Poly Haven `rusty_metal_02` source listed in
`assets/hq/SOURCES.md`; put them in a local `scans` directory with their
original names. Output directories in this example are scratch directories.

```sh
python sand3d/long_cho.py work/cage.glb
python sand3d/long_cho.py work/cage_high.glb --high-poly
python sand3d/finish_cage_one.py work/cage.glb work/cage_high.glb scans work/cage_result
python sand3d/long_cho.py work/cage_lod.glb --lod
python sand3d/long_cho.py work/cage_lod_high.glb --lod --high-poly
python sand3d/finish_cage_one.py work/cage_lod.glb work/cage_lod_high.glb scans work/cage_lod_result --texture-size 512
```

The finish pass packs one UV atlas set, bakes AO and selected-to-active normals,
combines the normal bake with the scan, and joins static and moving parts. The
hinged door remains separate. Review the result in Blender and
the real game, run Khronos glTF Validator, and update the manifest and reports
before replacing an asset. The scripts do not guarantee artistic acceptance.

## Flashlight

Using the same environment:

```sh
python sand3d/flashlight_qc_geometry.py work/flash.glb
python sand3d/flashlight_qc_geometry.py work/flash_high.glb --high-poly
python sand3d/finish_flashlight_qc.py work/flash.glb work/flash_result --high work/flash_high.glb
python sand3d/flashlight_qc_geometry.py work/flash_lod.glb --lod
python sand3d/finish_flashlight_qc.py work/flash_lod.glb work/flash_lod_result --size 256 --high work/flash_high.glb
```

The QC draft combines high-to-low normal baking with procedural normal detail
on the grip, a baked AO channel and separate reflector, rubber and emissive
`lamp_glass` materials. Knurl spacing, edge wear, `hand_r` placement and in-game
appearance remain to be checked before acceptance.

## Audio

`audio/ui_sounds.py` synthesizes the two UI sounds with NumPy and FFmpeg.
`audio/cc0_foley.py` converts the dog and iron-door recordings listed in
`assets/hq/SOURCES.md`. It expects `iron_door.ogg` and the extracted
`dog-source/Dog/` directory under its first argument. Both scripts take an
output directory as their last argument. Check format, loudness, edges, and
listen in context before acceptance.

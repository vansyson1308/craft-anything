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
python sand3d/finish_cage_two.py work/cage.glb scans work/cage_result --high work/cage_high.glb
python sand3d/long_cho.py work/cage_lod.glb --lod
python sand3d/long_cho.py work/cage_lod_high.glb --lod --high-poly
python sand3d/finish_cage_two.py work/cage_lod.glb scans work/cage_lod_result --texture-size 256 --high work/cage_lod_high.glb
```

The finish pass packs two UV atlases, bakes AO and selected-to-active normals,
combines the normal bake with the scan, and joins static and moving parts by
material. The hinged door remains separate. Review the result in Blender and
the real game, run Khronos glTF Validator, and update the manifest and reports
before replacing an asset. The scripts do not guarantee artistic acceptance.

## Flashlight

Using the same environment and scan directory:

```sh
python sand3d/den_pin.py work/flash.glb
python sand3d/den_pin.py work/flash_high.glb --high-poly
python sand3d/finish_flashlight.py work/flash.glb work/flash_high.glb scans work/flash_result
python sand3d/den_pin.py work/flash_lod.glb --lod
python sand3d/den_pin.py work/flash_lod_high.glb --lod --high-poly
python sand3d/finish_flashlight.py work/flash_lod.glb work/flash_lod_high.glb scans work/flash_lod_result --texture-size 256
```

The metal UV atlas, high-to-low normal and AO are baked; the lens has its own
emissive `lamp_glass` material. Verify `hand_r` placement in-game before
acceptance.

## Audio

`audio/ui_sounds.py` synthesizes the two UI sounds with NumPy and FFmpeg.
`audio/cc0_foley.py` converts the dog and iron-door recordings listed in
`assets/hq/SOURCES.md`. It expects `iron_door.ogg` and the extracted
`dog-source/Dog/` directory under its first argument. Both scripts take an
output directory as their last argument. Check format, loudness, edges, and
listen in context before acceptance.

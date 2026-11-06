# Draft Hàng Mã lanterns

The sources construct three original lanterns: red, yellow, and red diamond.
They require Python 3.11 with `sand3d==1.6.2` and its Blender 4.5 `bpy`,
NumPy, Pillow and `rectpack` dependencies. Obtain the CC0 Poly Haven `crepe_satin` 1K
diffuse JPG, OpenGL normal PNG and ARM PNG listed in `assets/hq/SOURCES.md`.
Put them in `scans/` with the names recorded there.

For each variant (`1`, `2`, `3`), the following example generates the near
model and its matched high-poly normal source. Repeat with `--lod` for both
geometry commands and `--size 256` for the far model's finish command.

```sh
python lantern_geometry.py work/lantern.glb --variant 1
python lantern_geometry.py work/lantern_high.glb --variant 1 --high-poly
python finish_lantern.py work/lantern.glb work/out --variant 1 --high work/lantern_high.glb --scans scans
```

The opaque body atlas combines satin colour and normal detail with original
frame/tassel regions and a silk-only emissive mask; the inner `lamp_glass`
bulb is a second material. The body deliberately renders opaque so its bamboo
frame remains visible in WebGL; emission approximates light through the thin
shell. The finish pass bakes AO and selected-to-active geometry normals. This
is an approximation and needs artistic review and testing in its actual hung
placement. Its lighting is not switched off during the day by these assets.
LOD0 body UV coverage measures roughly 44%, 44% and 33% for variants 01–03.
Some sampled edges have less than the brief's 4 px/cm target, so UV density
and packing need further work before accepting the models as final.

To reproduce the validator reports, install `gltf-validator` for Node and run
`node check_lanterns.cjs ../../assets/hq` from this directory. Render review
images with `render_lantern.py`; the `--dark` option sets world illumination
to zero. Do not treat either render as in-game acceptance.

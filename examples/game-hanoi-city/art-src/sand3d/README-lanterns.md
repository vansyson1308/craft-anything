# Draft Hàng Mã lanterns

The sources construct three original lanterns: red, yellow, and red diamond.
They require Python 3.11 with `sand3d==1.6.2` and its Blender 4.5 `bpy`,
NumPy, Pillow and `rectpack` dependencies. Obtain the CC0 Poly Haven `crepe_satin` 1K
diffuse JPG, OpenGL normal PNG and ARM PNG listed in `assets/hq/SOURCES.md`.
Put them in `scans/` with the names recorded there.

For each variant (`1`, `2`, `3`), the following example generates the near
model and its matched high-poly normal source. Repeat with `--lod` for both
geometry commands and `--size 512 --density 256` for the far model's finish
command. The near model defaults to a 1024 px atlas and 512 px/m.

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
The body UV islands are scaled to 512 px/m for LOD0 and packed with an 8 px
border; LOD1 uses 256 px/m and an 8 px border. Measurements for the exported
files are in `assets/hq/reports/lantern-uv-density.json`. At the requested
density the summed island area occupies only about 19.2%, 18.2% and 14.9% of
the LOD0 1024 px atlas. This leaves the earlier atlas-utilization target
unmet and should be reviewed against the requested density and padding.
Seam placement still needs close visual inspection.

To reproduce the validator reports, install `gltf-validator` for Node and run
`node check_lanterns.cjs ../../assets/hq` from this directory. Render review
images with `render_lantern.py`; the `--dark` option sets world illumination
to zero. Do not treat either render as in-game acceptance.

# Lakeside lamp — early study, not game-ready

This is a single-globe cast-iron lamp study based on the brief, not a surveyed
reproduction of a particular Hoàn Kiếm lamp. The two GLBs are deliberately
kept here and are not registered in `assets/hq/manifest.json`. No gameplay
placement or lighting behavior is changed by this draft.

The lamp is 3.8 m tall with its origin centred at ground level and the globe
centred at 3.5 m, matching the intended in-game light position. LOD0 and LOD1
are 1,718 and 446 triangles, respectively, within the 2,000/500 prop budgets.
Each has a separate `lamp_glass` emissive material and a dark metal material;
glTF Validator reports zero errors and warnings. Metal maps are 1024 and 512
pixels respectively. `lamp.png` and `lamp_lod1.png` show the two models in
Cycles; `lamp_dark.png` shows the LOD0 globe with world and review lights off.

The metal uses the CC0 [rusty_metal_02](https://polyhaven.com/a/rusty_metal_02)
scan by Rob Tuytel, with the diffuse darkened and desaturated. The scan maps
are reused as surface detail; they are not model-specific bakes.

Outstanding: validate the silhouette and details against suitable reference
photos, improve the base and surface wear, unwrap and measure UV coverage and
seams, bake high-to-low normals and object-specific AO, examine LOD switching,
pack and inspect the asset in the actual game by day and night. The raw GLB's
globe emits continuously: before integration the game must also switch this
material off by day. The screenshot is a Blender study, not proof of the
final streetscape or its lighting.

To reproduce, use Blender 4.5 Python with sand3d 1.6.2 and Pillow, placing the
`rusty_metal_02_diff_1k.jpg`, `rusty_metal_02_nor_gl_1k.png`, and
`rusty_metal_02_arm_1k.png` files in a scan directory:

```text
python art-src/sand3d/den_ven_ho.py <lod0.glb> --scans <scan-directory>
python art-src/sand3d/den_ven_ho.py <lod1.glb> --scans <scan-directory> --lod
python art-src/sand3d/render_lake_lamp.py <lod0.glb> <preview.png>
python art-src/sand3d/render_lake_lamp.py <lod0.glb> <dark.png> --dark
```

# Ô Quan Chưởng — early blockout, not game-ready

This is an early shape study, **not a finished recreation**. The three GLBs are
kept under `review/10` and are deliberately absent from `assets/hq/manifest.json`,
so they cannot replace the game landmark until the outstanding work is done.

The blockout has a central arch and two smaller arches, brick walls, parapets,
and a four-sided tiled pavilion roof. The base is made with sand3d; the arch
cutters and roof surface use custom Blender meshes. The entrance sizes are
approximate: 3 m wide/high in the centre and 1.65 m wide / 2.5 m high at the
sides. The footprint is oriented to the named street in the game map. Its
measured axis-aligned world bounds are about 11.26 × 17.96 m, with a height of
8.64 m; this is an interpretation of the map footprint, not a site survey.

`gate_front.png` and `gate_diagonal.png` are Cycles views. All three LOD GLBs
pass glTF Validator with no errors or warnings, at 1,400 / 800 / 440 triangles.
Eight geometry ray checks per LOD verify that the openings are clear at sample
heights and that a wall sample is solid. This does not prove the installed game
scene, player navigation, appearance or landmarks are correct.

The companion navigation fix in this draft carves only the central passage
through the named gate's map polygon, and only if the named street passes close
to it. The real-map test exercises the player's collision circle along the
street and checks that both side walls remain blocked. The 32 core and four
real-city tests pass. The 2 m raster is an approximation; the game still needs
an actual playthrough of the final installed model.

**Outstanding:** the silhouette and proportions need verification against
the actual monument; the roof is only a simple hipped shape, not sculpted
curved tiles; plaster, brick variation, moss, stains, parapet motifs, the
inscription and other details are missing. The 1K scans are reused on every
LOD rather than reduced for distance. The UVs use projection, no high-to-low
normal or model-specific AO has been baked, and seams, texel density, texture
budgets, game placement and visual approval are unverified. The blank plaque
must not be mistaken for a finished inscription.

Shape references: [Vietnam Tourism](https://www.vietnamtourism.com/en/places/hanoi/o-quan-chuong-gate)
and [Hoàn Kiếm heritage portal](https://hoankiem360.tourism.vn/chi_tiet_di_tich_lich_su/cong-o-quan-chuong.html).
Scanned textures: [Medieval Red Brick](https://polyhaven.com/a/medieval_red_brick)
by Rob Tuytel and [Roof Tiles](https://polyhaven.com/a/roof_tiles) by Stephan
Seeliger, both CC0. The source PNG data maps are converted to ordinary 8-bit
RGB without embedded ICC profiles for glTF compatibility.

To reproduce, download each scan's 1K Diffuse JPG, OpenGL normal PNG and ARM
PNG as `<asset>_Diffuse.jpg`, `<asset>_nor_gl.png`, and `<asset>_arm.png`, then:

```text
python art-src/sand3d/o_quan_chuong.py <output.glb> --scans <scan-directory>
python art-src/sand3d/o_quan_chuong.py <output_lod1.glb> --scans <scan-directory> --lod
python art-src/sand3d/o_quan_chuong.py <output_lod2.glb> --scans <scan-directory> --lod2
python art-src/sand3d/check_o_quan_chuong.py <output.glb>
```

Use Blender 4.5's Python environment with sand3d 1.6.2 and Pillow.

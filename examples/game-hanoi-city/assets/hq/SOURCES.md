# Sources and licence

The delivered assets and original modifications are released under CC0-1.0 for public use, redistribution, modification and inclusion in open-source projects. No assets from other games or brand marks are included. Human geometry is synthetic.

## Scan materials

Poly Haven assets are CC0: https://polyhaven.com/license . Each texture set was downloaded at 2K; maps are resampled for individual LOD budgets.

- [granite_tile](https://polyhaven.com/a/granite_tile): diffuse, OpenGL normal and ARM; sidewalk uses the full scan, curb uses a cropped stone interior.
- [denim_fabric](https://polyhaven.com/a/denim_fabric): diffuse, OpenGL normal, ARM and displacement.
- [cotton_jersey](https://polyhaven.com/a/cotton_jersey): diffuse, OpenGL normal, ARM and displacement.
- [tree_bark_03](https://polyhaven.com/a/tree_bark_03): diffuse, OpenGL normal, ARM and displacement.
- [wood_planks_dirt](https://polyhaven.com/a/wood_planks_dirt): diffuse, OpenGL normal, ARM and displacement.
- [aerial_asphalt_01](https://polyhaven.com/a/aerial_asphalt_01): diffuse, OpenGL normal, ARM and displacement.
- [aerial_grass_rock](https://polyhaven.com/a/aerial_grass_rock): diffuse, OpenGL normal, ARM and displacement.
- [square_floor_patern_01](https://polyhaven.com/a/square_floor_patern_01): diffuse, OpenGL normal, ARM and displacement.
- [concrete_wall_006](https://polyhaven.com/a/concrete_wall_006): diffuse, OpenGL normal, ARM and displacement.
- [red_brick_03](https://polyhaven.com/a/red_brick_03): diffuse, OpenGL normal, ARM and displacement.
- [brown_leather](https://polyhaven.com/a/brown_leather): diffuse, OpenGL normal, ARM and displacement.
- [roof_tiles_14](https://polyhaven.com/a/roof_tiles_14): diffuse, OpenGL normal, ARM and displacement.

## Characters and source rig

- MakeHuman Community base mesh, synthetic Asian young male macro shapes: CC0 core assets, commit a8bc2d54ff0ac92e78ff71431b1023eda42bf482. https://github.com/makehumancommunity/makehuman
- CC0 system asset pack: male_casualsuit01, male_casualsuit05, short01 hair, young Asian male skin colour maps. Original clothing diffuse maps were not used; scanned fabric replaces them. https://static.makehumancommunity.org/assets/assetpacks/makehuman_system_assets.html
- Source pack: https://files.makehumancommunity.org/asset_packs/makehuman_system_assets/makehuman_system_assets_cc0.zip . Individual asset headers credit the MakeHuman contributors, including Data Collection AB, Joel Palmius and Jonas Hauquier.
- Exact game rig: ../people/minh/person.gltf and ../people/shared/person.bin, sourced from Quaternius Universal Base Characters, CC0. https://quaternius.com/packs/universalbasecharacters.html
- Compatibility animation library: ../anim.glb, Quaternius Universal Animation Library, CC0. https://quaternius.com/packs/universalanimationlibrary.html . These clips are not duplicated in the delivered human models.

## Original construction and references

- Dog, dog rig and five 30 fps clips, bridge, trees, foliage atlas, facades, landmarks, props, generic motorcycle and posed rider: original construction and adaptation in Blender/Python. Dog coat and water normal are original procedural textures.
- Art direction: the previously generated game-style Hanoi image supplied in this chat; it is a generated concept, not a surveyed photograph. Inadvertent brands in that concept are not reproduced.
- Architectural reference only, not texture sources: Hanoi Department of Culture and Sports, https://sovhtt.hanoi.gov.vn/den-ngoc-son/ ; Vietnam National Authority of Tourism, https://vietnamtourism.gov.vn/en/post/13090 . Buildings are reconstructions, not measured scans.
- Scan checksums are recorded in reports/source-texture-hashes.json. Source textures and Blender working files are retained outside the repository to avoid duplicate embedded data.

## Tools

Blender 4.5.3 LTS: metre-scale A-pose Cloth settling, fitted garment thickness, Multires detail meshes, Cycles selected-to-active cage baking and static AO baking; NumPy/Pillow for packed image data and LOD resampling; Khronos glTF Validator; the actual Bevy WebGL game for review captures.

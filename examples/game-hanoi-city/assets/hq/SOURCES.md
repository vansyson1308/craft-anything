# Sources and licence

The delivered assets and original modifications are released under CC0-1.0 for public use, redistribution, modification and inclusion in open-source projects. No assets from other games or brand marks are included. Human geometry is synthetic.

## Scan materials

Poly Haven assets are CC0: https://polyhaven.com/license . Each texture set was downloaded at 2K; maps are resampled for individual LOD budgets.

- [crepe_satin](https://polyhaven.com/a/crepe_satin), Rico Cilliers / colormass, CC0: 1K diffuse, OpenGL normal and ARM scans recoloured and combined with original UV detail for the three draft Hàng Mã lanterns. For reproducibility, the local downloaded files are named `crepe_satin_diff.jpg`, `crepe_satin_nor_gl.png`, and `crepe_satin_arm.png`. Their SHA-256 values, respectively: `403b5af633cb194117bcd63b3158497507290ee3fd7b7092fb5638d05426bb86`, `89f25b39bbf985828b874115c2612d5ed328a6af799f6123bcab5525c4e06a02`, and `6b46e4c6fdb731ca527f3a92bbee7ed319cb9511c85aaf3aad5d54d180d20985`.

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

- [old_linoleum_flooring_01](https://polyhaven.com/a/old_linoleum_flooring_01): CC0 diffuse scan, high-pass processed to make shared worn-plastic colour, normal and roughness detail for the draft stools and table. The original linoleum tile pattern is not intended as the plastic surface. AO is not yet baked.
- [rusty_metal_02](https://polyhaven.com/a/rusty_metal_02), by Rob Tuytel, CC0: diffuse, OpenGL normal and ARM scans, resampled for the draft cage. The cage normal map combines scan detail with a selected-to-active high-poly bake; AO is baked into its atlas. The revised flashlight uses original procedural textures rather than this scan.

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

## Original interface audio

- `audio/ui_bam.ogg` and `audio/ui_meo.ogg`: original synthesized tones and seeded noise, authored by Codex for this project without recorded samples, released under CC0-1.0. The reproducible generator is `../../art-src/audio/ui_sounds.py`; FFmpeg converts and normalizes the output to Ogg Vorbis. Listening and in-game reviews remain pending.

## CC0 sound recordings

- `audio/cho_sua_01.ogg`, `audio/cho_sua_02.ogg`, `audio/cho_sua_03.ogg`, and `audio/cho_rit.ogg`: converted respectively from `Dog Bark 1.wav`, `Dog Bark 2.wav`, `Dog Bark 3.wav`, and `Sad Dog 1.wav` in [Dog sounds by pauliuw](https://opengameart.org/content/dog-sounds), CC0. The source pack notes that some sounds were recorded through an MP3 player. Barks use transient compression; all have short edge fades and loudness normalization.
- `audio/long_mo.ogg`: converted from [Iron Door by themightyglider](https://opengameart.org/content/iron-door), CC0, itself derived from [open the iron door.wav by forseti1121](https://freesound.org/people/forseti1121/sounds/319324/), also CC0. Converted with edge fades and loudness normalization.
- Reproduce conversions with `../../art-src/audio/cc0_foley.py` and the linked source files. These sounds have not yet had listening or in-game review.

### QC bark experiments, not yet delivered to the game

The five files under `../../review/05/audio/` are review candidates only;
they do not replace the rejected production sounds. Candidates 01–03 are
from [Old dog barking #1](https://bigsoundbank.com/old-dog-barking-1-s2352.html),
and 04–05 from [Old dog barking #2](https://bigsoundbank.com/old-dog-barking-2-s2353.html),
both by Joseph Sardin, CC0, recorded with a SoundDevices MixPre-3 and
Sennheiser ME66, 48 kHz/24-bit WAV. The recordings are not Vietnamese dogs;
similarity and expressive suitability need listening review. Reproduce with
`../../art-src/audio/qc_bark_candidates.py`, naming the downloads
`source_2352.wav` and `source_2353.wav` in the supplied source directory.

| Candidate | FFmpeg astats noise floor (dBFS) | Highest FFT bin ≥ −90 dBFS (Hz) | Peak FFT bin in 8–12 kHz (dBFS) | Decoded true peak (dBTP) |
|---|---:|---:|---:|---:|
| cho_sua_01 | −53.3 | 13011 | −80.2 | −0.93 |
| cho_sua_02 | −62.6 | 15166 | −75.8 | −0.81 |
| cho_sua_03 | −64.5 | 12878 | −81.0 | −1.05 |
| cho_sua_04 | −42.5 | 8918 | −74.8 | −0.91 |
| cho_sua_05 | −59.4 | 12193 | −75.1 | −1.08 |

FFT measurements are a windowed per-bin magnitude, not proof that all high
frequencies contain useful bark rather than noise. Candidate 04 does not meet
the requested noise-floor threshold. Short-event astats noise floors can depend heavily on
the selected decay and are not a substitute for listening to isolated noise.
True peak is an approximately 4× oversampled decoded measurement; several
candidates still need closer normalization. The required long, distant,
growling and happy variants, seamless whimper, cleaned cage opening, and
listening/game review remain unfinished.

`../../review/05/audio/cho_rit_candidate.ogg` is an unapproved tentative loop
from [Dogs barking and crying #1](https://bigsoundbank.com/dogs-barking-and-crying-1-s1544.html),
Joseph Sardin, CC0, MixPre-3/ME66, 48 kHz/24-bit WAV. A 200 ms overlap is
applied and the loop is rotated to a zero crossing; the decoded end/start
sample difference is about 0.00005. FFmpeg astats reports −50.5 dBFS noise
floor and FFmpeg ebur128 reports approximately −1.2 dBTP. The highest FFT
bin above −90 dBFS is 7.85 kHz, using the same measurement definition as the
table. These numerical checks do not prove the loop is click-free or that the
chosen region is a usable whimper; listening and game review are required.
An independent YAMNet audio-event classifier ranked bird calls above dog
whimper for this candidate; it is therefore rejected pending a better source.
The optional diagnostic script is `../../art-src/audio/qc_audio_classifier.py`;
the [YAMNet ONNX model](https://huggingface.co/audiomagic/yamnet-onnx) is an
Apache-2.0 conversion of Google's AudioSet classifier. Its labels are uncertain
model predictions and cannot replace a listening test.
The experimental generator is `../../art-src/audio/qc_whimper_candidate.py`.

`../../review/05/audio/long_mo_candidate.ogg` is an unapproved experiment
from [Grinding metal gate](https://bigsoundbank.com/grinding-metal-gate-s0683.html),
Joseph Sardin, CC0, 48 kHz/16-bit WAV. The script learns a profile from
4.0–4.4 s of that recording, extracts 0.45–1.05 s, applies 12 dB reduction,
and is `../../art-src/audio/qc_gate_candidate.py`. FFmpeg astats reports
−24.2 dBFS noise floor, the decoded 4× estimate is −1.14 dBTP and the highest
FFT bin above −90 dBFS is 18.1 kHz. The short continuous creak can affect
the astats noise estimate, but this result does not meet the requested
numerical noise criterion and needs listening and further work.
YAMNet also ranked telephone/ringtone above gate sounds for this candidate;
this is not conclusive, but reinforces that it should not be delivered yet.

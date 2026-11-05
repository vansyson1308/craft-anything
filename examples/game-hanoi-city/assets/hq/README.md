# Hanoi City assets: revision 02 review candidate

This replaces the five batch 01 models and adds the requested batch 02 inventory: 28 base models, 66 GLBs including LODs, five ground material sets (13 PNG maps), and one tree billboard. **This is a draft art submission, not an accepted photorealistic asset pack.**

The manifest contains measured bounds, triangle counts, LOD paths, skeleton/animation metadata, tools, licence and limitations. Existing asset names are preserved. Human files use the exact original 65-joint rig and the game's external 43-animation library. Dog files contain Idle, Walk, Run, Sit and Bark. Source materials and editable Blender files are retained outside the repository.

## Batch 03 interim night fix

The shared joinery material in the street lamp and the eight facade modules (including LOD1) has its emissive factor set to zero. The original textures and geometry are unchanged. This stops the metal from emitting, but it also leaves these models without emissive lamp glass or lit windows until the dedicated materials are authored. The existing review images predate this change; it has not yet been checked in Blender or in the game at night. The remainder of batch 03 is not delivered here.

## Review and integration

Open [REVIEW.md](REVIEW.md) for side-by-side Cycles and actual Bevy images of all 33 model/material items. All review images are JPEG, at most 1280 px. Four full-manifest review cameras are included. The full lineup view is obstructed by the new gate/nearby facade placement, so a separately labelled isolated lineup is also included. Individual model views use the actual Bevy game with only the requested HQ asset enabled and fallback surroundings. Material views use temporary 2 m swatches, not a claim that every ground placement was visually inspected.

Reviewed with the game integration from PR #5 at b3e65f3 and the local review-only patch in reports/bevy-review-harness.patch. Apply the zero-context patch with `git apply --unidiff-zero`. That patch adds an isolated-asset camera and six animation keys; it is evidence, not a production game-code change. Keep the existing game loader/packer from that integration. The delivered source GLBs remain self-contained; do not load packed web GLBs without their external texture folder.

Large architecture uses repeating base colour, OpenGL normal and metallic/roughness on UV0, with baked occlusion on UV1. Small props use packed AO/roughness/metallic on UV1. Human atlases use packed ORM. All files stay within two UV channels. LOD texture maximum edges are 1024 / 512 / 256 for models; ground maps are 2048. Simple props already below the LOD1 triangle budget may retain topology while reducing texture size. Tree billboard placement metadata is in manifest.json.

The current web packer skips previews/reports but does not skip review/. Exclude review/ and REVIEW.md from deployment; they are review evidence, not runtime assets. A size measurement excluding those files is in reports/package-size.json.

## Measured verification

- 66 binary glTF 2.0 files: Khronos Validator reports zero errors and zero warnings.
- Triangle/material budgets, power-of-two textures, required PBR channels, maximum two UV channels, skin-weight normalization and identity scene-root rotation/scale passed. File hashes are in reports/spec-checks.json.
- All 15 human GLBs retain original joint order, hierarchy, rest transforms and inverse-bind matrices. No replacement human animations are embedded.
- CPU skin evaluation sampled five times in each of 43 clips for five base human meshes, plus five dog clips: 220 clip/model combinations with finite results and resolved targets. This is compatibility sampling, not a clothing collision test or exhaustive motion review.
- Bevy WebGL loaded and captured all 28 base models, five material swatches and five representative LODs (the isolated bridge LOD view has limited framing). Six human motion states were inspected: Idle_Loop, Sprint_Loop, Crouch_Fwd_Loop, Punch_Cross, Roll and Death01. Review captures do not prove every frame is free from intersections.
- Facade horizontal bounds fit 0..4 m, projection is <=0.6 m; cage has a node named door and the sign has material sign_text. LOD image caps and review-file existence are checked in reports/delivery-checks.json.
- Browser: headless Chromium, ANGLE OpenGL, 1100 x 700 viewport, actual Bevy WebGL WASM. Canvas drawing buffers were preserved for capture. No WebGPU, physical phone, frame-rate or memory-budget certification was performed.

## Unmet requirements / next art pass

- **Visual target remains unmet.** The generated game-style Hanoi reference is more natural than this reconstruction. Clothing has close-up shading/shape defects and visible dark breaks during the sampled Roll pose; dog anatomy and coat remain stylized; repeated tree silhouettes and simplified buildings remain apparent.
- **UV packing remains below 70%.** Base human material coverage estimates range from 18.38% to 60.93%; dog atlases are 39.83% and 51.14%. Values are raster estimates at 512 px. Hair alpha adds intentional empty space, but that does not waive the requested threshold.
- **Clothing texel density remains insufficient.** Most garments average roughly 3.32-5.29 px/cm at 1024; dog fur averages 7.56 px/cm. Face density has not been demonstrated to be twice body density. See reports/uv-measurements.json. Repacking alone will not resolve all density targets within the present material and image caps.
- Thief and three pedestrians share the fitted young male base; pedestrian variants change material and face width rather than providing three independently sculpted people. Thief body differentiation is not fully achieved. Windbreaker and sneaker detail need further art work.
- Dog coat is authored procedural detail, not scanned fur. The motorcycle rider is a static pose derived from Driving_Loop; grip contact, helmet and close-up anatomy need refinement.
- Landmark geometry is an approximate artist reconstruction, not an exact surveyed model. The gate cluster obstructs the current full-manifest lineup camera. Mature trees crowd the island under current game placement; footprint, placement density and variation need review alongside asset refinement.
- AO/normal baking and finite animation sampling do not establish final artistic acceptance. Keep this submission in draft until the missing art and placement work is resolved.

## Licence and provenance

CC0-1.0 for original work and modifications, with CC0 MakeHuman, Quaternius and Poly Haven inputs. See [SOURCES.md](SOURCES.md), [LICENSE.txt](LICENSE.txt) and source texture hashes. No other-game assets, real-person likenesses or intentional brand marks are included. Blank signs are left for the game to label.

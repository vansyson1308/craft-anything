# Dog Wag and Jump — motion study

This is an early motion study on the **existing** chó cỏ mesh and 26-bone rig.
`dog_motion_study.glb` is kept in review and is not registered in the manifest
or used by the game. It is not a replacement for the requested anatomy, fur or
high-poly work. The game has not yet been wired to play these clips.

`Wag` is a one-second, 30-fps loop driven by five tail bones. `Jump` is a
one-second, 30-fps tentative rear-up with the front legs reaching forward;
its root returns to its initial transform. The contact sheet shows selected
poses, not continuous playback. No player is present, so the paws' contact
with Minh's legs, timing, collision and emotional effect are not verified.

The GLB retains the original mesh, skin, node and material definitions and
the five original animation definitions. It has 7,586 triangles, seven clips
and passes glTF Validator with zero errors and warnings. Both new clips return
to their initial values within 0.00001. For `Jump`, a Blender floor correction
was applied per frame; at 61 half-frame samples the lowest skinned vertex was
approximately 0.001–0.010 m above the floor. This does not prove that each paw
is planted, that no other mesh intersections occur or that the animation looks
natural in motion. A proper animation review and gameplay interaction remain
necessary.

The separate [Grab3D dog](https://grab3d.com/models/animals/dog/) was examined
as a possible anatomical starting point and was **not used** here. Its
[license](https://grab3d.com/dl/dog/LICENSE.txt) identifies it as CC0 and
AI-assisted, but its drooping ears, tail and unrigged static pose do not match
the brief. No file from that source is included in this study.

Reproduce with Blender 4.5 Python. Run from `examples/game-hanoi-city`:

```text
python art-src/animation/study_dog_motion.py assets/hq/animals/cho_co.glb <uncorrected.glb>
python art-src/animation/ground_dog_motion.py <uncorrected.glb> <dog_motion_study.glb>
python art-src/animation/check_dog_motion.py assets/hq/animals/cho_co.glb <dog_motion_study.glb>
python art-src/animation/render_dog_motion.py <dog_motion_study.glb> <poses-directory>
```

Run glTF Validator separately after regeneration. Floor correction is based
on the lowest point of the evaluated mesh and does not solve a foot-placement
or player-contact constraint.

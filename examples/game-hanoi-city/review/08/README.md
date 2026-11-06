# Cage weathering and tie — draft review

These images show an optional revision of the cage already in the game. The
four `cage_qc_*.png` images are Cycles views of the delivered LOD0 GLB; the
door is rotated to 110° in `cage_qc_open_110.png`. `cage_rope_detail.png` is
a close view of the two-turn wrap around the right door rail and frame.

The original uniformly orange scan is converted to muted metal while retaining
its surface variation. Procedural 3–6 cm downward streaks are placed below
rail intersections, with additional staining on the lower rails and tray.
AO and a normal-gradient approximation modulate the mask; this is not a full
concave-curvature bake. The appearance needs artistic review in the game.

The tie is attached to the moving door and follows it when the door opens.
There is no separate rope-release animation: it changes from a closed loop
around both rails to a loop remaining on the opened door. Decide whether the
rescue needs an explicit untie or cut interaction before accepting the revision.

The `game_day.png`, `game_night.png` and `game_open.png` screenshots use a
fresh WebGL build of this branch and the game's `look=cage` camera. The
revised raw cage GLBs were served in place of the packaged cage; other game
assets and the manifest came from an existing local web build. The cage GLB
loaded successfully, and the game visibly rotates the door in the open view.
The review camera crops part of the opened door, while the night flashlight
overexposes portions of the bars. These captures do not establish LOD
transitions, packaging behavior or final artistic approval.

Both models retain a single material and the named `door_hinge` revolute joint
with its 0–110° range. The measured LOD0/LOD1 triangle counts are 1,780/474,
within the 2,000/500 budgets. glTF Validator reports no errors or warnings.
The joint pose was also inspected in the Cycles render. Seam and LOD-transition
checks, a full view of the open door in the game, and final visual approval
remain outstanding.

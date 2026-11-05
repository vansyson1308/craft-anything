# Batch 05 review - incomplete

QC revision: `cage_qc_front.png`, `cage_qc_back.png`, `cage_qc_diagonal.png`
and `cage_qc_open_110.png` are Cycles renders of the revised single-atlas cage.
`den_pin_qc_blender.png` and `den_pin_qc_dark.png` show the revised flashlight
in Cycles with normal and zero world lighting respectively. These do not
replace the required close-up game captures or establish art acceptance.

The files in `audio/` are unapproved experiments with spectrum plots, not
replacements for the game audio. The tentative `cho_rit_candidate` and
`long_mo_candidate` are rejected: an audio classifier ranked them as bird and
telephone sounds respectively, and the gate candidate fails the stated noise
floor limit. Their sources and measured limitations are documented in
`assets/hq/SOURCES.md`.

These are partial game captures, not art acceptance. The earlier captures below were captured from
the local WebGL build in headless Chrome, using ANGLE OpenGL at 640 × 480 and
reading the preserved WebGL framebuffer. The initial screenshot API returned
black or white images despite a rendered test canvas, so the framebuffer was
read directly instead.

The newer `cage_qc_game_day.png` and `cage_qc_game_night.png` use a temporary
asset-review camera in the real WebGL game, 1.5 m horizontally from the cage
origin at 1.5 m eye height. At that test position the curb intersects the cage
tray. The `*_lift.png` images repeat the test after lifting only the temporary
packed copy by 0.15 m; the delivered GLB is unmodified. This clears the curb
but is an artificial placement. Both night views are too dark for art review.
The reproducible temporary source patch is
`assets/hq/reports/qc-asset-review-harness.patch` and is not applied to game
source. All four are raw framebuffer captures at 1000 × 750.

`cage_qc_game_day_lamp.png` and `cage_qc_game_night_lamp.png` instead use
`assets/hq/reports/qc-lamp-review-harness.patch` to place the review instance
below an existing game street light, with its origin raised 0.15 m. The camera
is 1.5 m horizontally away and at 1.5 m eye height. The night frame is lit by
the game's existing lamp; the day frame is heavily shaded by the surrounding
scene. The earlier day-lift frame provides a clearer day view. Neither camera
patch is part of the shipped game code.

- `lineup_day.png`: full-manifest `?gl&look=lineup&time=day`; the existing
  gate/facade obstructs the background. The cage is not in this view.
- `cage_day_partial.png`: chapter one started with `?gl&time=day#peek`, then
  moved left. The game loaded the new `people/long_cho.glb` with HTTP 200;
  only part of the cage is visible at the left edge.
- `cage_day.png`: another frame from the same approach; the full cage appears
  at the left, behind the objective beam. Distance limits detail inspection.
- `cage_night.png`: chapter one started with `?gl&time=night#peek`, then moved
  left. The new GLB loaded with HTTP 200 and the whole cage is visible at the
  right, though the objective beam and distance limit detail inspection.
- `den_pin_blender.png` and `den_pin_world_zero.png`: separate Cycles renders
  of the flashlight, including a world-strength-zero emission check.
- `den_pin_night_context.png`: the game requested the flashlight GLB with HTTP
  200 during chapter one. The hand is too small and dark in this frame to
  verify grip alignment or the direction of the light visually.

The `streetlife` and `bridge` test frames and the lineup-night frame were
black or almost black in this setup. With an empty manifest, `streetlife`
produced a visible image. Removing only the newly added cage and audio entries
from the full manifest did not change the dark `streetlife` frame. This does
not identify the cause or establish behavior on other devices.

Still required: art acceptance, the streetlife
view, actual game-side opening and audio
integration, audio listening, UV seam/LOD visual inspection, and the rest of
the requested asset batch. Do not treat the screenshots as final approval.

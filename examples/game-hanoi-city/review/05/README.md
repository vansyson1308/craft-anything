# Batch 05 review — incomplete

These are partial game captures, not art acceptance. They were captured from
the local WebGL build in headless Chrome, using ANGLE OpenGL at 640 × 480 and
reading the preserved WebGL framebuffer. The initial screenshot API returned
black or white images despite a rendered test canvas, so the framebuffer was
read directly instead.

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

Still required: a clear close-up of the cage in the real game, the streetlife
view, actual game-side opening and audio
integration, audio listening, UV seam/LOD visual inspection, and the rest of
the requested asset batch. Do not treat the screenshots as final approval.

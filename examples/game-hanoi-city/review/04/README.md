# Batch 04: draft plastic furniture review

These screenshots show the three draft plastic furniture models placed by the actual Bevy WebGL game. The scene used the unmerged game integration from PR #10 at commit `128c875`; `streetlife-review-camera.patch` adds only a fixed review camera that looks at a naturally placed table group. The patch is not part of the production game. The packer used the draft GLBs and manifest from this branch.

- `ban-ghe-ngay.jpg`: daytime, a single shopfront at approximately eye height. The table and stools are visible, but this is not a review of several streets or proof of final art quality.
- `ban-ghe-dem.jpg`: the same approach at night. The scene is too dark to assess the furniture; night lighting and a useful night review remain open.

The browser requested all six GLBs successfully. Khronos validator reports for them are in `assets/hq/reports/`. The stools' estimated UV coverage is approximately 67.4% and the table's is 76.3% (rasterized at 2048 px); the stool falls short of the requested 70%. Their common ORM has a white, unbaked occlusion channel. High-poly baking and art refinement remain open. These models are review candidates, not finished deliveries. The requested four parked motorcycles are not included.

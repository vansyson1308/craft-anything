# Hàng Mã lantern review — draft

`den_long_01_cycles.png`, `den_long_02_cycles.png` and
`den_long_03_cycles.png` are Cycles views of the actual LOD0 files. The
`world_zero` frame tests emission with no world or review light. The original
translucent body had a WebGL sorting problem that hid the bamboo ribs; these
files instead use an opaque shell and a silk-only emissive mask. This is a
visual approximation rather than physical transmission.

The two `den_long_01_game_*.png` images use the actual WebGL game with the
temporary `assets/hq/reports/qc-lantern-review-harness.patch`. It places the
prop on the street beside an existing lamp for inspection, rather than in its
eventual hanging position. The camera is 1.5 m above the ground and 1.5 m
horizontally from the prop. The daytime image is heavily shaded by the scene;
the night image uses the game's existing street light. Both are raw 1000 × 750
framebuffer captures. The harness is not applied to game source.

The real game does not yet place these props in the lantern strings. LOD0 body
UV coverage measures approximately 43.6%, 44.1% and 33.0% for variants 01–03;
some sampled edges are below the requested 4 px/cm. UV packing and density,
day/night views of variants 02 and 03, LOD transitions, close seam inspection,
actual hanging placement and final artistic approval remain outstanding.

Round 2 requests 8 px UV padding and consistent density around 512 px/m
within 15% between islands. The current packing uses 2 px padding and the
per-island density tolerance has not been verified; both remain outstanding.

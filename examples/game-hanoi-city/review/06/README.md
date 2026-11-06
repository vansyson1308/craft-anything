# Hàng Mã lantern review — draft

`den_long_01_cycles.png`, `den_long_02_cycles.png` and
`den_long_03_cycles.png` are Cycles views of the actual LOD0 files. The
`world_zero` frame tests emission with no world or review light. The original
translucent body had a WebGL sorting problem that hid the bamboo ribs; these
files instead use an opaque shell and a silk-only emissive mask. This is a
visual approximation rather than physical transmission.

The two `den_long_01_game_*.png` images show the previous UV version in the
actual WebGL game with the
temporary `assets/hq/reports/qc-lantern-review-harness.patch`. It places the
prop on the street beside an existing lamp for inspection, rather than in its
eventual hanging position. The camera is 1.5 m above the ground and 1.5 m
horizontally from the prop. The daytime image is heavily shaded by the scene;
the night image uses the game's existing street light. Both are raw 1000 × 750
framebuffer captures. The harness is not applied to game source.

The real game does not yet place these props in the lantern strings. The revised
LOD0 uses 1024 px atlases, 8 px island borders and 512 px/m density; all measured
body islands fall within the requested ±15% tolerance. LOD1 uses 512 px atlases
and 256 px/m. Summed UV island areas cover only about 19.2%, 18.2% and 14.9%
of the LOD0 atlases: the earlier 70% utilization goal remains unmet. See
`assets/hq/reports/lantern-uv-density.json` for the measurements.

Current in-game day/night captures, LOD transitions, close seam inspection,
actual hanging placement and final artistic approval remain outstanding.

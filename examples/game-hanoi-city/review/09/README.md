# Streetlight lens — draft review

The existing `den_duong_01` mesh used one emissive material for the metal pole,
arm, shade and lens. This draft preserves the existing geometry and separates
the thin lens into `lamp_glass`; `streetlight_metal` no longer emits. The lens
has a warm emissive factor. Both LODs are included.

`streetlight_before.png` and `streetlight_after.png` are comparable Cycles
close-ups. `streetlight_world_zero.png` renders the revised GLB with world and
review lights set to zero: the visible emission is limited to the lens. This
is a material check, not a game night-lighting test.

Both models pass glTF Validator with zero errors or warnings. Triangle counts
remain 960 and 476 (LOD0/LOD1), within the 2,000/500 street-prop budgets.
Each uses two materials and three image textures. Metal keeps its normal-map
tangents; the glass has no normal map and its degenerate tangents are omitted.

The four `upper_0*.png` images show the upper-storey facade variants with
their window panels separated as `window_glow` and their masonry and joinery
non-emissive. `upper_01_world_zero.png` demonstrates the result without world
or review lighting. An emissive texture lights one window in each variant and
shows a partially drawn curtain; the other window is dark. About 39.6% of the
texture pixels are bright (red > 100 and green > 40 on an 8-bit scale). Shutters
and perspective affect how much is visible in the scene, so this is not a
measurement of the fraction of visible panes. This is a simple pattern, and the final streetscape still
need visual review. All eight upper facade GLBs, including LOD1, pass glTF Validator with
zero errors and warnings; their triangle counts are unchanged and within budget.

The four `ground_0*.png` images show the ground-floor variants: open shops
have a separate emissive `shop_light` strip; the closed shutter has no strip.
`ground_01_world_zero.png` shows the strip with no world or review lighting.
All eight ground-floor GLBs pass glTF Validator without errors or warnings,
and their mesh bounds extend no farther than 0.59 m in the outward +Z direction.

The game now has a draft system intended to switch the streetlight lens, upper
windows and shop strips off during the day and restore their source emission at
night, including when a new LOD loads. It scopes this behavior to these asset
paths so the player's flashlight is unaffected. The complete 32-test Rust suite
passes, including tests for day/night changes and a late-loaded LOD.

The six `webgl_*_final.png` screenshots exercise the revised models through
the game's LOD and day/night material path in a fresh WebGL build. They use the
temporary `material-review-harness.patch` on a build from before PR #25: it places one model and the camera
100 m above the city to avoid obstructions. There is no local point light on the
test prop, so its non-emissive surfaces are very dark at night. Raw GLBs were
served for these captures; separate packing checks passed. This tests selected
models and is not a review of the final streetscape or every LOD transition.
The upper-window pair was refreshed after adding the emission texture; the
streetlight and shop images predate that texture-only change.

The closed shutter retains an unused original emissive image in its GLB, but
no material references it. Final streetscape day/night review, LOD transitions
and artistic approval remain pending.

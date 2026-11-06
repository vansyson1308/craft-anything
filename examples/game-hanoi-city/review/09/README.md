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
or review lighting. The panels currently have a flat warm fill and need artistic
approval. All eight upper facade GLBs, including LOD1, pass glTF Validator with
zero errors and warnings; their triangle counts are unchanged and within budget.

The four `ground_0*.png` images show the ground-floor variants: open shops
have a separate emissive `shop_light` strip; the closed shutter has no strip.
`ground_01_world_zero.png` shows the strip with no world or review lighting.
All eight ground-floor GLBs pass glTF Validator without errors or warnings,
and their mesh bounds extend no farther than 0.59 m in the outward +Z direction.

The emissive lens, windows and shop strips currently remain emissive during the
day; no automatic time-of-day material switching is implemented here. The
closed shutter retains an unused original emissive image in its GLB, but no
material references it. Day/night game views, LOD transitions and artistic
approval are also pending.

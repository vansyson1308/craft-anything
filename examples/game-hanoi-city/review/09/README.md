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

The emissive lens currently remains emissive during the day; no automatic
time-of-day material switching is implemented here. The other part of A4 —
separating glowing windows from non-emissive facade fittings — is still
outstanding. Day/night in-game views and artistic approval are also pending.

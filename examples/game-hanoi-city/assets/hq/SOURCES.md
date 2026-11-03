# Provenance and license

All delivered model, texture and authored animation assets are offered under CC0-1.0 for unrestricted public/open-source use. No game-extracted assets, brand marks, photographs as textures, or real-person likenesses were used.

- Character base body portions and exact 65-joint skeleton: Quaternius Universal Base Characters (CC0), supplied through the user's game repository. https://quaternius.com/packs/universalbasecharacters.html
- Exact source: [game Minh skeleton](../people/minh/person.gltf) and [shared binary data](../people/shared/person.bin). Source hashes are in reports/spec-checks.json. The game skeleton nodes and inverse bind matrices are preserved exactly; fresh clothing, accessories and a different head were fitted to it.
- Head mesh and synthetic Asian young male morph: MakeHuman Community core assets, CC0. Repository commit a8bc2d54ff0ac92e78ff71431b1023eda42bf482. https://github.com/makehumancommunity/makehuman ; https://www.makehumancommunity.org/about/license.html
- External 43 clips used only for compatibility testing: user's anim.glb, based on Quaternius Universal Animation Library, CC0. https://quaternius.com/packs/universalanimationlibrary.html . These are not duplicated in the delivery.
- Bridge, tree, dog, dog clips, clothing/accessories and all PBR image maps: original procedural authoring by Codex in Blender/Python for this request. AO was baked with Cycles on CPU and packed into ORM R. Normal maps use OpenGL tangent-space convention. Tangents are included in each primitive.
- Validation: Khronos glTF Validator (Node package). Rendering: Blender Cycles and Three.js in Chrome. The preview renders derive from delivered geometry, not image generation.

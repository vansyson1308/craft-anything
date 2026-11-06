"""Separate the existing streetlight lens from its non-emissive metal.

Run with Blender 4.5's Python environment: source and destination are GLBs.
The lamp coordinates are in Blender's Z-up space. The component checks prevent
silently assigning the wrong faces if the source geometry changes.
"""
import argparse
import json
import struct
from pathlib import Path
import bpy
import bmesh

p = argparse.ArgumentParser()
p.add_argument('source', type=Path)
p.add_argument('destination', type=Path)
args = p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
if len(meshes) != 1:
    raise RuntimeError('Expected one mesh in the supplied streetlight')
obj = meshes[0]
bm = bmesh.new()
bm.from_mesh(obj.data)
bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
remaining = set(bm.verts)
candidates = []
while remaining:
    stack = [remaining.pop()]
    group = []
    while stack:
        vert = stack.pop()
        group.append(vert)
        for edge in vert.link_edges:
            other = edge.other_vert(vert)
            if other in remaining:
                remaining.remove(other)
                stack.append(other)
    lo = [min(v.co[i] for v in group) for i in range(3)]
    hi = [max(v.co[i] for v in group) for i in range(3)]
    if (len(group) >= 10 and -.13 <= lo[0] <= -.08 and .08 <= hi[0] <= .13
            and -1.24 <= lo[1] <= -1.20 and -.76 <= hi[1] <= -.72
            and 4.44 <= lo[2] <= 4.46 and 4.46 <= hi[2] <= 4.48):
        candidates.append(group)
if len(candidates) != 1:
    raise RuntimeError(f'Expected one identifiable lens, found {len(candidates)}')
key = lambda co: tuple(round(v, 5) for v in co)
lens_vertices = {key(v.co) for v in candidates[0]}
bm.free()

if len(obj.data.materials) != 1:
    raise RuntimeError('Expected one original metal material')
metal = obj.data.materials[0]
metal.name = 'streetlight_metal'
shader = metal.node_tree.nodes.get('Principled BSDF')
if shader is None:
    raise RuntimeError('Missing imported Principled BSDF')
emission = shader.inputs['Emission Color']
for link in list(emission.links):
    metal.node_tree.links.remove(link)
emission.default_value = (0, 0, 0, 1)
shader.inputs['Emission Strength'].default_value = 0

glass = bpy.data.materials.new('lamp_glass')
glass.use_nodes = True
shader = glass.node_tree.nodes.get('Principled BSDF')
shader.inputs['Base Color'].default_value = (1.0, .79, .49, 1)
shader.inputs['Roughness'].default_value = .35
shader.inputs['Emission Color'].default_value = (1.0, .68, .35, 1)
shader.inputs['Emission Strength'].default_value = 1
obj.data.materials.append(glass)
selected = 0
for poly in obj.data.polygons:
    if all(key(obj.data.vertices[v].co) in lens_vertices for v in poly.vertices):
        poly.material_index = 1
        selected += 1
if selected < 10:
    raise RuntimeError(f'Lens selection unexpectedly small: {selected} faces')
print('Lens component:', len(lens_vertices), 'vertices;', selected, 'faces')
args.destination.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.export_scene.gltf(filepath=str(args.destination.resolve()), export_format='GLB',
                          export_tangents=True, export_extras=True)

# The existing lens UV contains degenerate tangent vectors. Its simple glass
# material has no normal map, so omit only that primitive's tangents. Preserve
# the metal's original normal-map tangents and all binary buffer data.
raw = args.destination.read_bytes()
magic, version, length = struct.unpack_from('<4sII', raw)
if (magic, version, length) != (b'glTF', 2, len(raw)):
    raise RuntimeError('Unexpected GLB header')
offset = 12
chunks = []
while offset < len(raw):
    size, kind = struct.unpack_from('<I4s', raw, offset)
    offset += 8
    chunks.append((kind, raw[offset:offset + size]))
    offset += size
if not chunks or chunks[0][0] != b'JSON':
    raise RuntimeError('Missing GLB JSON chunk')
doc = json.loads(chunks[0][1])
glass_index = next(i for i, mat in enumerate(doc['materials']) if mat.get('name') == 'lamp_glass')
removed = 0
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        if primitive.get('material') == glass_index and 'TANGENT' in primitive['attributes']:
            del primitive['attributes']['TANGENT']
            removed += 1
if not removed:
    raise RuntimeError('Expected a tangent-bearing glass primitive')
encoded = json.dumps(doc, ensure_ascii=False, separators=(',', ':')).encode('utf8')
encoded += b' ' * ((-len(encoded)) % 4)
chunks[0] = (b'JSON', encoded)
payload = b''.join(struct.pack('<I4s', len(chunk), kind) + chunk for kind, chunk in chunks)
args.destination.write_bytes(struct.pack('<4sII', b'glTF', 2, len(payload) + 12) + payload)

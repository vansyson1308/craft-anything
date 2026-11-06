"""Split the two upper-storey window panels into an emissive material.

Input is one of the existing mat_tien_tang_01–04 LOD0/LOD1 GLBs. Guarded
component bounds intentionally reject unfamiliar variants. Blender 4.5 Python.
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
    raise RuntimeError('Expected one mesh for an upper facade')
obj = meshes[0]
bm = bmesh.new()
bm.from_mesh(obj.data)
bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
remaining = set(bm.verts)
groups = []
while remaining:
    stack = [remaining.pop()]
    group = []
    while stack:
        v = stack.pop()
        group.append(v)
        for edge in v.link_edges:
            other = edge.other_vert(v)
            if other in remaining:
                remaining.remove(other)
                stack.append(other)
    lo = [min(v.co[i] for v in group) for i in range(3)]
    hi = [max(v.co[i] for v in group) for i in range(3)]
    if (len(group) >= 8 and .99 < hi[0]-lo[0] < 1.1 and
            1.6 < hi[2]-lo[2] < 1.8 and -.04 < lo[1] < 0 and .04 < hi[1] < .08):
        groups.append(group)
if len(groups) != 2:
    raise RuntimeError(f'Expected two upper facade window components, found {len(groups)}')
key = lambda co: tuple(round(v, 5) for v in co)
window_vertices = {key(v.co) for group in groups for v in group}
bm.free()
metal = next((m for m in obj.data.materials if m and m.name == 'Scanned joinery metal and fittings'), None)
if metal is None:
    raise RuntimeError('Expected original joinery material')
metal.name = 'facade_joinery_non_emissive'
shader = metal.node_tree.nodes.get('Principled BSDF')
emission = shader.inputs['Emission Color']
for link in list(emission.links):
    metal.node_tree.links.remove(link)
emission.default_value = (0, 0, 0, 1)
shader.inputs['Emission Strength'].default_value = 0
window = bpy.data.materials.new('window_glow')
window.use_nodes = True
shader = window.node_tree.nodes.get('Principled BSDF')
shader.inputs['Base Color'].default_value = (.23, .14, .065, 1)
shader.inputs['Roughness'].default_value = .35
shader.inputs['Emission Color'].default_value = (.8, .38, .10, 1)
shader.inputs['Emission Strength'].default_value = .5
obj.data.materials.append(window)
selected = 0
for poly in obj.data.polygons:
    if all(key(obj.data.vertices[v].co) in window_vertices for v in poly.vertices):
        poly.material_index = len(obj.data.materials)-1
        selected += 1
if selected < 8:
    raise RuntimeError(f'Window selection unexpectedly small: {selected} faces')
print('Window components:', len(groups), 'faces:', selected)
args.destination.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.export_scene.gltf(filepath=str(args.destination.resolve()), export_format='GLB',
                          export_tangents=True, export_extras=True)

# This simple emissive panel does not use a normal map. Remove its generated
# tangents while preserving normal-map tangents on all original materials.
raw = args.destination.read_bytes()
magic, version, length = struct.unpack_from('<4sII', raw)
if (magic, version, length) != (b'glTF', 2, len(raw)):
    raise RuntimeError('Unexpected GLB header')
offset, chunks = 12, []
while offset < len(raw):
    size, kind = struct.unpack_from('<I4s', raw, offset)
    offset += 8
    chunks.append((kind, raw[offset:offset+size]))
    offset += size
if not chunks or chunks[0][0] != b'JSON':
    raise RuntimeError('Missing JSON chunk')
doc = json.loads(chunks[0][1])
window_index = next(i for i, mat in enumerate(doc['materials']) if mat.get('name') == 'window_glow')
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        if primitive.get('material') == window_index:
            primitive['attributes'].pop('TANGENT', None)
encoded = json.dumps(doc, ensure_ascii=False, separators=(',', ':')).encode('utf8')
encoded += b' ' * ((-len(encoded)) % 4)
chunks[0] = (b'JSON', encoded)
payload = b''.join(struct.pack('<I4s', len(chunk), kind)+chunk for kind, chunk in chunks)
args.destination.write_bytes(struct.pack('<4sII', b'glTF', 2, len(payload)+12)+payload)

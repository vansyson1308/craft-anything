"""Remove false joinery emission and retain the open-shop light strip.

Input is a mat_tien_tret_01–04 LOD0/LOD1 GLB. The closed-shutter variant 02
has no strip. Component bounds are checked so unfamiliar geometry fails safely.
"""
import argparse
import json
import re
import struct
import sys
from pathlib import Path
import bpy
import bmesh

p = argparse.ArgumentParser()
p.add_argument('source', type=Path)
p.add_argument('destination', type=Path)
args = p.parse_args()
match = re.fullmatch(r'mat_tien_tret_0([1-4])(?:_lod1)?\.glb', args.source.name)
if not match:
    raise ValueError('Expected a known ground-floor facade filename')
closed = match.group(1) == '2'
if closed:
    # This shutter has no illuminated component to split. Editing only the
    # material JSON preserves its original, already-valid metal tangents.
    raw = args.source.read_bytes()
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
    metal = next((m for m in doc['materials'] if m.get('name') == 'Scanned joinery metal and fittings'), None)
    if metal is None:
        raise RuntimeError('Expected original joinery material')
    metal['name'] = 'facade_joinery_non_emissive'
    metal.pop('emissiveTexture', None)
    metal['emissiveFactor'] = [0, 0, 0]
    encoded = json.dumps(doc, ensure_ascii=False, separators=(',', ':')).encode('utf8')
    encoded += b' ' * ((-len(encoded)) % 4)
    chunks[0] = (b'JSON', encoded)
    payload = b''.join(struct.pack('<I4s', len(chunk), kind)+chunk for kind, chunk in chunks)
    args.destination.parent.mkdir(parents=True, exist_ok=True)
    args.destination.write_bytes(struct.pack('<4sII', b'glTF', 2, len(payload)+12)+payload)
    print('Closed shutter: original geometry preserved, joinery emission removed')
    sys.exit(0)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
meshes = [o for o in bpy.context.scene.objects if o.type == 'MESH']
if len(meshes) != 1:
    raise RuntimeError('Expected one mesh for a shopfront')
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
    if (len(group) >= 6 and .44 < lo[0] < .46 and 3.54 < hi[0] < 3.56 and
            -.15 < lo[1] < -.13 and -.07 < hi[1] < -.05 and
            2.52 < lo[2] < 2.54 and 2.58 < hi[2] < 2.60):
        groups.append(group)
expected = 0 if closed else 1
if len(groups) != expected:
    raise RuntimeError(f'Expected {expected} shop light components, found {len(groups)}')
key = lambda co: tuple(round(v, 5) for v in co)
light_vertices = {key(v.co) for group in groups for v in group}
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
if not closed:
    light = bpy.data.materials.new('shop_light')
    light.use_nodes = True
    shader = light.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Base Color'].default_value = (1, .79, .49, 1)
    shader.inputs['Roughness'].default_value = .4
    shader.inputs['Emission Color'].default_value = (1, .68, .35, 1)
    shader.inputs['Emission Strength'].default_value = .6
    obj.data.materials.append(light)
    count = 0
    for poly in obj.data.polygons:
        if all(key(obj.data.vertices[v].co) in light_vertices for v in poly.vertices):
            poly.material_index = len(obj.data.materials)-1
            count += 1
    if count < 6:
        raise RuntimeError(f'Shop light selection unexpectedly small: {count} faces')
    print('Shop light faces:', count)
else:
    print('Closed shutter: no light strip')
args.destination.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.export_scene.gltf(filepath=str(args.destination.resolve()), export_format='GLB',
                          export_tangents=True, export_extras=True)

if not closed:
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
    index = next(i for i, mat in enumerate(doc['materials']) if mat.get('name') == 'shop_light')
    for mesh in doc['meshes']:
        for primitive in mesh['primitives']:
            if primitive.get('material') == index:
                primitive['attributes'].pop('TANGENT', None)
    encoded = json.dumps(doc, ensure_ascii=False, separators=(',', ':')).encode('utf8')
    encoded += b' ' * ((-len(encoded)) % 4)
    chunks[0] = (b'JSON', encoded)
    payload = b''.join(struct.pack('<I4s', len(chunk), kind)+chunk for kind, chunk in chunks)
    args.destination.write_bytes(struct.pack('<4sII', b'glTF', 2, len(payload)+12)+payload)

"""Create an experimental distant LOD; inspect shape and weights before use."""
import argparse
import json
import struct
from pathlib import Path
import bpy

p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);p.add_argument('--lod2',action='store_true');args=p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
rig=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE')
meshes=[o for o in bpy.context.scene.objects if o.type=='MESH' and o.vertex_groups]
for obj in meshes:
    world=obj.matrix_world.copy();obj.parent=None;obj.matrix_world=world
    name=obj.name
    if name=='study_Human':ratio=.05 if args.lod2 else .30
    elif 'male_casualsuit05' in name:ratio=.105 if args.lod2 else .35
    elif 'shoes01' in name:ratio=.05 if args.lod2 else .26
    elif 'short01' in name:ratio=.075 if args.lod2 else .30
    elif name in ('cap_crown','cap_brim','face_covering'):ratio=1.0 if args.lod2 else .75
    else:ratio=.25 if args.lod2 else 1.0
    if ratio<1:
        bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
        mod=obj.modifiers.new('lod','DECIMATE');mod.ratio=ratio
        # Collapse the bind-pose mesh before skinning.
        while obj.modifiers.find(mod.name)>0:bpy.ops.object.modifier_move_up(modifier=mod.name)
        bpy.ops.object.modifier_apply(modifier=mod.name)
    bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
    bpy.ops.object.vertex_group_limit_total(group_select_mode='ALL',limit=4)
    bpy.ops.object.vertex_group_normalize_all(group_select_mode='ALL',lock_active=False)
size=256 if args.lod2 else 512
for image in bpy.data.images:
    if image.source=='FILE' and max(image.size)>size:image.scale(size,size)
bpy.ops.object.select_all(action='DESELECT');rig.select_set(True)
for obj in meshes:obj.select_set(True)
bpy.context.view_layer.objects.active=rig
args.output.parent.mkdir(parents=True,exist_ok=True)
bpy.ops.export_scene.gltf(filepath=str(args.output.resolve()),export_format='GLB',use_selection=True,export_tangents=True,export_extras=True)
# Blender sometimes emits degenerate tangents on materials that do not use a
# normal map. Remove those attributes while retaining tangents for normal maps.
raw=args.output.read_bytes();length=struct.unpack_from('<I',raw,12)[0]
doc=json.loads(raw[20:20+length]);tail=raw[20+length:]
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        if 'normalTexture' not in doc['materials'][primitive['material']]:
            primitive['attributes'].pop('TANGENT',None)
encoded=json.dumps(doc,separators=(',',':'),ensure_ascii=False).encode()
encoded+=b' '*((-len(encoded))%4)
chunks=struct.pack('<I4s',len(encoded),b'JSON')+encoded+tail
args.output.write_bytes(struct.pack('<4sII',b'glTF',2,12+len(chunks))+chunks)

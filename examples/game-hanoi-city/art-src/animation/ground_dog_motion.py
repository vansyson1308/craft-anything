"""Offset draft Jump root keys so its lowest skinned point stays at the floor.

This is a numerical floor correction, not a substitute for paw contact or
animation review. Blender 4.5 Python, with scene fps fixed at 30.
"""
import argparse
import json
import struct
from pathlib import Path
import bpy

p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('destination',type=Path);a=p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True);scene=bpy.context.scene;scene.render.fps=30
bpy.ops.import_scene.gltf(filepath=str(a.source.resolve()))
arm=next(o for o in scene.objects if o.type=='ARMATURE');dog=next(o for o in scene.objects if o.type=='MESH' and o.vertex_groups)
arm.animation_data.action=bpy.data.actions['Jump'];heights=[]
for f in range(31):
    scene.frame_set(f);evaluated=dog.evaluated_get(bpy.context.evaluated_depsgraph_get());mesh=evaluated.to_mesh()
    heights.append(min((evaluated.matrix_world@v.co).z for v in mesh.vertices));evaluated.to_mesh_clear()
raw=a.source.read_bytes();jlen=struct.unpack_from('<I',raw,12)[0];doc=json.loads(raw[20:20+jlen]);binary_offset=20+jlen+8
anim=next(x for x in doc['animations'] if x['name']=='Jump');root=next(i for i,n in enumerate(doc['nodes']) if n.get('name')=='root')
ch=next(x for x in anim['channels'] if x['target']=={'node':root,'path':'translation'});acc=doc['accessors'][anim['samplers'][ch['sampler']]['output']]
view=doc['bufferViews'][acc['bufferView']]
if acc['type']!='VEC3' or acc['componentType']!=5126 or acc['count']!=31 or view.get('byteStride',12)!=12:raise ValueError('Unexpected root keys')
start=binary_offset+view.get('byteOffset',0)+acc.get('byteOffset',0);out=bytearray(raw);baseline=heights[0]
for i,h in enumerate(heights):
    pos=start+i*12;x,y,z=struct.unpack_from('<3f',out,pos);struct.pack_into('<3f',out,pos,x,y+(baseline-h),z)
a.destination.parent.mkdir(parents=True,exist_ok=True);a.destination.write_bytes(out)
print('Ground correction relative to frame 0:',round(min(heights)-baseline,5),'to',round(max(heights)-baseline,5),'m')

"""Check preserved source data, clip endpoints and sampled dog floor clearance."""
import argparse
import json
import struct
from pathlib import Path
import bpy


def read(path):
    b=path.read_bytes();n=struct.unpack_from('<I',b,12)[0]
    return b,json.loads(b[20:20+n]),20+n+8


p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('study',type=Path);a=p.parse_args()
original,old,_=read(a.source);data,new,offset=read(a.study)
for key in ('meshes','skins','materials','nodes'):
    if new[key]!=old[key]:raise AssertionError('changed '+key)
if new['animations'][:len(old['animations'])]!=old['animations']:raise AssertionError('original clips changed')
for name in ('Wag','Jump'):
    clip=next(x for x in new['animations'] if x['name']==name);error=0
    for sampler in clip['samplers']:
        acc=new['accessors'][sampler['output']];view=new['bufferViews'][acc['bufferView']]
        width=4 if acc['type']=='VEC4' else 3;start=offset+view.get('byteOffset',0)+acc.get('byteOffset',0)
        first=struct.unpack_from('<'+'f'*width,data,start);last=struct.unpack_from('<'+'f'*width,data,start+(acc['count']-1)*4*width)
        error=max(error,max(abs(x-y) for x,y in zip(first,last)))
    if error>1e-5:raise AssertionError(f'{name} does not return to start: {error}')
    print(name,'channels',len(clip['channels']),'endpoint max error',error)
bpy.ops.wm.read_factory_settings(use_empty=True);scene=bpy.context.scene;scene.render.fps=30
bpy.ops.import_scene.gltf(filepath=str(a.study.resolve()));arm=next(o for o in scene.objects if o.type=='ARMATURE')
dog=next(o for o in scene.objects if o.type=='MESH' and o.vertex_groups);arm.animation_data.action=bpy.data.actions['Jump'];heights=[]
for half in range(61):
    scene.frame_set(half//2,subframe=(half%2)*.5);obj=dog.evaluated_get(bpy.context.evaluated_depsgraph_get());mesh=obj.to_mesh()
    heights.append(min((obj.matrix_world@v.co).z for v in mesh.vertices));obj.to_mesh_clear()
print('Jump lowest skinned point, half-frame samples:',round(min(heights),5),'to',round(max(heights),5),'metres')
if min(heights)<-.005:raise AssertionError('Sampled mesh passes below floor by over 5 mm')
print('Existing mesh, skin, material, nodes and five original clip definitions preserved.')

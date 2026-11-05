"""Experimental QC flashlight materials; outputs a GLB for review."""
import argparse
from pathlib import Path
import bpy
import numpy as np
from PIL import Image, ImageDraw

p=argparse.ArgumentParser()
p.add_argument('source',type=Path)
p.add_argument('output',type=Path)
p.add_argument('--size',type=int,default=512)
p.add_argument('--high',type=Path,help='optional matching high-poly GLB for a normal bake')
args=p.parse_args()
args.output.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
for existing in bpy.data.materials:
    existing.name = 'source_' + existing.name
meshes={o.name:o for o in bpy.context.scene.objects if o.type=='MESH'}
body=[meshes[k] for k in ('grip','bezel','tail_cap','lanyard_loop')]
for o in body:
    for poly in o.data.polygons:
        poly.use_smooth=abs(poly.normal.z)<.8
bpy.ops.object.select_all(action='DESELECT')
for o in body: o.select_set(True)
bpy.context.view_layer.objects.active=body[0]
bpy.ops.object.mode_set(mode='EDIT')
bpy.ops.mesh.select_all(action='SELECT')
bpy.ops.uv.smart_project(island_margin=.006)
bpy.ops.object.mode_set(mode='OBJECT')
size=args.size
grip=Image.new('L',(size,size),0)
wear=Image.new('L',(size,size),0)
gd,wd=ImageDraw.Draw(grip),ImageDraw.Draw(wear)
for obj in body:
    uv=obj.data.uv_layers.active.data
    zmax=max(v.co.z for v in obj.data.vertices)
    zmin=min(v.co.z for v in obj.data.vertices)
    edges={}
    for poly in obj.data.polygons:
        points=[(int(uv[i].uv.x*(size-1)),int((1-uv[i].uv.y)*(size-1))) for i in poly.loop_indices]
        if obj.name=='grip' and abs(poly.normal.z)<.7:
            gd.polygon(points,fill=255)
        if obj.name in ('bezel','tail_cap'):
            ids=list(poly.loop_indices)
            for a,b in zip(ids,ids[1:]+ids[:1]):
                va=obj.data.vertices[obj.data.loops[a].vertex_index].co
                vb=obj.data.vertices[obj.data.loops[b].vertex_index].co
                same_end=(abs(va.z-zmax)<.0001 and abs(vb.z-zmax)<.0001) or (abs(va.z-zmin)<.0001 and abs(vb.z-zmin)<.0001)
                radius_limit=.018 if obj.name=='bezel' else .0125
                if same_end and va.xy.length>radius_limit and vb.xy.length>radius_limit and abs(va.xy.length-vb.xy.length)<.0002 and (va-vb).length<.006:
                    pa=(int(uv[a].uv.x*(size-1)),int((1-uv[a].uv.y)*(size-1)))
                    pb=(int(uv[b].uv.x*(size-1)),int((1-uv[b].uv.y)*(size-1)))
                    key=tuple(sorted((tuple(round(c,5) for c in va),tuple(round(c,5) for c in vb))))
                    edges.setdefault(key,[]).append((poly.normal.copy(),pa,pb))
    # High angular change between adjacent face normals identifies a sharp
    # rim. Draw only the UV edges belonging to that curvature mask.
    for entries in edges.values():
        if any(a[0].dot(b[0])<.9 for a in entries for b in entries):
            for _,pa,pb in entries:wd.line((pa,pb),fill=255,width=2)
g=np.asarray(grip)>0
w=np.asarray(wear)>0
y,x=np.mgrid[0:size,0:size]
# The grip's measured UV scale is about 2.75 pixels/mm at 512px.  Diagonal
# lines at 3.9 pixels in x+y coordinates are approximately 1 mm apart.
period=3.9*size/512
height=(np.cos(2*np.pi*(x+y)/period)+np.cos(2*np.pi*(x-y)/period))*.5*g
dy,dx=np.gradient(height.astype(float))
normal=np.stack((-dx*.15,dy*.15,np.ones_like(dx)),axis=-1)
normal/=np.maximum(np.linalg.norm(normal,axis=2,keepdims=True),1e-9)
normal_path=args.output/'normal.png'
Image.fromarray(np.uint8((normal+1)*127.5)).save(normal_path)
base=np.empty((size,size,3),dtype=np.uint8);base[:]=[48,48,52];base[w]=[231,231,234]
base_path=args.output/'basecolor.png';Image.fromarray(base).save(base_path)
orm=np.empty((size,size,3),dtype=np.uint8);orm[:]=[255,89,255];orm[w,1]=77
orm_path=args.output/'orm.png';Image.fromarray(orm).save(orm_path)
mat=bpy.data.materials.new('body_anodized');mat.use_nodes=True
nodes=mat.node_tree.nodes;links=mat.node_tree.links
sh=nodes.get('Principled BSDF')
def tex(path,noncolor=False):
    n=nodes.new('ShaderNodeTexImage');n.image=bpy.data.images.load(str(path.resolve()))
    if noncolor:n.image.colorspace_settings.name='Non-Color'
    return n
links.new(tex(base_path).outputs['Color'],sh.inputs['Base Color'])
n=tex(normal_path,True);nm=nodes.new('ShaderNodeNormalMap');links.new(n.outputs['Color'],nm.inputs['Color']);links.new(nm.outputs['Normal'],sh.inputs['Normal'])
packed=tex(orm_path,True);sep=nodes.new('ShaderNodeSeparateColor');links.new(packed.outputs['Color'],sep.inputs['Color']);links.new(sep.outputs['Green'],sh.inputs['Roughness']);links.new(sep.outputs['Blue'],sh.inputs['Metallic'])
for obj in body:obj.data.materials.clear();obj.data.materials.append(mat)
# Bake ambient occlusion on the packed body UVs, then attach the red channel
# through Blender's standard glTF material-output group.
scene=bpy.context.scene
scene.render.engine='CYCLES'
scene.cycles.samples=16
scene.render.bake.margin=2
ao=bpy.data.images.new('flashlight_ao',width=size,height=size,alpha=False)
target=nodes.new('ShaderNodeTexImage');target.image=ao;nodes.active=target
bpy.ops.object.select_all(action='DESELECT')
for obj in body:obj.select_set(True)
bpy.context.view_layer.objects.active=body[0]
bpy.ops.object.bake(type='AO')
ao_path=args.output/'ao.png';ao.filepath_raw=str(ao_path.resolve());ao.file_format='PNG';ao.save()
nodes.remove(target)
orm[...,0]=np.asarray(Image.open(ao_path).convert('L'))
Image.fromarray(orm).save(orm_path)
packed.image.reload()
group=bpy.data.node_groups.new('glTF Material Output','ShaderNodeTree')
group.interface.new_socket(name='Occlusion',in_out='INPUT',socket_type='NodeSocketFloat')
group_node=nodes.new('ShaderNodeGroup');group_node.node_tree=group
links.new(sep.outputs['Red'],group_node.inputs['Occlusion'])
if args.high:
    before=set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(args.high.resolve()))
    imported=[obj for obj in bpy.data.objects if obj not in before]
    high={obj.name.removesuffix('.001'):obj for obj in imported if obj.type=='MESH'}
    missing=[obj.name for obj in body if obj.name not in high]
    if missing:raise RuntimeError(f'Missing high-poly objects: {missing}')
    for obj in high.values():
        for poly in obj.data.polygons:poly.use_smooth=abs(poly.normal.z)<.8
    baked=bpy.data.images.new('flashlight_geometry_normal',width=size,height=size,alpha=False)
    baked.generated_color=(.5,.5,1,1)
    baked.colorspace_settings.name='Non-Color'
    target=nodes.new('ShaderNodeTexImage');target.image=baked;nodes.active=target
    scene.render.bake.use_selected_to_active=True
    scene.render.bake.use_cage=True
    scene.render.bake.cage_extrusion=.001
    scene.render.bake.max_ray_distance=.004
    scene.render.bake.use_clear=False
    for obj in body:
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True);high[obj.name].select_set(True)
        bpy.context.view_layer.objects.active=obj
        bpy.ops.object.bake(type='NORMAL')
    geom_path=args.output/'geometry_normal.png'
    baked.filepath_raw=str(geom_path.resolve());baked.file_format='PNG';baked.save()
    for obj in imported:bpy.data.objects.remove(obj,do_unlink=True)
    nodes.remove(target)
    geom=np.asarray(Image.open(geom_path).convert('RGB'),dtype=np.float32)/127.5-1
    detail=np.asarray(Image.open(normal_path).convert('RGB'),dtype=np.float32)/127.5-1
    combined=np.stack((geom[...,0]+detail[...,0],geom[...,1]+detail[...,1],geom[...,2]*detail[...,2]),axis=-1)
    combined/=np.maximum(np.linalg.norm(combined,axis=2,keepdims=True),1e-9)
    Image.fromarray(np.uint8(np.clip((combined+1)*127.5,0,255))).save(normal_path)
    n.image.reload()
reflector=meshes['reflector'];reflector.data.materials.clear()
metal=bpy.data.materials.new('reflector');metal.use_nodes=True
m=metal.node_tree.nodes.get('Principled BSDF');m.inputs['Base Color'].default_value=(.8,.8,.82,1);m.inputs['Metallic'].default_value=1;m.inputs['Roughness'].default_value=.08
reflector.data.materials.append(metal)
for poly in reflector.data.polygons:poly.use_smooth=True
switch=meshes['rubber_switch'];switch.data.materials.clear()
rubber=bpy.data.materials.new('rubber');rubber.use_nodes=True
r=rubber.node_tree.nodes.get('Principled BSDF');r.inputs['Base Color'].default_value=(.015,.015,.015,1);r.inputs['Roughness'].default_value=.8
switch.data.materials.append(rubber)
lens=meshes['lens'];lens.data.materials.clear()
uv=lens.data.uv_layers.active or lens.data.uv_layers.new(name='UVMap')
for loop in lens.data.loops:
    pos=lens.data.vertices[loop.vertex_index].co
    uv.data[loop.index].uv=(.5+pos.x/.032,.5+pos.y/.032)
rad=np.sqrt(((x-size/2)/(size/2))**2+((y-size/2)/(size/2))**2)
strength=np.clip(np.exp(-(rad/.22)**2),0,1)
emission=np.stack([strength*255,strength*246,strength*229],axis=-1).astype('uint8')
emission_path=args.output/'emissive.png';Image.fromarray(emission).save(emission_path)
glass=bpy.data.materials.new('lamp_glass');glass.use_nodes=True;glass.surface_render_method='DITHERED'
shg=glass.node_tree.nodes.get('Principled BSDF');shg.inputs['Base Color'].default_value=(1,.92,.78,1);shg.inputs['Roughness'].default_value=.08;shg.inputs['Alpha'].default_value=.14;shg.inputs['Emission Strength'].default_value=1
em=glass.node_tree.nodes.new('ShaderNodeTexImage');em.image=bpy.data.images.load(str(emission_path.resolve()));glass.node_tree.links.new(em.outputs['Color'],shg.inputs['Emission Color'])
lens.data.materials.append(glass)
bpy.ops.object.select_all(action='DESELECT')
for o in body:o.select_set(True)
bpy.context.view_layer.objects.active=body[0];bpy.ops.object.join()
bpy.ops.export_scene.gltf(filepath=str((args.output/'den_pin.glb').resolve()),export_format='GLB',export_tangents=True)

"""Draft lantern shared body atlas and emissive inner bulb."""
import argparse
from pathlib import Path
import bpy
import bmesh
import numpy as np
from PIL import Image,ImageDraw,ImageFilter
from pack_lantern_uv import pack_uv

p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('out',type=Path);p.add_argument('--variant',type=int,required=True);p.add_argument('--size',type=int,default=512);p.add_argument('--high',type=Path);p.add_argument('--scans',type=Path,required=True);a=p.parse_args()
a.out.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(a.source.resolve()))
meshes=[x for x in bpy.context.scene.objects if x.type=='MESH']
bulb=next(x for x in meshes if x.name=='inner_bulb')
body=[x for x in meshes if x is not bulb]
for m in bpy.data.materials:m.name='source_'+m.name
for ob in body:
    # The glTF import splits vertices at hard normals; weld coincident points
    # before UV unwrapping so each physical shell is not hundreds of islands.
    bm=bmesh.new();bm.from_mesh(ob.data)
    bmesh.ops.remove_doubles(bm,verts=bm.verts,dist=1e-7)
    bm.to_mesh(ob.data);bm.free()
    if ob.name=='silk_shell':
        for f in ob.data.polygons:f.use_smooth=True
bpy.ops.object.select_all(action='DESELECT')
for ob in body:ob.select_set(True)
bpy.context.view_layer.objects.active=body[0];bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.uv.smart_project(island_margin=.006);bpy.ops.object.mode_set(mode='OBJECT')
size=a.size
print('Lantern UV packing:',pack_uv(body,size,padding=2))
masks={x:Image.new('L',(size,size),0) for x in ('silk','tassel')}
for ob in body:
    typ='silk' if ob.name.startswith('silk_') else 'tassel' if ob.name.startswith('tassel_') else None
    if not typ:continue
    uv=ob.data.uv_layers.active.data;d=ImageDraw.Draw(masks[typ])
    for f in ob.data.polygons:
        d.polygon([(int(uv[i].uv.x*(size-1)),int((1-uv[i].uv.y)*(size-1))) for i in f.loop_indices],fill=255)
silk=np.asarray(masks['silk'].filter(ImageFilter.MaxFilter(5)))>0
tassel=np.asarray(masks['tassel'].filter(ImageFilter.MaxFilter(5)))>0
y,x=np.mgrid[:size,:size];rng=np.random.default_rng(513)
base=np.zeros((size,size,4),dtype=np.uint8);base[:]=[160,117,61,255]
scan=np.asarray(Image.open(a.scans/'crepe_satin_diff.jpg').convert('RGB').resize((size,size),Image.Resampling.LANCZOS),dtype=np.float32).mean(axis=2)
noise=rng.normal(0,2,(size,size))+(scan-scan.mean())*.3
color=np.array([192,36,29] if a.variant!=2 else [219,164,35])
colored=np.clip(color[None,None,:]+noise[:,:,None],0,255).astype('uint8')
base[silk,:3]=colored[silk];base[silk,3]=255
base[tassel,:3]=[150,28,23]
Image.fromarray(base,'RGBA').save(a.out/'basecolor.png')
height=(np.sin(x*1.4)+np.sin(y*1.7))*.5*silk
dy,dx=np.gradient(height);normal=np.stack((-dx*.1,dy*.1,np.ones_like(dx)),axis=-1);normal/=np.linalg.norm(normal,axis=2,keepdims=True)
scan_normal=np.asarray(Image.open(a.scans/'crepe_satin_nor_gl.png').convert('RGB').resize((size,size),Image.Resampling.LANCZOS),dtype=np.float32)/127.5-1
normal[silk,:2]+=scan_normal[silk,:2]*.25
normal/=np.maximum(np.linalg.norm(normal,axis=2,keepdims=True),1e-9)
Image.fromarray(np.uint8((normal+1)*127.5)).save(a.out/'normal.png')
orm=np.zeros((size,size,3),dtype=np.uint8);orm[:]=[255,211,0]
arm=np.asarray(Image.open(a.scans/'crepe_satin_arm.png').convert('RGB').resize((size,size),Image.Resampling.LANCZOS))
orm[silk,1]=arm[silk,1]
mat=bpy.data.materials.new('lantern_body');mat.use_nodes=True;nodes=mat.node_tree.nodes;links=mat.node_tree.links;sh=nodes.get('Principled BSDF');sh.inputs['Metallic'].default_value=0
def texture(path,noncolor=False):
    t=nodes.new('ShaderNodeTexImage');t.image=bpy.data.images.load(str(path.resolve()));
    if noncolor:t.image.colorspace_settings.name='Non-Color'
    return t
base_node=texture(a.out/'basecolor.png');links.new(base_node.outputs['Color'],sh.inputs['Base Color'])
emissive=np.zeros((size,size,3),dtype=np.uint8)
emissive[silk]=[145,25,10] if a.variant!=2 else [170,95,12]
Image.fromarray(emissive).save(a.out/'emissive.png')
em=texture(a.out/'emissive.png');links.new(em.outputs['Color'],sh.inputs['Emission Color']);sh.inputs['Emission Strength'].default_value=1
nt=texture(a.out/'normal.png',True);nm=nodes.new('ShaderNodeNormalMap');links.new(nt.outputs['Color'],nm.inputs['Color']);links.new(nm.outputs['Normal'],sh.inputs['Normal'])
for ob in body:ob.data.materials.clear();ob.data.materials.append(mat)
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=16;scene.render.bake.margin=2
ao=bpy.data.images.new('lantern_ao',width=size,height=size,alpha=False);target=nodes.new('ShaderNodeTexImage');target.image=ao;nodes.active=target
bpy.ops.object.select_all(action='DESELECT')
for ob in body:ob.select_set(True)
bpy.context.view_layer.objects.active=body[0];bpy.ops.object.bake(type='AO')
ao.filepath_raw=str((a.out/'ao.png').resolve());ao.file_format='PNG';ao.save();nodes.remove(target)
orm[:,:,0]=np.asarray(Image.open(a.out/'ao.png').convert('L'));Image.fromarray(orm).save(a.out/'orm.png')
packed=texture(a.out/'orm.png',True);sep=nodes.new('ShaderNodeSeparateColor');links.new(packed.outputs['Color'],sep.inputs['Color']);links.new(sep.outputs['Green'],sh.inputs['Roughness']);links.new(sep.outputs['Blue'],sh.inputs['Metallic'])
group=bpy.data.node_groups.new('glTF Material Output','ShaderNodeTree');group.interface.new_socket(name='Occlusion',in_out='INPUT',socket_type='NodeSocketFloat');g=nodes.new('ShaderNodeGroup');g.node_tree=group;links.new(sep.outputs['Red'],g.inputs['Occlusion'])
if a.high:
    before=set(bpy.data.objects);bpy.ops.import_scene.gltf(filepath=str(a.high.resolve()))
    imported=[o for o in bpy.data.objects if o not in before]
    high={o.name.removesuffix('.001'):o for o in imported if o.type=='MESH'}
    missing=[o.name for o in body if o.name not in high]
    if missing:raise RuntimeError(f'High-poly meshes missing: {missing}')
    for ob in high.values():
        for face in ob.data.polygons:face.use_smooth=True
    baked=bpy.data.images.new('lantern_normal_bake',width=size,height=size,alpha=False);baked.generated_color=(.5,.5,1,1);baked.colorspace_settings.name='Non-Color'
    target=nodes.new('ShaderNodeTexImage');target.image=baked;nodes.active=target
    scene.render.bake.use_selected_to_active=True;scene.render.bake.use_cage=True;scene.render.bake.cage_extrusion=.003;scene.render.bake.max_ray_distance=.006;scene.render.bake.use_clear=False
    for ob in body:
        bpy.ops.object.select_all(action='DESELECT');ob.select_set(True);high[ob.name].select_set(True);bpy.context.view_layer.objects.active=ob;bpy.ops.object.bake(type='NORMAL')
    baked.filepath_raw=str((a.out/'geometry_normal.png').resolve());baked.file_format='PNG';baked.save()
    for ob in imported:bpy.data.objects.remove(ob,do_unlink=True)
    nodes.remove(target)
    geometry=np.asarray(Image.open(a.out/'geometry_normal.png').convert('RGB'),dtype=np.float32)/127.5-1
    detail=np.asarray(Image.open(a.out/'normal.png').convert('RGB'),dtype=np.float32)/127.5-1
    combined=np.stack((geometry[:,:,0]+detail[:,:,0],geometry[:,:,1]+detail[:,:,1],geometry[:,:,2]*detail[:,:,2]),axis=-1)
    combined/=np.maximum(np.linalg.norm(combined,axis=2,keepdims=True),1e-9)
    Image.fromarray(np.uint8(np.clip((combined+1)*127.5,0,255))).save(a.out/'normal.png');nt.image.reload()
glass=bpy.data.materials.new('lamp_glass');glass.use_nodes=True;glass.surface_render_method='DITHERED';gsh=glass.node_tree.nodes.get('Principled BSDF');gsh.inputs['Base Color'].default_value=(1,.82,.50,1);gsh.inputs['Alpha'].default_value=.85;gsh.inputs['Emission Color'].default_value=(1,.7,.35,1);gsh.inputs['Emission Strength'].default_value=2
bulb.data.materials.clear();bulb.data.materials.append(glass)
for f in bulb.data.polygons:f.use_smooth=True
bpy.ops.object.select_all(action='DESELECT')
for ob in body:ob.select_set(True)
bpy.context.view_layer.objects.active=body[0];bpy.ops.object.join()
bpy.ops.export_scene.gltf(filepath=str((a.out/'lantern.glb').resolve()),export_format='GLB',export_tangents=True)

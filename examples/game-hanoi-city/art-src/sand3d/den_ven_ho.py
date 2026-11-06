"""Early single-globe lakeside lamp study in metres, not a final asset.

The 3.8 m silhouette and 3.5 m lamp centre follow the brief. This study uses
CC0 rusty_metal_02 maps; game placement, bakes and visual review remain.
"""
import argparse
import math
from pathlib import Path
import tempfile
import bpy
import bmesh
from PIL import Image, ImageEnhance
import sand3d as s3d


def lathe_lod(name, profile, sides=8):
    """Build a low-sided revolution while preserving the stepped profile."""
    vertices=[];rings=[];faces=[]
    for radius,z in profile:
        if radius == 0:
            rings.append([len(vertices)]);vertices.append((0,0,z))
        else:
            ring=[]
            for i in range(sides):
                angle=2*math.pi*i/sides;ring.append(len(vertices))
                vertices.append((radius*math.cos(angle),radius*math.sin(angle),z))
            rings.append(ring)
    for lower,upper in zip(rings,rings[1:]):
        for i in range(sides):
            j=(i+1)%sides
            if len(lower)==1 and len(upper)>1:faces.append((lower[0],upper[j],upper[i]))
            elif len(upper)==1 and len(lower)>1:faces.append((lower[i],lower[j],upper[0]))
            elif len(lower)>1 and len(upper)>1:faces.append((lower[i],lower[j],upper[j],upper[i]))
    mesh=bpy.data.meshes.new(name);mesh.from_pydata(vertices,[],faces);mesh.update()
    obj=bpy.data.objects.new(name,mesh);bpy.context.collection.objects.link(obj)
    return obj


def material(scan, size):
    m=bpy.data.materials.new('painted_cast_iron');m.use_nodes=True
    n=m.node_tree.nodes;l=m.node_tree.links;sh=n.get('Principled BSDF')
    def image(path, noncolor=False, darken=False):
        im=Image.open(path).convert('RGB');im.thumbnail((size,size),Image.Resampling.LANCZOS)
        if darken:
            # Preserve scanned variation while suggesting an old dark paint coat.
            im=ImageEnhance.Color(im).enhance(.2);im=ImageEnhance.Brightness(im).enhance(.25)
        im.info.clear();out=Path(tempfile.gettempdir())/('lake_lamp_'+path.stem+'_'+str(size)+'.png');im.save(out)
        node=n.new('ShaderNodeTexImage');node.image=bpy.data.images.load(str(out),check_existing=True)
        if noncolor:node.image.colorspace_settings.name='Non-Color'
        return node
    diff=image(scan/'rusty_metal_02_diff_1k.jpg',darken=True)
    normal=image(scan/'rusty_metal_02_nor_gl_1k.png',True)
    arm=image(scan/'rusty_metal_02_arm_1k.png',True)
    l.new(diff.outputs['Color'],sh.inputs['Base Color'])
    nm=n.new('ShaderNodeNormalMap');l.new(normal.outputs['Color'],nm.inputs['Color']);l.new(nm.outputs['Normal'],sh.inputs['Normal'])
    sep=n.new('ShaderNodeSeparateColor');l.new(arm.outputs['Color'],sep.inputs['Color'])
    l.new(sep.outputs['Green'],sh.inputs['Roughness']);l.new(sep.outputs['Blue'],sh.inputs['Metallic'])
    group=bpy.data.node_groups.new('glTF Material Output','ShaderNodeTree');group.interface.new_socket(name='Occlusion',in_out='INPUT',socket_type='NodeSocketFloat')
    g=n.new('ShaderNodeGroup');g.node_tree=group;l.new(sep.outputs['Red'],g.inputs['Occlusion'])
    return m


def main():
    p=argparse.ArgumentParser();p.add_argument('output',type=Path);p.add_argument('--scans',type=Path,required=True);p.add_argument('--lod',action='store_true');a=p.parse_args()
    s3d.empty_scene();s3d.units('m')
    # Stepped plinth, collars and a gentle taper up to the globe support.
    profile=[(0,0),(.24,0),(.24,.06),(.20,.08),(.19,.14),(.16,.17),(.14,.42),(.11,.48),(.105,.58),(.075,.63),(.052,3.08),(.09,3.11),(.10,3.17),(.16,3.20),(.16,3.25),(0,3.25)]
    cap_profile=[(0,3.68),(.11,3.68),(.14,3.72),(.11,3.75),(.045,3.77),(0,3.80)]
    column=lathe_lod('cast_iron_column',profile) if a.lod else s3d.revolve('cast_iron_column',profile)
    cap=lathe_lod('globe_cap',cap_profile) if a.lod else s3d.revolve('globe_cap',cap_profile)
    globe=s3d.at(s3d.sphere('lantern_globe',.46),z=3.27)
    ribs=[]
    for i in range(0 if a.lod else 8):
        theta=2*math.pi*i/8;c,s=math.cos(theta),math.sin(theta)
        ribs.append(s3d.sweep('base_flute',s3d.polygon(4,.016),[(.154*c,.154*s,.18),(.112*c,.112*s,.48)]))
    metal=material(a.scans,512 if a.lod else 1024)
    for obj in (column,cap,*ribs):
        obj.data.materials.clear();obj.data.materials.append(metal)
        bm=bmesh.new();bm.from_mesh(obj.data);bmesh.ops.triangulate(bm,faces=list(bm.faces));bm.to_mesh(obj.data);bm.free()
        bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
        bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.uv.cube_project(cube_size=2);bpy.ops.object.mode_set(mode='OBJECT')
        for face in obj.data.polygons:face.use_smooth=abs(face.normal.z)<.8
    glass=bpy.data.materials.new('lamp_glass');glass.use_nodes=True
    sh=glass.node_tree.nodes.get('Principled BSDF');sh.inputs['Base Color'].default_value=(.65,.48,.25,1);sh.inputs['Roughness'].default_value=.28
    sh.inputs['Emission Color'].default_value=(1,.62,.28,1);sh.inputs['Emission Strength'].default_value=.8
    globe.data.materials.clear();globe.data.materials.append(glass)
    for face in globe.data.polygons:face.use_smooth=True
    # Material groups stay separate; simplify the fine sand3d revolutions to
    # the street-prop budget while keeping a smooth single-globe silhouette.
    bpy.ops.object.select_all(action='DESELECT')
    for obj in (column,cap,*ribs):obj.select_set(True)
    bpy.context.view_layer.objects.active=column;bpy.ops.object.join()
    ratio=.04 if a.lod else .27
    for obj in ((globe,) if a.lod else (column,globe)):
        mod=obj.modifiers.new('budget_decimate','DECIMATE');mod.ratio=ratio
        bpy.context.view_layer.objects.active=obj;bpy.ops.object.modifier_apply(modifier=mod.name)
    root=bpy.data.objects.new('den_ven_ho',None);bpy.context.collection.objects.link(root)
    for obj in (column,globe):obj.parent=root
    a.output.parent.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(a.output.resolve()),export_format='GLB',export_tangents=True)


if __name__=='__main__':main()

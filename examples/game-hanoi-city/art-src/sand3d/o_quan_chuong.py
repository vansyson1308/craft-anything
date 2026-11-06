"""Early Ô Quan Chưởng blockout, metres, Blender Z up / glTF Y up.

The surveyed footprint and opening sizes are approximate. This blockout is for
visual QC: detailed sculpt, weathering, UV/AO bake and in-game review remain.
Run with sand3d 1.6.2, Blender 4.5 Python, and the CC0 scan files described in
the review notes. `--lod` and `--lod2` lower the arch and railing subdivisions.
"""
import argparse
import math
import tempfile
from pathlib import Path
import bpy
import bmesh
from PIL import Image
import sand3d as s3d


def cutter(name, x, radius, spring, depth, steps):
    """A closed arched prism extending below ground for the Boolean cut."""
    outline = [(-radius, -.15), (radius, -.15)]
    outline += [(radius * math.cos(i * math.pi / steps), spring + radius * math.sin(i * math.pi / steps)) for i in range(steps + 1)]
    count = len(outline)
    vertices = [(x+px, -depth/2, z) for px,z in outline] + [(x+px, depth/2, z) for px,z in outline]
    faces = [tuple(reversed(range(count))), tuple(range(count, 2*count))]
    faces += [(i, (i+1)%count, (i+1)%count+count, i+count) for i in range(count)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    return obj


def hip_roof(name, width, depth, eave, rise, segments):
    """Four sloped tiled faces with subtly raised corners."""
    vertices, faces = [], []
    corners = [(-width/2,-depth/2), (width/2,-depth/2), (width/2,depth/2), (-width/2,depth/2)]
    for side in range(4):
        a, b = corners[side], corners[(side+1)%4]
        start = len(vertices)
        for row in range(segments+1):
            t = row/segments
            for col in range(segments+1):
                u = col/segments
                x = ((1-u)*a[0] + u*b[0]) * (1-t)
                y = ((1-u)*a[1] + u*b[1]) * (1-t)
                z = eave + rise*(t**1.6) + .20*(abs(2*u-1)**3)*(1-t)
                vertices.append((x,y,z))
        for row in range(segments):
            for col in range(segments):
                i=start+row*(segments+1)+col
                faces.append((i,i+1,i+segments+2,i+segments+1))
    mesh=bpy.data.meshes.new(name)
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj=bpy.data.objects.new(name,mesh)
    bpy.context.collection.objects.link(obj)
    return obj


def scanned_material(name, root, asset, max_size):
    material=bpy.data.materials.new(name)
    material.use_nodes=True
    nodes=material.node_tree.nodes
    links=material.node_tree.links
    sh=nodes.get('Principled BSDF')
    def tex(suffix, ext, noncolor=False):
        node=nodes.new('ShaderNodeTexImage')
        path=root/(asset+'_'+suffix+'.'+ext)
        if ext == 'png' or max_size < 1024:
            # Normalize PNG metadata for glTF, and use smaller source maps
            # for distant LODs. Unique paths prevent Blender image reuse.
            path=Path(tempfile.gettempdir())/('oquanchuong_'+asset+'_'+suffix+'_'+str(max_size)+'.'+ext)
            image=Image.open(root/(asset+'_'+suffix+'.'+ext)).convert('RGB')
            image.thumbnail((max_size,max_size),Image.Resampling.LANCZOS)
            image.info.clear()
            image.save(path,quality=92 if ext == 'jpg' else None)
        node.image=bpy.data.images.load(str(path.resolve()),check_existing=True)
        if noncolor:node.image.colorspace_settings.name='Non-Color'
        return node
    diffuse=tex('Diffuse','jpg')
    links.new(diffuse.outputs['Color'],sh.inputs['Base Color'])
    normal=tex('nor_gl','png',True)
    nm=nodes.new('ShaderNodeNormalMap')
    links.new(normal.outputs['Color'],nm.inputs['Color'])
    links.new(nm.outputs['Normal'],sh.inputs['Normal'])
    packed=tex('arm','png',True)
    sep=nodes.new('ShaderNodeSeparateColor')
    links.new(packed.outputs['Color'],sep.inputs['Color'])
    links.new(sep.outputs['Green'],sh.inputs['Roughness'])
    links.new(sep.outputs['Blue'],sh.inputs['Metallic'])
    group=bpy.data.node_groups.new('glTF Material Output','ShaderNodeTree')
    group.interface.new_socket(name='Occlusion',in_out='INPUT',socket_type='NodeSocketFloat')
    g=nodes.new('ShaderNodeGroup');g.node_tree=group
    links.new(sep.outputs['Red'],g.inputs['Occlusion'])
    return material


def assign_and_project(objects, material, scale):
    for obj in objects:
        obj.data.materials.clear()
        obj.data.materials.append(material)
        bpy.ops.object.select_all(action='DESELECT')
        obj.select_set(True)
        bpy.context.view_layer.objects.active=obj
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.uv.cube_project(cube_size=scale)
        bpy.ops.object.mode_set(mode='OBJECT')


def main():
    p=argparse.ArgumentParser()
    p.add_argument('output',type=Path)
    p.add_argument('--scans',type=Path,required=True)
    p.add_argument('--lod',action='store_true')
    p.add_argument('--lod2',action='store_true')
    a=p.parse_args()
    s3d.empty_scene();s3d.units('m')
    steps=6 if a.lod2 else 12 if a.lod else 24
    # The map's roughly 11.5 × 17.8 m axis-aligned bounds enclose a rotated
    # footprint. Its minimum rectangle is about 16.66 × 7.89 m, and the
    # adjacent road crosses in the short direction.
    width,depth=16.66,7.89
    wall=s3d.box('three_gate_wall',width,depth,3.8)
    cuts=[cutter('main_opening',0,1.5,1.5,depth+.5,steps),
          cutter('west_opening',-5.65,.825,1.675,depth+.5,steps),
          cutter('east_opening',5.65,.825,1.675,depth+.5,steps)]
    wall=s3d.cut(wall,*cuts)
    # Boolean cuts yield n-gons; triangulate before UVs so Blender can produce
    # tangents for the scanned brick normal map.
    bm=bmesh.new();bm.from_mesh(wall.data)
    bmesh.ops.triangulate(bm,faces=list(bm.faces))
    bm.to_mesh(wall.data);bm.free()
    brick=[wall]
    plaster=[]
    tile=[]
    def box(name,w,d,h,x,y,z,collection):
        obj=s3d.at(s3d.box(name,w,d,h),x=x,y=y,z=z)
        collection.append(obj)
        return obj
    def bar(name,start,end,diameter,collection):
        obj=s3d.sweep(name,s3d.polygon(4,diameter),[start,end])
        collection.append(obj)
        return obj
    box('central_upper_wall',5.65,depth,1.6,0,0,3.8,plaster)
    box('central_cornice',6.0,depth,.18,0,0,5.4,plaster)
    box('central_parapet_floor',5.7,4.8,.24,0,0,5.58,plaster)
    # Perforated railings around the pavilion and the lower side walls.
    for x in (-7.55,-3.75,3.75,7.55):
        box('side_parapet_post',.22,depth,.55,x,0,3.8,plaster)
    for x in (-5.65,5.65):
        for y in (-depth/2+.13,depth/2-.13):
            box('side_parapet_rail',3.8,.18,.13,x,y,4.22,plaster)
            box('side_parapet_base',3.8,.18,.12,x,y,3.8,plaster)
            if not a.lod2:
                n=4 if a.lod else 7
                for j in range(n):
                    left=x-1.75+j*3.5/n;right=x-1.75+(j+1)*3.5/n
                    bar('side_lattice_diagonal',(left,y,3.91),(right,y,4.22),.045,plaster)
                    bar('side_lattice_cross',(left,y,4.22),(right,y,3.91),.045,plaster)
    for x in (-2.45,2.45):
        for y in (-1.95,1.95):box('pavilion_column',.30,.30,1.65,x,y,5.82,plaster)
    for y in (-2.0,2.0):box('pavilion_lintel',5.2,.24,.20,0,y,7.36,plaster)
    for x in (-2.55,2.55):box('pavilion_lintel_side',.24,4.2,.20,x,0,7.36,plaster)
    for y in (-2.0,2.0):
        box('pavilion_balustrade',4.6,.12,.10,0,y,6.22,plaster)
        if not a.lod2:
            n=4 if a.lod else 8
            for j in range(n):
                left=-2.25+j*4.5/n;right=-2.25+(j+1)*4.5/n
                bar('pavilion_lattice',(left,y,5.91),(right,y,6.22),.04,plaster)
                bar('pavilion_lattice_cross',(left,y,6.22),(right,y,5.91),.04,plaster)
    roof=hip_roof('four_sided_tile_roof',6.2,5.25,7.54,1.1,2 if a.lod2 else 4 if a.lod else 7)
    tile.append(roof)
    for x in (-3.1,3.1):
        for y in (-2.625,2.625):
            bar('raised_roof_corner',(x,y,7.74),(x*1.08,y*1.08,8.08),.11,tile)
    # Decorative slab over the central opening; lettering is deliberately blank.
    box('blank_gate_plaque',1.4,.08,.5,0,-depth/2-.05,4.15,plaster)
    max_size=256 if a.lod2 else 512 if a.lod else 1024
    brick_mat=scanned_material('old_red_brick',a.scans,'medieval_red_brick',max_size)
    roof_mat=scanned_material('aged_roof_tile',a.scans,'roof_tiles',max_size)
    plaster_mat=scanned_material('aged_mossy_plaster',a.scans,'worn_mossy_plasterwall',max_size)
    # Normal-mapped faces need triangles before projection and tangent export.
    for obj in plaster + tile:
        bm=bmesh.new();bm.from_mesh(obj.data)
        bmesh.ops.triangulate(bm,faces=list(bm.faces))
        bm.to_mesh(obj.data);bm.free()
    assign_and_project(brick,brick_mat,2.0)
    assign_and_project(tile,roof_mat,2.0)
    assign_and_project(plaster,plaster_mat,2.0)
    # Combine only objects sharing a material to keep draws bounded.
    assembled=[]
    for name,objects in [('brickwork',brick),('plaster',plaster),('roof',tile)]:
        bpy.ops.object.select_all(action='DESELECT')
        for o in objects:o.select_set(True)
        bpy.context.view_layer.objects.active=objects[0]
        if len(objects)>1:bpy.ops.object.join()
        o=bpy.context.view_layer.objects.active;o.name=name;assembled.append(o)
    root=bpy.data.objects.new('o_quan_chuong',None);bpy.context.collection.objects.link(root)
    for o in assembled:
        world=o.matrix_world.copy();o.parent=root;o.matrix_world=world
    root.rotation_euler.z=-math.atan2(55,12)
    a.output.parent.mkdir(parents=True,exist_ok=True)
    bpy.ops.export_scene.gltf(filepath=str(a.output.resolve()),export_format='GLB',export_tangents=True)


if __name__=='__main__':main()

"""Experimental transfer to the game's rig; output is not ready for delivery."""
import bpy,sys,bmesh
from pathlib import Path
from mathutils import Matrix

root=Path(sys.argv[1]);source=Path(sys.argv[2]);out=root/'stocky_transferred.glb'
bpy.ops.wm.open_mainfile(filepath=str(root/'stocky_posed.blend'))
dg=bpy.context.evaluated_depsgraph_get();new=[]
for old in list(bpy.context.scene.objects):
    if old.type!='MESH' or not old.name.startswith('Human'):continue
    ev=old.evaluated_get(dg);mesh=bpy.data.meshes.new_from_object(ev,preserve_all_data_layers=True,depsgraph=dg)
    mesh.transform(ev.matrix_world);obj=bpy.data.objects.new('study_'+old.name,mesh);bpy.context.collection.objects.link(obj);new.append(obj)
for obj in list(bpy.context.scene.objects):
    if obj not in new:bpy.data.objects.remove(obj,do_unlink=True)
bpy.ops.import_scene.gltf(filepath=str(source.resolve()))
rig=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE');donor=next(o for o in bpy.context.scene.objects if o.type=='MESH' and len(o.vertex_groups)==65)
print('rig',rig.name,'bones',len(rig.data.bones),'donor',donor.name,flush=True)
for obj in new:
    # Evaluated MPFB meshes keep their old rig and helper group definitions.
    # Clear those before adding the game's groups, or normalization will
    # retain unrelated weights and the exporter will invent a neutral bone.
    for group in list(obj.vertex_groups):obj.vertex_groups.remove(group)
    for group in donor.vertex_groups:obj.vertex_groups.new(name=group.name)
    mod=obj.modifiers.new('transfer','DATA_TRANSFER');mod.object=donor;mod.use_vert_data=True;mod.data_types_verts={'VGROUP_WEIGHTS'}
    mod.vert_mapping='POLYINTERP_NEAREST';mod.layers_vgroup_select_src='ALL';mod.layers_vgroup_select_dst='NAME'
    bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
    bpy.ops.object.modifier_apply(modifier=mod.name)
    missing=sum(1 for v in obj.data.vertices if not v.groups)
    print(obj.name,'vertices',len(obj.data.vertices),'unweighted',missing,flush=True)
    if missing:raise ValueError('Transfer produced unweighted vertices')
    if obj.name in ('study_Human','study_Human.short01'):
        decimate=obj.modifiers.new('review_budget','DECIMATE');decimate.ratio=.74 if obj.name=='study_Human' else .75
        bpy.ops.object.modifier_apply(modifier=decimate.name)
    # Evaluated MPFB meshes can retain orphaned source-group indices. Remove
    # them before testing weights; otherwise Blender invents a neutral bone.
    bm=bmesh.new();bm.from_mesh(obj.data);layer=bm.verts.layers.deform.active
    if layer:
        for v in bm.verts:
            weights=v[layer]
            for index in list(weights.keys()):
                if index>=len(obj.vertex_groups):del weights[index]
        bm.to_mesh(obj.data)
    bm.free()
    candidates=[b for b in rig.data.bones if b.name in {'root','pelvis','spine_01','spine_02','spine_03','neck_01','Head','upperarm_l','upperarm_r','lowerarm_l','lowerarm_r','hand_l','hand_r','thigh_l','thigh_r','calf_l','calf_r','foot_l','foot_r'}]
    from mathutils.geometry import intersect_point_line
    fallback=0
    for v in obj.data.vertices:
        if sum(g.weight for g in v.groups)<1e-6:
            point=obj.matrix_world@v.co
            def distance(b):
                a=rig.matrix_world@b.head_local;z=rig.matrix_world@b.tail_local;closest,t=intersect_point_line(point,a,z);closest=a.lerp(z,max(0,min(1,t)))
                return (point-closest).length_squared
            bone=min(candidates,key=distance);obj.vertex_groups[bone.name].add([v.index],1.0,'REPLACE');fallback+=1
    print(obj.name,'nearest-bone fallbacks',fallback,flush=True)
    bpy.ops.object.vertex_group_limit_total(group_select_mode='ALL',limit=4)
    bpy.ops.object.vertex_group_normalize_all(group_select_mode='ALL',lock_active=False)
    zero=[v.index for v in obj.data.vertices if sum(g.weight for g in v.groups)<1e-6]
    print(obj.name,'after normalization unweighted',len(zero),flush=True)
    if zero:raise ValueError('Vertices have no positive weights')
    # Keep skinned meshes at scene root while the modifier references the rig.
    obj.parent=None;obj.modifiers.new('Armature','ARMATURE').object=rig
bpy.data.objects.remove(donor,do_unlink=True)
# Unbranded cap and a simple cloth face covering, both weighted to the head.
fabric=bpy.data.materials.new('trom2_dark_fabric');fabric.diffuse_color=(.018,.02,.022,1);fabric.use_nodes=True
shader=fabric.node_tree.nodes.get('Principled BSDF');shader.inputs['Base Color'].default_value=(.018,.02,.022,1);shader.inputs['Roughness'].default_value=.85
details=[]
bpy.ops.mesh.primitive_uv_sphere_add(segments=16,ring_count=8,radius=1,location=(0,-.005,1.785));cap=bpy.context.object;cap.name='cap_crown';cap.scale=(.094,.105,.060)
bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);details.append(cap)
bpy.ops.mesh.primitive_cube_add(size=1,location=(0,-.144,1.738));brim=bpy.context.object;brim.name='cap_brim';brim.scale=(.17,.09,.012)
bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);details.append(brim)
verts=[(-.075,-.135,1.655),(0,-.180,1.655),(.075,-.135,1.655),(-.075,-.135,1.585),(0,-.180,1.585),(.075,-.135,1.585),(-.09,-.035,1.64),(-.09,-.035,1.60),(.09,-.035,1.64),(.09,-.035,1.60)]
faces=[(0,3,4,1),(1,4,5,2),(6,7,3,0),(2,5,9,8)]
mesh=bpy.data.meshes.new('face_covering');mesh.from_pydata(verts,[],faces);mesh.update();mask=bpy.data.objects.new('face_covering',mesh);bpy.context.collection.objects.link(mask);details.append(mask)
for obj in details:
    obj.data.materials.clear();obj.data.materials.append(fabric)
    group=obj.vertex_groups.new(name='Head');group.add(list(range(len(obj.data.vertices))),1.0,'REPLACE')
    obj.modifiers.new('Armature','ARMATURE').object=rig;new.append(obj)
bpy.ops.object.select_all(action='DESELECT');rig.select_set(True)
for obj in new:obj.select_set(True)
bpy.context.view_layer.objects.active=rig
bpy.ops.export_scene.gltf(filepath=str(out),export_format='GLB',use_selection=True,export_tangents=True,export_extras=True)
print('WROTE',out,flush=True)

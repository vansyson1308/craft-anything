import bpy,sys
from pathlib import Path
from mathutils import Vector
root=Path(sys.argv[1]);library=Path(sys.argv[2]);bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(root/'stocky_atlas.glb'));rig=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE');before=set(bpy.context.scene.objects)
bpy.ops.import_scene.gltf(filepath=str(library.resolve()))
for o in list(bpy.context.scene.objects):
    if o not in before:bpy.data.objects.remove(o,do_unlink=True)
s=bpy.context.scene;s.render.engine='CYCLES';s.cycles.samples=16;s.render.resolution_x=650;s.render.resolution_y=650
s.world=bpy.data.worlds.new('review');s.world.use_nodes=True;s.world.node_tree.nodes['Background'].inputs['Strength'].default_value=.5
bpy.ops.object.camera_add(location=(3,-5,2.9));c=bpy.context.object;c.rotation_euler=(Vector((0,0,.85))-c.location).to_track_quat('-Z','Y').to_euler();c.data.type='ORTHO';c.data.ortho_scale=2.45;s.camera=c
bpy.ops.object.light_add(type='AREA',location=(3,-4,5));bpy.context.object.data.energy=700;bpy.context.object.data.size=4
rig.animation_data_create()
for name,frame in [('Idle_Loop',20),('Sprint_Loop',4),('Crouch_Fwd_Loop',12),('Punch_Cross',10),('Roll',12),('Death01',32)]:
    if name not in bpy.data.actions:raise ValueError('Missing action '+name)
    action=bpy.data.actions[name];rig.animation_data.action=action
    rig.animation_data.action_slot=action.slots[0]
    s.frame_set(frame);s.render.filepath=str(root/f'action_{name}.png');bpy.ops.render.render(write_still=True)

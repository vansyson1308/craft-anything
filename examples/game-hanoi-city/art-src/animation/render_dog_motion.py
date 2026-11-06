"""Render a few still poses from the draft dog motion GLB for visual QC."""
import argparse
from pathlib import Path
import bpy
from mathutils import Vector

p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True);bpy.context.scene.render.fps=30
bpy.ops.import_scene.gltf(filepath=str(a.source.resolve()))
arm=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE')
dog=next(o for o in bpy.context.scene.objects if o.type=='MESH' and o.vertex_groups)
points=[dog.matrix_world@v.co for v in dog.data.vertices]
lo=Vector(tuple(min(v[i] for v in points) for i in range(3)));hi=Vector(tuple(max(v[i] for v in points) for i in range(3)))
centre=(lo+hi)/2;span=max(hi-lo)
s=bpy.context.scene;s.render.engine='CYCLES';s.cycles.samples=24;s.render.resolution_x=800;s.render.resolution_y=650
s.world=bpy.data.worlds.new('review');s.world.use_nodes=True;s.world.node_tree.nodes['Background'].inputs['Strength'].default_value=.5
bpy.ops.object.camera_add(location=centre+Vector((span*1.8,-span*1.8,span*.95)));c=bpy.context.object;c.rotation_euler=(centre-c.location).to_track_quat('-Z','Y').to_euler();c.data.type='ORTHO';c.data.ortho_scale=span*1.65;s.camera=c
bpy.ops.object.light_add(type='AREA',location=centre+Vector((span,-span*2,span*3)));bpy.context.object.data.energy=400;bpy.context.object.data.size=span*3
a.output.mkdir(parents=True,exist_ok=True)
for clip,frames in [('Wag',[0,4,11,19]),('Jump',[0,10,15,25])]:
    arm.animation_data.action=bpy.data.actions[clip]
    for frame in frames:
        s.frame_set(frame);s.render.filepath=str((a.output/f'{clip.lower()}_{frame:02d}.png').resolve());bpy.ops.render.render(write_still=True)

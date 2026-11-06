"""Cycles front and diagonal renders of the early gate model."""
import argparse
import math
from pathlib import Path
import bpy
from mathutils import Matrix, Vector

p=argparse.ArgumentParser();p.add_argument('model',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
a.output.mkdir(parents=True,exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(a.model.resolve()))
s=bpy.context.scene;s.render.engine='CYCLES';s.cycles.samples=32;s.render.resolution_x=1200;s.render.resolution_y=800
s.world=bpy.data.worlds.new('review');s.world.use_nodes=True;s.world.node_tree.nodes['Background'].inputs['Strength'].default_value=.45
rotation=Matrix.Rotation(-math.atan2(55,12),4,'Z')
bpy.ops.object.camera_add();c=bpy.context.object;c.data.type='ORTHO';s.camera=c
bpy.ops.object.light_add(type='AREA',location=rotation@Vector((0,-15,22)));bpy.context.object.data.energy=4000;bpy.context.object.data.size=15
for name,position,scale in [('front',(0,-35,11),20.0),('diagonal',(26,-31,21),25.0)]:
    c.location=rotation@Vector(position);c.data.ortho_scale=scale;c.rotation_euler=(Vector((0,0,3.6))-c.location).to_track_quat('-Z','Y').to_euler()
    s.render.filepath=str((a.output/(name+'.png')).resolve());bpy.ops.render.render(write_still=True)

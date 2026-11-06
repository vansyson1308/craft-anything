"""Cycles review of the lakeside lamp, optionally with world and lights off."""
import argparse
from pathlib import Path
import bpy
from mathutils import Vector

p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);p.add_argument('--dark',action='store_true');a=p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(a.source.resolve()))
s=bpy.context.scene;s.render.engine='CYCLES';s.cycles.samples=32;s.render.resolution_x=700;s.render.resolution_y=900
s.world=bpy.data.worlds.new('review');s.world.use_nodes=True;s.world.node_tree.nodes['Background'].inputs['Strength'].default_value=0 if a.dark else .4
bpy.ops.object.camera_add(location=(5,-8,4.5));camera=bpy.context.object;camera.rotation_euler=(Vector((0,0,1.85))-camera.location).to_track_quat('-Z','Y').to_euler();camera.data.type='ORTHO';camera.data.ortho_scale=4.4;s.camera=camera
if not a.dark:
    bpy.ops.object.light_add(type='AREA',location=(3,-4,6));bpy.context.object.data.energy=700;bpy.context.object.data.size=5
s.render.filepath=str(a.output.resolve());bpy.ops.render.render(write_still=True)

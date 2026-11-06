import argparse
from pathlib import Path
import bpy
from mathutils import Vector
p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('output',type=Path);p.add_argument('--dark',action='store_true');a=p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.gltf(filepath=str(a.source.resolve()))
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=32;scene.render.resolution_x=800;scene.render.resolution_y=900
scene.world=bpy.data.worlds.new('review_world');scene.world.use_nodes=True;scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value=0 if a.dark else .3
bpy.ops.object.camera_add(location=(1,-2,.36));camera=bpy.context.object;camera.rotation_euler=(Vector((0,0,-.32))-camera.location).to_track_quat('-Z','Y').to_euler();camera.data.type='ORTHO';camera.data.ortho_scale=.85;scene.camera=camera
if not a.dark:
    bpy.ops.object.light_add(type='AREA',location=(1,-2,2.36));bpy.context.object.data.energy=100;bpy.context.object.data.size=3
scene.render.filepath=str(a.output.resolve());bpy.ops.render.render(write_still=True)

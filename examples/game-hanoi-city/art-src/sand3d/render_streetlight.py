"""Close Cycles inspection of a streetlight head, with optional zero lighting."""
import argparse
from pathlib import Path
import bpy
from mathutils import Vector

p = argparse.ArgumentParser()
p.add_argument('source', type=Path)
p.add_argument('output', type=Path)
p.add_argument('--dark', action='store_true')
args = p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
s = bpy.context.scene
s.render.engine = 'CYCLES'
s.cycles.samples = 32
s.render.resolution_x, s.render.resolution_y = 800, 900
s.world = bpy.data.worlds.new('review')
s.world.use_nodes = True
s.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0 if args.dark else .5
bpy.ops.object.camera_add(location=(3, -5, 5.8))
camera = bpy.context.object
camera.rotation_euler = (Vector((0, -.85, 4.48)) - camera.location).to_track_quat('-Z', 'Y').to_euler()
camera.data.type = 'ORTHO'
camera.data.ortho_scale = 1.5
s.camera = camera
if not args.dark:
    bpy.ops.object.light_add(type='AREA', location=(1, -3, 7))
    bpy.context.object.data.energy = 200
    bpy.context.object.data.size = 4
s.render.filepath = str(args.output.resolve())
bpy.ops.render.render(write_still=True)

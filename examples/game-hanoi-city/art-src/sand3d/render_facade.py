"""Render an upper facade for material review in Cycles."""
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
s.cycles.samples = 24
s.render.resolution_x, s.render.resolution_y = 900, 750
s.world = bpy.data.worlds.new('review')
s.world.use_nodes = True
s.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0 if args.dark else .5
bpy.ops.object.camera_add(location=(4, -7, 4))
camera = bpy.context.object
camera.rotation_euler = (Vector((2, 0, 1.65)) - camera.location).to_track_quat('-Z', 'Y').to_euler()
camera.data.type = 'ORTHO'
camera.data.ortho_scale = 5
s.camera = camera
if not args.dark:
    bpy.ops.object.light_add(type='AREA', location=(4, -5, 8))
    bpy.context.object.data.energy = 1000
    bpy.context.object.data.size = 6
s.render.filepath = str(args.output.resolve())
bpy.ops.render.render(write_still=True)

"""Render a close QC view of the temporary tie on the closed cage door."""
import argparse
from pathlib import Path
import bpy
from mathutils import Vector

p = argparse.ArgumentParser()
p.add_argument('model', type=Path)
p.add_argument('output', type=Path)
args = p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.model.resolve()))
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.samples = 64
scene.render.resolution_x = scene.render.resolution_y = 800
scene.world = bpy.data.worlds.new('Review World')
scene.world.color = (.25, .25, .25)
bpy.ops.object.camera_add(location=(.9, -1.4, .75))
camera = bpy.context.object
camera.rotation_euler = (Vector((.36, -.35, .465)) - camera.location).to_track_quat('-Z', 'Y').to_euler()
camera.data.type = 'ORTHO'
camera.data.ortho_scale = .22
scene.camera = camera
bpy.ops.object.light_add(type='AREA', location=(1, -2, 3))
bpy.context.object.data.energy = 250
bpy.context.object.data.size = 3
scene.render.filepath = str(args.output.resolve())
bpy.ops.render.render(write_still=True)

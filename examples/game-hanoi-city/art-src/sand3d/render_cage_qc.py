"""Render review views from the final GLB using Blender's Cycles renderer."""
import argparse
import math
from pathlib import Path
import bpy
from mathutils import Vector

p = argparse.ArgumentParser()
p.add_argument('model', type=Path)
p.add_argument('output', type=Path)
args = p.parse_args()
args.output.mkdir(parents=True, exist_ok=True)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.model.resolve()))
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.samples = 32
scene.render.resolution_x = 1100
scene.render.resolution_y = 900
scene.world = bpy.data.worlds.new('Review World')
scene.world.color = (.25, .25, .25)
bpy.ops.object.camera_add()
camera = bpy.context.object
camera.data.type = 'ORTHO'
camera.data.ortho_scale = 1.45
scene.camera = camera
bpy.ops.object.light_add(type='AREA', location=(1, -2, 3))
bpy.context.object.data.energy = 250
bpy.context.object.data.shape = 'DISK'
bpy.context.object.data.size = 3
hinge = bpy.data.objects['door_hinge']
for name, position, angle in [
    ('front', (0, -2.4, .90), 0),
    ('back', (0, 2.4, .90), 0),
    ('diagonal', (1.7, -2.3, 1.5), 0),
    ('open_110', (1.7, -2.3, 1.5), -110),
]:
    hinge.rotation_mode = 'XYZ'
    hinge.rotation_euler.z = math.radians(angle)
    camera.location = position
    camera.data.ortho_scale = 2.0 if angle else 1.45
    target = Vector((-.35, -.12, .36)) if angle else Vector((0, 0, .36))
    camera.rotation_euler = (target - camera.location).to_track_quat('-Z', 'Y').to_euler()
    scene.render.filepath = str((args.output / ('cage_qc_' + name + '.png')).resolve())
    bpy.ops.render.render(write_still=True)

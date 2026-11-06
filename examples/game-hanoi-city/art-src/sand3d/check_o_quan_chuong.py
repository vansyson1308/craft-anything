"""Check that each gate opening remains clear in an exported GLB."""
import argparse
import math
from pathlib import Path
import bpy
from mathutils import Matrix, Vector

p=argparse.ArgumentParser()
p.add_argument('model',type=Path)
args=p.parse_args()
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.model.resolve()))
depsgraph=bpy.context.evaluated_depsgraph_get()
scene=bpy.context.scene
rotation=Matrix.Rotation(-math.atan2(55,12),4,'Z')
for x,z,should_open in [(0,1.5,True),(0,2.9,True),(0,3.1,False),
                        (-5.65,1.5,True),(5.65,1.5,True),(-5.65,2.4,True),(5.65,2.4,True),(3,1.5,False)]:
    origin=rotation@Vector((x,-4.5,z))
    direction=rotation.to_3x3()@Vector((0,1,0))
    hit,*_=scene.ray_cast(depsgraph,origin,direction,distance=9)
    if (not hit)!=should_open:
        raise AssertionError(f'Unexpected passage result at x={x}, z={z}: hit={hit}')
print('Eight opening and wall ray checks passed:',args.model)

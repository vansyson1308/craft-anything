"""Measure UV-island texel density on a GLB body using world-space area."""
import argparse
import json
import math
import sys
from pathlib import Path
import bpy
import bmesh
from bpy_extras.mesh_utils import mesh_linked_uv_islands

parser = argparse.ArgumentParser()
parser.add_argument('source', type=Path)
parser.add_argument('--size', type=int, default=512)
parser.add_argument('--target', type=float, default=512)
args = parser.parse_args(sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else None)
bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
islands = []
for obj in [o for o in bpy.context.scene.objects if o.type == 'MESH' and o.name != 'inner_bulb']:
    mesh = obj.data
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-7)
    bm.to_mesh(mesh)
    bm.free()
    uv = mesh.uv_layers.active.data
    mesh.calc_loop_triangles()
    by_polygon = {}
    for tri in mesh.loop_triangles:
        by_polygon.setdefault(tri.polygon_index, []).append(tri)
    for polygons in mesh_linked_uv_islands(mesh):
        world_area = uv_area = 0.0
        for polygon in polygons:
            for tri in by_polygon.get(polygon, []):
                p = [obj.matrix_world @ mesh.vertices[i].co for i in tri.vertices]
                u = [uv[i].uv for i in tri.loops]
                world_area += (p[1] - p[0]).cross(p[2] - p[0]).length / 2
                uv_area += abs((u[1].x-u[0].x)*(u[2].y-u[0].y) - (u[1].y-u[0].y)*(u[2].x-u[0].x)) / 2
        if world_area > 1e-12:
            islands.append({'area_m2': world_area, 'uv_area': uv_area,
                            'density_px_per_m': args.size * math.sqrt(uv_area / world_area)})
total = sum(x['area_m2'] for x in islands)
valid = sum(x['area_m2'] for x in islands if args.target * .85 <= x['density_px_per_m'] <= args.target * 1.15)
summary = {'file': str(args.source), 'atlas_size': args.size, 'island_count': len(islands),
           'world_area_m2': total, 'target_px_per_m': args.target,
           'within_target_plus_minus_15_percent_area_fraction': valid / total if total else 0,
           'islands': sorted(islands, key=lambda x: x['area_m2'], reverse=True)}
print('UV_DENSITY_JSON:', json.dumps(summary))

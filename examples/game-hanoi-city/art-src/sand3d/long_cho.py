"""Editable cage geometry. Run with sand3d 1.6.2 / Blender 4.5 Python.

The finish pipeline supplies the UV, scanned material and bakes.
Dimensions below are expressed in millimetres and converted to metres before
construction. Blender Z is up; glTF export converts to Y-up.
"""

import argparse
import math
from pathlib import Path

import bpy  # Initializes Blender's Python modules before importing bmesh.
import bmesh
from mathutils import Vector
import sand3d as s3d


def build_cage(lod=False, high_poly=False):
    s3d.empty_scene()
    s3d.units("m")
    fixed = []

    def bar(name, start, end, diameter=7):
        # The rail centres in the drawing start 18 mm above the ground; move
        # the complete assembly down 9 mm so its lowest surface meets z = 0.
        points = [(x / 1000, y / 1000, (z - 9) / 1000) for x, y, z in (start, end)]
        # Eight sides preserve a rounded silhouette on thin rods while staying
        # within the street-prop triangle budget.
        sides = 32 if high_poly else 4 if lod else 8
        obj = s3d.sweep(name, s3d.polygon(sides, diameter / 1000), points)
        # Structural rods terminate inside another rail. Their hidden end caps
        # waste UV area and triangles; retain visible hinge, latch and shackle caps.
        if not name.startswith(("hinge_", "latch", "padlock_")):
            direction = (Vector(points[1]) - Vector(points[0])).normalized()
            bm = bmesh.new()
            bm.from_mesh(obj.data)
            bm.normal_update()
            caps = [face for face in bm.faces if abs(face.normal.dot(direction)) > 0.98]
            bmesh.ops.delete(bm, geom=caps, context="FACES")
            bm.to_mesh(obj.data)
            bm.free()
        s3d.material(obj, "painted_metal", color="#4b4a40", roughness=0.78, metallic=0.75)
        return obj

    # External footprint 900 x 700 mm; ground-level pivot. Four rails and
    # corner uprights are kept as distinct named parts for later texturing.
    for z, label in ((18, "bottom"), (682, "top")):
        for y, side in ((-341, "front"), (341, "back")):
            fixed.append(bar(f"{label}_{side}", (-441, y, z), (441, y, z), 18))
        for x, side in ((-441, "left"), (441, "right")):
            fixed.append(bar(f"{label}_{side}", (x, -341, z), (x, 341, z), 18))
    posts = {}
    for x, side in ((-441, "left"), (441, "right")):
        for y, face in ((-341, "front"), (341, "back")):
            obj = bar(f"post_{side}_{face}", (x, y, 18), (x, y, 682), 18)
            fixed.append(obj)
            posts[(side, face)] = obj

    # Rear and side bars, with intermediate horizontal rails.
    back_count = 3 if lod else 7
    for i in range(1, back_count + 1):
        x = -441 + i * 882 / (back_count + 1)
        fixed.append(bar(f"back_vertical_{i:02}", (x, 341, 18), (x, 341, 682)))
    for side, x in (("left", -441), ("right", 441)):
        side_count = 2 if lod else 7
        for i in range(1, side_count + 1):
            y = -341 + i * 682 / (side_count + 1)
            fixed.append(bar(f"{side}_vertical_{i:02}", (x, y, 18), (x, y, 682)))
        if not lod:
            fixed.append(bar(f"{side}_middle", (x, -341, 350), (x, 341, 350)))
    fixed.append(bar("back_middle", (-441, 341, 350), (441, 341, 350)))
    for z, face in ((18, "floor"), (682, "roof")):
        cross_count = 2 if lod else 7
        for i in range(1, cross_count + 1):
            x = -441 + i * 882 / (cross_count + 1)
            fixed.append(bar(f"{face}_bar_{i:02}", (x, -341, z), (x, 341, z)))

    # A shallow tray and two overlapping sheets of crumpled old newspaper.
    # The paper's ink and stains are authored into the shared atlas, not logos.
    tray = s3d.at(s3d.box("tray_floor", .836, .616, .012), z=.012)
    paper = s3d.at(s3d.box("paper_main", .695, .50, .002), x=-.025, y=.015, z=.025 if lod else .034)
    scrap = s3d.at(s3d.box("paper_scrap", .23, .32, .002), x=.25, y=-.10, z=.027 if lod else .046, rz=8)
    if not lod:
        for sheet in (paper, scrap):
            bm = bmesh.new()
            bm.from_mesh(sheet.data)
            bmesh.ops.subdivide_edges(bm, edges=list(bm.edges), cuts=3, use_grid_fill=True)
            for vertex in bm.verts:
                x, y = vertex.co.x, vertex.co.y
                vertex.co.z += .008 * math.sin(34*x + 19*y) + .004 * math.sin(67*y - 11*x)
            bm.normal_update()
            bm.to_mesh(sheet.data)
            bm.free()
    for obj in (tray, paper, scrap):
        s3d.material(obj, "painted_metal", color="#4b4a40", roughness=.78, metallic=.75)
        fixed.append(obj)
    # Twine temporarily knots the right front corner, with a loose hanging end.
    fixed.extend([
        bar("rope_front", (422, -357, 480), (449, -357, 480), 3),
        bar("rope_side", (449, -357, 480), (449, -325, 480), 3),
        bar("rope_back", (449, -325, 480), (422, -325, 480), 3),
        bar("rope_end", (449, -357, 480), (449, -357, 390), 3),
    ])

    # Front doorway: outer frame is fixed; door rotates around its left edge.
    for x, side in ((-360, "left"), (360, "right")):
        fixed.append(bar(f"doorway_{side}", (x, -341, 18), (x, -341, 682), 14))
    for z, label in ((140, "lower"), (560, "upper")):
        fixed.append(bar(f"hinge_{label}", (-355, -350, z - 24), (-355, -350, z + 24), 22))
    door_parts = []
    for z, label in ((65, "bottom"), (635, "top")):
        door_parts.append(bar(f"door_{label}", (-355, -350, z), (355, -350, z), 12))
    for x, side in ((-355, "left"), (355, "right")):
        door_parts.append(bar(f"door_{side}", (x, -350, 65), (x, -350, 635), 12))
    door_count = 3 if lod else 7
    for i in range(1, door_count + 1):
        x = -355 + i * 710 / (door_count + 1)
        door_parts.append(bar(f"door_vertical_{i:02}", (x, -350, 65), (x, -350, 635)))
    door_parts.append(bar("door_middle", (-355, -350, 350), (355, -350, 350)))
    door_parts.append(bar("latch", (310, -363, 330), (375, -363, 330), 13))
    # The padlock hangs from the latch and belongs to the moving door.
    lock = s3d.at(s3d.box("padlock_body", 0.034, 0.018, 0.038), x=0.336, y=-0.374, z=0.261)
    s3d.material(lock, "painted_metal", color="#4b4a40", roughness=0.78, metallic=0.75)
    door_parts.append(lock)
    door_parts.extend([
        bar("padlock_shackle_left", (326, -374, 307), (326, -374, 320), 5),
        bar("padlock_shackle_top", (326, -374, 320), (346, -374, 320), 5),
        bar("padlock_shackle_right", (346, -374, 320), (346, -374, 307), 5),
    ])
    door = s3d.group("door", *door_parts)
    # Rotating clockwise about Blender Z opens toward the outside (-Y).
    # Wire geometry is in scene coordinates, so the post's local origin is
    # the scene origin, not the post centre.
    hinge = s3d.local(posts[("left", "front")], -0.355, -0.350, -0.009)
    s3d.joint("door_hinge", door, hinge, axis="-Z", range=(0, 110))
    return s3d.assembly("long_cho", *fixed, door)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path, help="Draft GLB output path")
    parser.add_argument("--lod", action="store_true", help="Build the simplified distant cage")
    parser.add_argument("--high-poly", action="store_true", help="Build the high-resolution bake source")
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    s3d.export(build_cage(lod=args.lod, high_poly=args.high_poly), str(args.output))

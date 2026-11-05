"""QC 05 cage: single PBR atlas, selected-to-active normals and AO.

Requires bpy 4.5, numpy, Pillow and rectpack. Input GLBs come from long_cho.py;
the input scan is Poly Haven rusty_metal_02, listed in assets/hq/SOURCES.md.
"""
import argparse
import math
from pathlib import Path

import bpy
import numpy as np
from PIL import Image, ImageDraw, ImageFilter
from unwrap_cage import unwrap


def masks(objects, size):
    result = {kind: Image.new("L", (size, size), 0) for kind in ("paper", "rope", "tray", "vertical")}
    for obj in objects:
        name = obj.name
        kind = ("paper" if name.startswith("paper_") else "rope" if name.startswith("rope_") else
                "tray" if name.startswith("tray_") else "vertical" if
                ("vertical_" in name or name.startswith(("post_", "doorway_", "door_left", "door_right"))) else None)
        if kind is None:
            continue
        uv = obj.data.uv_layers.active.data
        draw = ImageDraw.Draw(result[kind])
        for poly in obj.data.polygons:
            pts = [(round(uv[i].uv.x * (size - 1)), round((1 - uv[i].uv.y) * (size - 1))) for i in poly.loop_indices]
            draw.polygon(pts, fill=255)
    return result


def paper_texture(size):
    """Original generic printed paper with dirt; no actual names or logos."""
    rng = np.random.default_rng(5026)
    y, x = np.mgrid[0:size, 0:size]
    grain = rng.normal(0, 6, (size, size)) + 5 * np.sin(x * .045 + y * .017)
    stains = np.zeros((size, size), dtype=np.float32)
    for _ in range(500):
        cx, cy = rng.integers(0, size, 2)
        rad = rng.uniform(size * .003, size * .018)
        stains += rng.uniform(25, 95) * np.exp(-((x-cx)**2 + (y-cy)**2) / (2 * rad**2))
    base = np.stack((210 + grain - stains, 199 + grain - stains, 174 + grain - stains), axis=-1)
    image = Image.fromarray(np.uint8(np.clip(base, 0, 255)), "RGB")
    draw = ImageDraw.Draw(image)
    # Lines imitate print texture, not legible text or a real newspaper.
    columns = 10
    for col in range(columns):
        left = col * size // columns + 8
        right = (col + 1) * size // columns - 8
        for yy in range(8, size - 5, max(7, size // 100)):
            end = int(rng.integers(max(left + 2, right - 25), max(left + 3, right)))
            color = int(rng.integers(65, 115))
            draw.line((left, yy, end, yy), fill=(color, color, max(0, color - 6)), width=1)
    return np.asarray(image)


def rust_flow(objects, size):
    """Rasterize downward drips in object space, independent of atlas rotation."""
    result = np.zeros((size, size), dtype=np.float32)
    bpy.context.view_layer.update()
    for obj in objects:
        name = obj.name
        if not ("vertical_" in name or name.startswith(("post_", "doorway_", "door_left", "door_right"))):
            continue
        mesh, transform = obj.data, obj.matrix_world
        uv = mesh.uv_layers.active.data
        world = [transform @ vertex.co for vertex in mesh.vertices]
        cx = (min(p.x for p in world) + max(p.x for p in world)) * .5
        cy = (min(p.y for p in world) + max(p.y for p in world)) * .5
        phase = sum(ord(c) for c in name) * .31
        for poly in mesh.polygons:
            if len(poly.loop_indices) != 3:
                continue
            loop_ids = list(poly.loop_indices)
            p = np.array([(uv[i].uv.x * (size - 1), (1 - uv[i].uv.y) * (size - 1)) for i in loop_ids])
            xmin, xmax = max(0, int(np.floor(p[:, 0].min()))), min(size - 1, int(np.ceil(p[:, 0].max())))
            ymin, ymax = max(0, int(np.floor(p[:, 1].min()))), min(size - 1, int(np.ceil(p[:, 1].max())))
            if xmax < xmin or ymax < ymin:
                continue
            x, y = np.meshgrid(np.arange(xmin, xmax + 1) + .5, np.arange(ymin, ymax + 1) + .5)
            den = (p[1, 1] - p[2, 1]) * (p[0, 0] - p[2, 0]) + (p[2, 0] - p[1, 0]) * (p[0, 1] - p[2, 1])
            if abs(den) < 1e-9:
                continue
            a = ((p[1, 1] - p[2, 1]) * (x - p[2, 0]) + (p[2, 0] - p[1, 0]) * (y - p[2, 1])) / den
            b = ((p[2, 1] - p[0, 1]) * (x - p[2, 0]) + (p[0, 0] - p[2, 0]) * (y - p[2, 1])) / den
            c = 1 - a - b
            inside = (a >= -.0001) & (b >= -.0001) & (c >= -.0001)
            verts = [world[mesh.loops[i].vertex_index] for i in loop_ids]
            wx = a * verts[0].x + b * verts[1].x + c * verts[2].x
            wy = a * verts[0].y + b * verts[1].y + c * verts[2].y
            wz = a * verts[0].z + b * verts[1].z + c * verts[2].z
            theta = np.arctan2(wy - cy, wx - cx)
            filament = np.clip((np.cos(theta * 7 + phase) - .72) / .28, 0, 1)
            drop = np.zeros_like(wz)
            for joint_z in (.673, .341, .009):
                distance = joint_z - wz
                drop = np.maximum(drop, np.where((distance >= 0) & (distance < .14), np.exp(-distance / .055), 0))
            region = result[ymin:ymax + 1, xmin:xmax + 1]
            np.maximum(region, np.where(inside, filament * drop, 0), out=region)
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("high", type=Path)
    parser.add_argument("scans", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--texture-size", type=int, choices=(512, 1024), default=1024)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    print("ATLAS", unwrap(meshes, size=args.texture_size, padding=2))
    shape_masks = masks(meshes, args.texture_size)
    mat = bpy.data.materials.new("cage_shared_atlas")
    mat.use_nodes = True
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    shader = nodes.new("ShaderNodeBsdfPrincipled")
    links.new(shader.outputs["BSDF"], output.inputs["Surface"])
    for obj in meshes:
        obj.data.materials.clear()
        obj.data.materials.append(mat)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.world = bpy.data.worlds.new("Bake World")
    scene.world.use_nodes = True
    scene.render.bake.margin = 2
    ao = bpy.data.images.new("cage_ao", width=args.texture_size, height=args.texture_size, alpha=False)
    target = nodes.new("ShaderNodeTexImage")
    target.image = ao
    nodes.active = target
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    bpy.ops.object.bake(type="AO")
    ao_path = args.output / "ao.png"
    ao.filepath_raw, ao.file_format = str(ao_path.resolve()), "PNG"
    ao.save()
    nodes.remove(target)

    normal_bake = bpy.data.images.new("cage_normal_bake", width=args.texture_size, height=args.texture_size, alpha=False)
    normal_bake.generated_color = (.5, .5, 1, 1)
    normal_bake.colorspace_settings.name = "Non-Color"
    target = nodes.new("ShaderNodeTexImage")
    target.image = normal_bake
    nodes.active = target
    original = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(args.high.resolve()))
    imported = [obj for obj in bpy.data.objects if obj not in original]
    high = {obj.name.removesuffix(".001"): obj for obj in imported if obj.type == "MESH"}
    missing = [obj.name for obj in meshes if obj.name not in high]
    if missing:
        raise RuntimeError(f"High-poly meshes missing: {missing}")
    for obj in high.values():
        axis = max(range(3), key=lambda i: obj.dimensions[i])
        for poly in obj.data.polygons:
            poly.use_smooth = not obj.name.startswith(("padlock_body", "tray_", "paper_")) and abs(poly.normal[axis]) < .8
    scene.render.bake.use_selected_to_active = True
    scene.render.bake.use_cage = True
    scene.render.bake.cage_extrusion = .003
    scene.render.bake.max_ray_distance = .01
    scene.render.bake.use_clear = False
    for low in meshes:
        bpy.ops.object.select_all(action="DESELECT")
        high[low.name].select_set(True)
        low.select_set(True)
        bpy.context.view_layer.objects.active = low
        bpy.ops.object.bake(type="NORMAL")
    baked_path = args.output / "normal_high.png"
    normal_bake.filepath_raw, normal_bake.file_format = str(baked_path.resolve()), "PNG"
    normal_bake.save()
    for obj in imported:
        bpy.data.objects.remove(obj, do_unlink=True)
    nodes.remove(target)

    size = args.texture_size
    dims = (size, size)
    base = np.asarray(Image.open(args.scans / "rusty_metal_02_diff_1k.jpg").convert("RGB").resize(dims, Image.Resampling.LANCZOS), dtype=np.float32).copy()
    arm = np.asarray(Image.open(args.scans / "rusty_metal_02_arm_1k.png").convert("RGB").resize(dims, Image.Resampling.LANCZOS)).copy()
    detail = np.asarray(Image.open(args.scans / "rusty_metal_02_nor_gl_1k.png").convert("RGB").resize(dims, Image.Resampling.LANCZOS), dtype=np.float32) / 127.5 - 1
    geometry = np.asarray(Image.open(baked_path).convert("RGB"), dtype=np.float32) / 127.5 - 1
    ao_values = np.asarray(Image.open(ao_path).convert("L"), dtype=np.float32) / 255
    paper_mask = np.asarray(shape_masks["paper"]) > 0
    rope_mask = np.asarray(shape_masks["rope"]) > 0
    tray_mask = np.asarray(shape_masks["tray"]) > 0
    base[paper_mask] = paper_texture(size)[paper_mask]
    base[rope_mask] = np.array([128, 105, 69])
    base[tray_mask] *= .68
    arm[paper_mask | rope_mask, 1] = 225
    arm[paper_mask | rope_mask, 2] = 0
    # AO and high-to-low normal gradients provide a weld/curvature wear mask.
    # The narrow vertical streaks are procedural; close-up approval is pending.
    grad = np.hypot(*np.gradient(geometry[..., 0])) + np.hypot(*np.gradient(geometry[..., 1]))
    curvature = np.clip(grad * 5, 0, 1)
    wear = np.clip((1 - ao_values) * .8 + curvature * .3, 0, 1)
    wear_img = Image.fromarray(np.uint8(wear * 255)).filter(ImageFilter.GaussianBlur(max(1, size / 512)))
    wear = np.asarray(wear_img, dtype=np.float32) / 255
    streak = rust_flow(meshes, size) * np.clip(.3 + wear, 0, 1)
    rust = np.array([125, 66, 34], dtype=np.float32)
    base = base * (1 - streak[..., None] * .45) + rust * streak[..., None] * .45
    base_path = args.output / "basecolor.png"
    Image.fromarray(np.uint8(np.clip(base, 0, 255))).save(base_path)
    detail[..., :2] *= .6
    detail[paper_mask | rope_mask, :2] *= .15
    combined = np.stack((geometry[..., 0] + detail[..., 0], geometry[..., 1] + detail[..., 1], geometry[..., 2] * detail[..., 2]), axis=-1)
    combined /= np.maximum(np.linalg.norm(combined, axis=-1, keepdims=True), 1e-8)
    normal_path = args.output / "normal.png"
    Image.fromarray(np.uint8(np.clip((combined + 1) * 127.5, 0, 255))).save(normal_path)
    orm_path = args.output / "orm.png"
    arm[..., 0] = np.uint8(ao_values * 255)
    Image.fromarray(arm).save(orm_path)

    def texture(path, non_color=False):
        node = nodes.new("ShaderNodeTexImage")
        node.image = bpy.data.images.load(str(path.resolve()), check_existing=True)
        if non_color:
            node.image.colorspace_settings.name = "Non-Color"
        return node
    color_node = texture(base_path)
    links.new(color_node.outputs["Color"], shader.inputs["Base Color"])
    normal_node = texture(normal_path, True)
    normal_map = nodes.new("ShaderNodeNormalMap")
    links.new(normal_node.outputs["Color"], normal_map.inputs["Color"])
    links.new(normal_map.outputs["Normal"], shader.inputs["Normal"])
    packed = texture(orm_path, True)
    split = nodes.new("ShaderNodeSeparateColor")
    links.new(packed.outputs["Color"], split.inputs["Color"])
    links.new(split.outputs["Green"], shader.inputs["Roughness"])
    links.new(split.outputs["Blue"], shader.inputs["Metallic"])
    group = bpy.data.node_groups.new("glTF Material Output", "ShaderNodeTree")
    group.interface.new_socket(name="Occlusion", in_out="INPUT", socket_type="NodeSocketFloat")
    group_node = nodes.new("ShaderNodeGroup")
    group_node.node_tree = group
    links.new(split.outputs["Red"], group_node.inputs["Occlusion"])

    root, hinge, door = (bpy.data.objects[n] for n in ("long_cho", "door_hinge", "door"))
    bpy.context.view_layer.update()
    world = hinge.matrix_world.copy()
    hinge.parent = root
    hinge.matrix_world = world
    bpy.context.view_layer.update()
    def moving(obj):
        parent = obj.parent
        while parent:
            if parent == door:
                return True
            parent = parent.parent
        return False
    groups = {False: [], True: []}
    for obj in meshes:
        groups[moving(obj)].append(obj)
    for move, parent in ((False, root), (True, door)):
        items = groups[move]
        bpy.ops.object.select_all(action="DESELECT")
        for obj in items:
            obj.select_set(True)
        bpy.context.view_layer.objects.active = items[0]
        bpy.ops.object.join()
        joined = bpy.context.view_layer.objects.active
        world = joined.matrix_world.copy()
        joined.parent = parent
        joined.matrix_world = world
        joined.name = "cage_door" if move else "cage_fixed"
        bpy.context.view_layer.update()
    bpy.ops.export_scene.gltf(filepath=str((args.output / "long_cho.glb").resolve()),
                              export_format="GLB", export_extras=True, export_tangents=True)


if __name__ == "__main__":
    main()

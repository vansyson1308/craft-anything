"""Experimental two-atlas cage bake. Requires bpy, Pillow and rectpack."""
import argparse
from pathlib import Path

import bpy
import numpy as np
from PIL import Image
from unwrap_cage import unwrap


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("scans", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--texture-size", type=int, choices=(256, 512), default=512)
    parser.add_argument("--high", type=Path, help="Matching high-resolution GLB for a normal bake")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    # Split by surface area, not object count, to balance the two atlases.
    groups, areas = [[], []], [0., 0.]
    for obj in sorted(meshes, key=lambda o: sum(p.area for p in o.data.polygons), reverse=True):
        index = 0 if areas[0] <= areas[1] else 1
        groups[index].append(obj)
        areas[index] += sum(p.area for p in obj.data.polygons)
    for i, group in enumerate(groups):
        print("ATLAS", i, unwrap(group, size=args.texture_size, padding=1))

    def make_material(i):
        mat = bpy.data.materials.new(f"cage_worn_metal_{i}")
        mat.use_nodes = True
        nodes = mat.node_tree.nodes
        nodes.clear()
        output = nodes.new("ShaderNodeOutputMaterial")
        shader = nodes.new("ShaderNodeBsdfPrincipled")
        mat.node_tree.links.new(shader.outputs["BSDF"], output.inputs["Surface"])
        image = bpy.data.images.new(f"cage_ao_{i}", width=args.texture_size, height=args.texture_size, alpha=False)
        target = nodes.new("ShaderNodeTexImage")
        target.image = image
        nodes.active = target
        return mat, shader, image, target

    materials = [make_material(i) for i in range(2)]
    for i, group in enumerate(groups):
        for obj in group:
            obj.data.materials.clear()
            obj.data.materials.append(materials[i][0])
    bpy.ops.object.select_all(action="DESELECT")
    for obj in meshes:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = meshes[0]
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.world = bpy.data.worlds.new("Bake World")
    scene.world.use_nodes = True
    scene.world.node_tree.nodes["Background"].inputs["Strength"].default_value = 1
    scene.render.bake.margin = 1
    bpy.ops.object.bake(type="AO")

    for i, (_, _, ao, _) in enumerate(materials):
        ao.filepath_raw = str((args.output / f"ao_{i}.png").resolve())
        ao.file_format = "PNG"
        ao.save()

    normal_bakes = []
    if args.high:
        for i, (mat, _, _, target) in enumerate(materials):
            nodes = mat.node_tree.nodes
            nodes.remove(target)
            normal_image = bpy.data.images.new(f"cage_normal_bake_{i}", width=args.texture_size,
                                               height=args.texture_size, alpha=False)
            normal_image.generated_color = (0.5, 0.5, 1.0, 1.0)
            normal_image.colorspace_settings.name = "Non-Color"
            node = nodes.new("ShaderNodeTexImage")
            node.image = normal_image
            nodes.active = node
            normal_bakes.append((normal_image, node))
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
                poly.use_smooth = not obj.name.startswith("padlock_body") and abs(poly.normal[axis]) < .8
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
        for i, (normal_image, _) in enumerate(normal_bakes):
            normal_image.filepath_raw = str((args.output / f"normal_high_{i}.png").resolve())
            normal_image.file_format = "PNG"
            normal_image.save()
        for obj in imported:
            bpy.data.objects.remove(obj, do_unlink=True)

    def resize(source, name):
        path = args.output / name
        Image.open(args.scans / source).convert("RGB").resize((args.texture_size, args.texture_size), Image.Resampling.LANCZOS).save(path)
        return path

    base_path = resize("rusty_metal_02_diff_1k.jpg", "basecolor.png")
    normal_path = resize("rusty_metal_02_nor_gl_1k.png", "normal_scan.png")
    source_orm = resize("rusty_metal_02_arm_1k.png", "orm_source.png")
    _, rough, metal = Image.open(source_orm).convert("RGB").split()
    gltf_group = bpy.data.node_groups.new("glTF Material Output", "ShaderNodeTree")
    gltf_group.interface.new_socket(name="Occlusion", in_out="INPUT", socket_type="NodeSocketFloat")
    for i, (mat, shader, ao, target) in enumerate(materials):
        ao_path = args.output / f"ao_{i}.png"
        red = Image.open(ao_path).convert("RGB").getchannel("R")
        orm_path = args.output / f"orm_{i}.png"
        Image.merge("RGB", (red, rough, metal)).save(orm_path)
        nodes, links = mat.node_tree.nodes, mat.node_tree.links
        nodes.remove(normal_bakes[i][1] if normal_bakes else target)

        def texture(path, non_color=False):
            node = nodes.new("ShaderNodeTexImage")
            node.image = bpy.data.images.load(str(path.resolve()), check_existing=True)
            if non_color:
                node.image.colorspace_settings.name = "Non-Color"
            return node

        base = texture(base_path)
        links.new(base.outputs["Color"], shader.inputs["Base Color"])
        if normal_bakes:
            # Whiteout blend: combine the baked geometric normal with the
            # tangent-space scan detail, and renormalize per pixel.
            geometry = np.asarray(Image.open(args.output / f"normal_high_{i}.png").convert("RGB"), dtype=np.float32) / 127.5 - 1
            detail = np.asarray(Image.open(normal_path).convert("RGB"), dtype=np.float32) / 127.5 - 1
            detail[..., :2] *= .6
            combined = np.stack((geometry[..., 0] + detail[..., 0], geometry[..., 1] + detail[..., 1],
                                 geometry[..., 2] * detail[..., 2]), axis=-1)
            combined /= np.maximum(np.linalg.norm(combined, axis=-1, keepdims=True), 1e-8)
            combined_path = args.output / f"normal_{i}.png"
            Image.fromarray(np.uint8(np.clip((combined + 1) * 127.5, 0, 255))).save(combined_path)
        else:
            combined_path = normal_path
        normal = texture(combined_path, True)
        normal_map = nodes.new("ShaderNodeNormalMap")
        links.new(normal.outputs["Color"], normal_map.inputs["Color"])
        links.new(normal_map.outputs["Normal"], shader.inputs["Normal"])
        packed = texture(orm_path, True)
        separate = nodes.new("ShaderNodeSeparateColor")
        links.new(packed.outputs["Color"], separate.inputs["Color"])
        links.new(separate.outputs["Green"], shader.inputs["Roughness"])
        links.new(separate.outputs["Blue"], shader.inputs["Metallic"])
        node = nodes.new("ShaderNodeGroup")
        node.node_tree = gltf_group
        links.new(separate.outputs["Red"], node.inputs["Occlusion"])

    # Consolidate draw calls without losing the door's independent pivot. The
    # sand3d export hangs its hinge under a static post, so reparent that node
    # to the root while preserving its world transform before joining meshes.
    root = bpy.data.objects["long_cho"]
    hinge = bpy.data.objects["door_hinge"]
    door = bpy.data.objects["door"]
    bpy.context.view_layer.update()
    hinge_world = hinge.matrix_world.copy()
    hinge.parent = root
    hinge.matrix_world = hinge_world
    bpy.context.view_layer.update()

    def belongs_to_door(obj):
        parent = obj.parent
        while parent:
            if parent == door:
                return True
            parent = parent.parent
        return False

    grouped = {}
    for obj in meshes:
        for i, (mat, _, _, _) in enumerate(materials):
            if obj.data.materials[0] == mat:
                grouped.setdefault((belongs_to_door(obj), i), []).append(obj)
                break
    for moving, parent in ((False, root), (True, door)):
        for i, (mat, _, _, _) in enumerate(materials):
            items = grouped.get((moving, i), [])
            if not items:
                continue
            bpy.ops.object.select_all(action="DESELECT")
            for obj in items:
                obj.select_set(True)
            bpy.context.view_layer.objects.active = items[0]
            bpy.ops.object.join()
            joined = bpy.context.view_layer.objects.active
            world = joined.matrix_world.copy()
            joined.parent = parent
            joined.matrix_world = world
            joined.name = f"cage_{'door' if moving else 'fixed'}_{i}"
            bpy.context.view_layer.update()
    bpy.ops.export_scene.gltf(filepath=str((args.output / "long_cho_textured.glb").resolve()),
                              export_format="GLB", export_extras=True, export_tangents=True)


if __name__ == "__main__":
    main()

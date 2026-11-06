"""Draft flashlight UV, PBR and selected-to-active bake; requires bpy/Pillow."""
import argparse
from pathlib import Path
import bpy
import numpy as np
from PIL import Image


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("high", type=Path)
    parser.add_argument("scans", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--texture-size", type=int, choices=(256, 512), default=512)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(args.source.resolve()))
    meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
    lens = next(o for o in meshes if o.name == "lens")
    body = [o for o in meshes if o != lens]
    bpy.ops.object.select_all(action="DESELECT")
    for obj in body:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = body[0]
    from unwrap_cage import unwrap
    print("Flashlight UV packing:", unwrap(body, size=args.texture_size, padding=2, axis_override=2))
    material = bpy.data.materials.new("flashlight_worn_metal")
    material.use_nodes = True
    nodes, links = material.node_tree.nodes, material.node_tree.links
    nodes.clear()
    output = nodes.new("ShaderNodeOutputMaterial")
    shader = nodes.new("ShaderNodeBsdfPrincipled")
    links.new(shader.outputs["BSDF"], output.inputs["Surface"])
    for obj in body:
        obj.data.materials.clear()
        obj.data.materials.append(material)
    glass = bpy.data.materials.new("lamp_glass")
    glass.use_nodes = True
    glass_shader = glass.node_tree.nodes.get("Principled BSDF")
    glass_shader.inputs["Base Color"].default_value = (0.72, .53, .30, 1)
    glass_shader.inputs["Emission Color"].default_value = (1, .60, .25, 1)
    glass_shader.inputs["Emission Strength"].default_value = 1
    lens.data.materials.clear()
    lens.data.materials.append(glass)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 32
    scene.world = bpy.data.worlds.new("Bake World")
    scene.world.use_nodes = True
    scene.render.bake.margin = 2
    ao = bpy.data.images.new("flash_ao", width=args.texture_size, height=args.texture_size, alpha=False)
    target = nodes.new("ShaderNodeTexImage")
    target.image = ao
    nodes.active = target
    bpy.ops.object.bake(type="AO")
    ao_path = args.output / "ao.png"
    ao.filepath_raw, ao.file_format = str(ao_path.resolve()), "PNG"
    ao.save()
    nodes.remove(target)
    normal_bake = bpy.data.images.new("flash_normal_bake", width=args.texture_size, height=args.texture_size, alpha=False)
    normal_bake.generated_color = (.5, .5, 1, 1)
    normal_bake.colorspace_settings.name = "Non-Color"
    target = nodes.new("ShaderNodeTexImage")
    target.image = normal_bake
    nodes.active = target
    old_objects = set(bpy.data.objects)
    bpy.ops.import_scene.gltf(filepath=str(args.high.resolve()))
    added = [obj for obj in bpy.data.objects if obj not in old_objects]
    high = {o.name.removesuffix(".001"): o for o in added if o.type == "MESH"}
    if any(obj.name not in high for obj in body):
        raise RuntimeError("Missing matching high-poly body mesh")
    scene.render.bake.use_selected_to_active = True
    scene.render.bake.use_cage = True
    scene.render.bake.cage_extrusion = .001
    scene.render.bake.max_ray_distance = .003
    scene.render.bake.use_clear = False
    for obj in body:
        bpy.ops.object.select_all(action="DESELECT")
        obj.select_set(True)
        high[obj.name].select_set(True)
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.bake(type="NORMAL")
    baked_path = args.output / "normal_baked.png"
    normal_bake.filepath_raw, normal_bake.file_format = str(baked_path.resolve()), "PNG"
    normal_bake.save()
    for obj in added:
        bpy.data.objects.remove(obj, do_unlink=True)
    nodes.remove(target)
    size = (args.texture_size, args.texture_size)
    diffuse = Image.open(args.scans / "rusty_metal_02_diff_1k.jpg").convert("RGB").resize(size, Image.Resampling.LANCZOS)
    pixels = np.asarray(diffuse, dtype=np.float32)
    # Recolour the scanned paint to a dark, worn neutral finish.
    gray = pixels.mean(axis=2, keepdims=True)
    color = np.clip(gray * .22 + 12, 0, 255).astype(np.uint8)
    base_path = args.output / "basecolor.png"
    Image.fromarray(np.repeat(color, 3, axis=2)).save(base_path)
    scan_normal = Image.open(args.scans / "rusty_metal_02_nor_gl_1k.png").convert("RGB").resize(size, Image.Resampling.LANCZOS)
    detail = np.asarray(scan_normal, dtype=np.float32) / 127.5 - 1
    geometry = np.asarray(Image.open(baked_path).convert("RGB"), dtype=np.float32) / 127.5 - 1
    detail[..., :2] *= .22
    combined = np.stack((geometry[..., 0] + detail[..., 0], geometry[..., 1] + detail[..., 1], geometry[..., 2] * detail[..., 2]), axis=-1)
    combined /= np.maximum(np.linalg.norm(combined, axis=-1, keepdims=True), 1e-8)
    normal_path = args.output / "normal.png"
    Image.fromarray(np.uint8(np.clip((combined + 1) * 127.5, 0, 255))).save(normal_path)
    arm = Image.open(args.scans / "rusty_metal_02_arm_1k.png").convert("RGB").resize(size, Image.Resampling.LANCZOS)
    _, rough, metal = arm.split()
    orm_path = args.output / "orm.png"
    Image.merge("RGB", (Image.open(ao_path).convert("RGB").getchannel("R"), rough, metal)).save(orm_path)

    def texture(path, non_color=False):
        node = nodes.new("ShaderNodeTexImage")
        node.image = bpy.data.images.load(str(path.resolve()), check_existing=True)
        if non_color:
            node.image.colorspace_settings.name = "Non-Color"
        return node
    base = texture(base_path)
    links.new(base.outputs["Color"], shader.inputs["Base Color"])
    normal = texture(normal_path, True)
    normal_map = nodes.new("ShaderNodeNormalMap")
    links.new(normal.outputs["Color"], normal_map.inputs["Color"])
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
    bpy.ops.object.select_all(action="DESELECT")
    for obj in body:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = body[0]
    bpy.ops.object.join()
    bpy.ops.export_scene.gltf(filepath=str((args.output / "den_pin.glb").resolve()), export_format="GLB", export_tangents=True)


if __name__ == "__main__":
    main()

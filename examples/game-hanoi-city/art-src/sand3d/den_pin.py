"""Editable flashlight geometry; `finish_flashlight.py` supplies the PBR bakes.

Blender's -Y exports as glTF +Z. The origin is at the grip, ready for hand_r.
"""
import argparse
from pathlib import Path

import sand3d as s3d


def build_flashlight(lod=False, high_poly=False):
    s3d.empty_scene()
    s3d.units("m")

    def cylinder(name, diameter, length, y):
        sides = 64 if high_poly else 10 if lod else 24
        obj = s3d.extrude(name, s3d.polygon(sides, diameter), length)
        return s3d.at(obj, y=y, rx=90)

    def metal(obj):
        return s3d.material(obj, "painted_metal", color="#303331", roughness=.65, metallic=.72)

    handle = metal(cylinder("grip", .030, .105, .055))
    head = metal(cylinder("head", .040, .040, -.050))
    tail = metal(cylinder("tail_cap", .032, .005, .060))
    rings = [metal(cylinder(f"grip_ring_{i}", .032, .002, .04 - i * .015)) for i in range(5)]
    glass = s3d.material(cylinder("lens", .032, .001, -.090), "light", color="#ffdca7", emission=1)
    return s3d.assembly("den_pin", handle, head, tail, rings, glass)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path)
    parser.add_argument("--lod", action="store_true")
    parser.add_argument("--high-poly", action="store_true")
    args = parser.parse_args()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    s3d.export(build_flashlight(args.lod, args.high_poly), str(args.output))

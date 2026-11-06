"""Prototype of revised flashlight, metres; Blender -Y is exported as glTF +Z."""
import argparse
import bpy
import sand3d as s3d
import sand3d._core as core

def build(lod=False, high_poly=False):
    s3d.empty_scene()
    s3d.units('m')
    n = 48 if high_poly else 8 if lod else 24
    core.TURN_MIN = core.TURN_MAX = n
    def cylinder(name, diameter, length, y):
        return s3d.at(s3d.extrude(name, s3d.polygon(n, diameter), length), y=y, rx=90)
    def turn(name, points, y):
        return s3d.at(s3d.revolve(name, points), y=y, rx=90)
    def material(obj, name, color, roughness, metallic=0, emission=None):
        preset = {'body_anodized': 'painted_metal', 'reflector': 'chrome', 'lamp_glass': 'light', 'rubber': 'rubber'}[name]
        result = s3d.material(obj, preset, color=color, roughness=roughness, metallic=metallic, emission=emission)
        obj.data.materials[0].name = name
        return result
    handle = material(cylinder('grip', .026, .090, .045), 'body_anodized', '#303031', .35, 1)
    bezel = material(turn('bezel', [(0,-.0001),(.013,-.0001),(.013,.002),(.018,.030),(.019,.032),(.019,.035),(.016,.035),(.016,.033),(.007,.020),(0,.020)], -.045), 'body_anodized', '#303031', .35, 1)
    reflector = material(turn('reflector',[(.002,.000),(.016,.015),(.0157,.015),(.0017,.000)], -.063), 'reflector', '#f3f3f3', .08, 1)
    lens = material(cylinder('lens', .032, .0005, -.0775), 'lamp_glass', '#fff4e4', .12, 0, 1)
    tail = material(cylinder('tail_cap', .027, .018, .063), 'body_anodized', '#303031', .35, 1)
    # A side-facing round button: a cylinder in X, 25 mm behind the head/body junction.
    switch = s3d.at(s3d.extrude('rubber_switch', s3d.polygon(n, .010), .002), x=.013, y=-.020, ry=90)
    material(switch, 'rubber', '#171717', .8)
    # A small actual torus provides the through-hole for the wrist cord.
    bpy.ops.mesh.primitive_torus_add(major_segments=32 if high_poly else 8 if lod else 16, minor_segments=12 if high_poly else 6, location=(0, .067, 0), major_radius=.004, minor_radius=.0012, rotation=(1.57079632679,0,0))
    loop=bpy.context.object
    loop.name='lanyard_loop'
    material(loop, 'body_anodized', '#303031', .35, 1)
    return s3d.assembly('den_pin', handle, bezel, reflector, lens, tail, switch, loop)

if __name__ == '__main__':
    p=argparse.ArgumentParser()
    p.add_argument('output')
    p.add_argument('--lod',action='store_true')
    p.add_argument('--high-poly',action='store_true')
    args=p.parse_args()
    s3d.export(build(args.lod,args.high_poly),args.output)

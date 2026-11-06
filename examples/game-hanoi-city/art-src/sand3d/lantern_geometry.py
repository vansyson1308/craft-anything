"""Draft Hang Ma lantern geometry; meters, Blender +Z up, GLTF +Y up."""
import argparse
import math
import bpy
import sand3d as s3d
import sand3d._core as core

def build(variant,lod=False,high_poly=False):
    s3d.empty_scene();s3d.units('m')
    n=48 if high_poly else 6 if lod else 20
    core.TURN_MIN=core.TURN_MAX=n
    if variant==3:
        levels=[(.045,.19),(.155,.37),(.045,.55)]
    elif variant==2:
        levels=[(.07,.20),(.125,.23),(.145,.31),(.145,.43),(.125,.51),(.07,.54)]
    else:
        levels=[(.055,.19),(.12,.23),(.15,.32),(.15,.43),(.12,.52),(.055,.56)]
    # Thin shell, leaving the top and bottom open for the light.
    profile=[(r,z) for r,z in levels]+[(max(.001,r-.002),z) for r,z in reversed(levels)]
    shell=s3d.revolve('silk_shell',profile)
    s3d.material(shell,'fabric',color='#ba1d19' if variant!=2 else '#d8a52e',roughness=.75)
    items=[shell]
    def strip(name,points,diameter):
        ob=s3d.sweep(name,s3d.polygon(12 if high_poly else 4 if lod else 6,diameter),points)
        s3d.material(ob,'pine',color='#aa7d3c',roughness=.8)
        return ob
    for i in range(3 if lod else 8):
        a=math.tau*i/(3 if lod else 8)
        points=[((r+.001)*math.cos(a),(r+.001)*math.sin(a),z) for r,z in levels]
        items.append(strip(f'bamboo_rib_{i}',points,.004))
    for label,z,r in [('lower',levels[0][1],levels[0][0]),('upper',levels[-1][1],levels[-1][0])]:
        ring=s3d.revolve('bamboo_ring_'+label,[(r-.004,z-.006),(r+.004,z-.006),(r+.004,z+.006),(r-.004,z+.006)])
        s3d.material(ring,'pine',color='#a97b38',roughness=.8)
        items.append(ring)
    items.append(strip('suspension',[(0,0,.56),(0,0,.64)],.003))
    items.append(strip('tassel_knot',[(0,0,.13),(0,0,.19)],.012))
    for i in range(3 if lod else 8):
        a=math.tau*i/(3 if lod else 8)
        obj=strip(f'tassel_{i}',[(.006*math.cos(a),.006*math.sin(a),.13),(.013*math.cos(a),.013*math.sin(a),0)],.002)
        # Tassels are dark red silk; the finish pass allocates them into the same atlas.
        obj.name=f'tassel_{i}';items.append(obj)
    bpy.ops.mesh.primitive_uv_sphere_add(segments=32 if high_poly else 6 if lod else 16, ring_count=20 if high_poly else 4 if lod else 10, radius=.035,location=(0,0,.37))
    bulb=bpy.context.object;bulb.name='inner_bulb'
    s3d.material(bulb,'light',color='#fff0cb',emission=1)
    items.append(bulb)
    return s3d.assembly(f'den_long_{variant:02}',items)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('output');p.add_argument('--variant',type=int,choices=(1,2,3),required=True);p.add_argument('--lod',action='store_true');p.add_argument('--high-poly',action='store_true')
    a=p.parse_args();s3d.export(build(a.variant,a.lod,a.high_poly),a.output)

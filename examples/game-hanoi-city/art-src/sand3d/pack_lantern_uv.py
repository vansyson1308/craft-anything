"""Repack existing Blender UV islands into a single atlas with pixel padding."""
import math
import bpy
from bpy_extras.mesh_utils import mesh_linked_uv_islands
from rectpack import newPacker,MaxRectsBssf,MaxRectsBaf,MaxRectsBlsf,MaxRectsBl,SkylineBl,GuillotineBssfSas,SORT_AREA,SORT_LSIDE,SORT_PERI

def pack_uv(objects,size,padding=8,target_density=512):
    islands=[]
    for obj in objects:
        data=obj.data;uv=data.uv_layers.active.data
        data.calc_loop_triangles()
        triangles={}
        for tri in data.loop_triangles: triangles.setdefault(tri.polygon_index,[]).append(tri)
        for faces in mesh_linked_uv_islands(data):
            area_3d=area_uv=0.0
            for fi in faces:
                for tri in triangles.get(fi,[]):
                    v=[obj.matrix_world @ data.vertices[i].co for i in tri.vertices]
                    u=[uv[i].uv for i in tri.loops]
                    area_3d+=(v[1]-v[0]).cross(v[2]-v[0]).length/2
                    area_uv+=abs((u[1].x-u[0].x)*(u[2].y-u[0].y)-(u[1].y-u[0].y)*(u[2].x-u[0].x))/2
            if area_3d<1e-12 or area_uv<1e-12: raise RuntimeError('Degenerate UV island')
            factor=target_density/(size*math.sqrt(area_uv/area_3d))
            for li in set(i for fi in faces for i in data.polygons[fi].loop_indices): uv[li].uv*=factor
            loops=[i for f in faces for i in data.polygons[f].loop_indices]
            x0=min(uv[i].uv.x for i in loops);y0=min(uv[i].uv.y for i in loops)
            x1=max(uv[i].uv.x for i in loops);y1=max(uv[i].uv.y for i in loops)
            islands.append((obj,loops,x0,y0,max(x1-x0,1e-7),max(y1-y0,1e-7)))
    def trial(scale,algorithm,sort):
        packer=newPacker(pack_algo=algorithm,sort_algo=sort,rotation=True)
        for i,(_,_,_,_,w,h) in enumerate(islands):
            packer.add_rect(max(1,math.ceil(w*scale))+2*padding,max(1,math.ceil(h*scale))+2*padding,rid=i)
        packer.add_bin(size,size);packer.pack();out=packer.rect_list()
        return out if len(out)==len(islands) else None
    best=size;placed=None
    for algorithm in (MaxRectsBssf,MaxRectsBaf,MaxRectsBlsf,MaxRectsBl,SkylineBl,GuillotineBssfSas):
        for sort in (SORT_AREA,SORT_LSIDE,SORT_PERI):
            placed=trial(best,algorithm,sort)
            if placed is not None: break
        if placed is not None: break
    if placed is None:raise RuntimeError('UV packing failed')
    for _,x,y,w,h,index in placed:
        obj,loops,x0,y0,oldw,oldh=islands[index];uv=obj.data.uv_layers.active.data
        iw,ih=w-2*padding,h-2*padding
        naturalw=max(1,math.ceil(oldw*best));naturalh=max(1,math.ceil(oldh*best))
        rotated=naturalw!=naturalh and iw==naturalh and ih==naturalw
        for li in loops:
            u=(uv[li].uv.x-x0)/oldw;v=(uv[li].uv.y-y0)/oldh
            if rotated:
                uv[li].uv=((x+padding+v*oldh*best)/size,(y+padding+(1-u)*oldw*best)/size)
            else:
                uv[li].uv=((x+padding+u*oldw*best)/size,(y+padding+v*oldh*best)/size)
    return {'islands':len(islands),'scale_px_per_original_uv':round(best,2),'padding_px':padding}

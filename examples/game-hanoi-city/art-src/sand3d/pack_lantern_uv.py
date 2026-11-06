"""Repack existing Blender UV islands into a single atlas with pixel padding."""
import math
import bpy
from bpy_extras.mesh_utils import mesh_linked_uv_islands
from rectpack import newPacker,MaxRectsBssf,MaxRectsBaf,MaxRectsBlsf,MaxRectsBl,SkylineBl,GuillotineBssfSas,SORT_AREA,SORT_LSIDE,SORT_PERI

def pack_uv(objects,size,padding=2):
    islands=[]
    for obj in objects:
        data=obj.data;uv=data.uv_layers.active.data
        for faces in mesh_linked_uv_islands(data):
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
    best=0;placed=None
    for algorithm in (MaxRectsBssf,MaxRectsBaf,MaxRectsBlsf,MaxRectsBl,SkylineBl,GuillotineBssfSas):
        for sort in (SORT_AREA,SORT_LSIDE,SORT_PERI):
            lo,hi=0,size*4
            for _ in range(15):
                mid=(lo+hi)/2;rects=trial(mid,algorithm,sort)
                if rects is None:hi=mid
                else:lo=mid
            if lo>best:best=lo;placed=trial(lo,algorithm,sort)
    if placed is None:raise RuntimeError('UV packing failed')
    for _,x,y,w,h,index in placed:
        obj,loops,x0,y0,oldw,oldh=islands[index];uv=obj.data.uv_layers.active.data
        iw,ih=w-2*padding,h-2*padding
        naturalw=max(1,math.ceil(oldw*best));naturalh=max(1,math.ceil(oldh*best))
        rotated=naturalw!=naturalh and iw==naturalh and ih==naturalw
        for li in loops:
            u=(uv[li].uv.x-x0)/oldw;v=(uv[li].uv.y-y0)/oldh
            if rotated:u,v=v,1-u
            uv[li].uv=((x+padding+u*iw)/size,(y+padding+v*ih)/size)
    return {'islands':len(islands),'scale_px_per_original_uv':round(best,2),'padding_px':padding}

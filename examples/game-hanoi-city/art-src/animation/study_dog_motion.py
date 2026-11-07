"""Append draft Wag and Jump clips to the existing dog GLB without re-exporting its skin.

This is an animation study only. The jump is an unaffiliated in-place motion;
contact with the player and artistic/rig review still need to be done.
"""
import argparse
import copy
import json
import math
import struct
from pathlib import Path


def read_glb(path):
    data=path.read_bytes();magic,version,total=struct.unpack_from('<4sII',data)
    if (magic,version,total)!=(b'glTF',2,len(data)):raise ValueError('Invalid GLB')
    chunks=[];offset=12
    while offset<len(data):
        length,kind=struct.unpack_from('<I4s',data,offset);offset+=8
        chunks.append((kind,data[offset:offset+length]));offset+=length
    if len(chunks)!=2 or chunks[0][0]!=b'JSON' or chunks[1][0]!=b'BIN\0':raise ValueError('Expected embedded GLB')
    return json.loads(chunks[0][1]),bytearray(chunks[1][1])


def quat(axis, angle):
    s=math.sin(angle/2);c=math.cos(angle/2)
    return [axis[0]*s,axis[1]*s,axis[2]*s,c]


def key_curve(keys,frame):
    for (a,x),(b,y) in zip(keys,keys[1:]):
        if frame<=b:
            t=(frame-a)/(b-a);t=t*t*(3-2*t)
            return x+(y-x)*t
    return keys[-1][1]


def build(source,destination):
    doc,blob=read_glb(source);original=copy.deepcopy(doc)
    if len(doc.get('buffers',[]))!=1:raise ValueError('Expected one buffer')
    names={n.get('name'):i for i,n in enumerate(doc['nodes']) if n.get('name')}
    required=['root','pelvis','spine','chest','neck','front_upper_l','front_upper_r','front_lower_l','front_lower_r',
              'rear_upper_l','rear_upper_r','tail_1','tail_2','tail_3','tail_4','tail_5']
    if any(n not in names for n in required):raise ValueError('Dog rig does not match expected bones')
    if any(a.get('name') in ('Wag','Jump') for a in doc.get('animations',[])):raise ValueError('Clip already exists')
    for n in required:
        node=doc['nodes'][names[n]]
        if node.get('rotation',[0,0,0,1]) != [0,0,0,1] or 'matrix' in node:raise ValueError('Unexpected rest rotation for '+n)
    views=doc.setdefault('bufferViews',[]);accessors=doc.setdefault('accessors',[])
    def accessor(values,width,kind,*,bounds=False):
        while len(blob)%4:blob.append(0)
        offset=len(blob);flat=[x for row in values for x in (row if isinstance(row,list) else [row])]
        blob.extend(struct.pack('<'+str(len(flat))+'f',*flat));view=len(views)
        views.append({'buffer':0,'byteOffset':offset,'byteLength':4*len(flat)})
        item={'bufferView':view,'componentType':5126,'count':len(values),'type':kind}
        if bounds:item.update(min=[min(flat)],max=[max(flat)])
        accessors.append(item);return len(accessors)-1
    times=[i/30 for i in range(31)];time_accessor=accessor(times,1,'SCALAR',bounds=True)
    def clip(name,channels):
        anim={'name':name,'samplers':[],'channels':[]}
        for bone,path,values in channels:
            width=4 if path=='rotation' else 3
            output=accessor(values,width,'VEC4' if width==4 else 'VEC3')
            sampler=len(anim['samplers']);anim['samplers'].append({'input':time_accessor,'output':output,'interpolation':'LINEAR'})
            anim['channels'].append({'sampler':sampler,'target':{'node':names[bone],'path':path}})
        doc.setdefault('animations',[]).append(anim)
    wag=[]
    for i in range(1,6):
        # Small phase offsets give the tip a soft follow-through; the loop closes.
        values=[quat((0,1,0),.14*math.sin(4*math.pi*t-(i-1)*.2)) for t in times]
        wag.append((f'tail_{i}','rotation',values))
    clip('Wag',wag)
    frames=list(range(31))
    lift=[(0,0),(4,0),(10,.03),(15,.05),(20,.03),(25,0),(30,0)]
    forward=[(0,0),(4,0),(10,.025),(15,.05),(20,.035),(25,0),(30,0)]
    angles={
        'pelvis':[(0,0),(4,.12),(10,-.25),(15,-.35),(20,-.20),(25,.04),(30,0)],
        'spine':[(0,0),(4,.06),(10,-.08),(15,-.10),(20,-.06),(25,.03),(30,0)],
        'chest':[(0,0),(4,.05),(10,-.08),(15,-.12),(20,-.06),(25,.02),(30,0)],
        'neck':[(0,0),(4,0),(10,.04),(15,.08),(20,.03),(25,0),(30,0)],
        'front_upper_l':[(0,0),(4,.06),(10,-.3),(15,-.45),(20,-.25),(25,.03),(30,0)],
        'front_upper_r':[(0,0),(4,.06),(10,-.3),(15,-.45),(20,-.25),(25,.03),(30,0)],
        'front_lower_l':[(0,0),(4,-.05),(10,.18),(15,.25),(20,.15),(25,-.03),(30,0)],
        'front_lower_r':[(0,0),(4,-.05),(10,.18),(15,.25),(20,.15),(25,-.03),(30,0)],
        'rear_upper_l':[(0,0),(4,-.12),(10,.25),(15,.35),(20,.2),(25,-.04),(30,0)],
        'rear_upper_r':[(0,0),(4,-.12),(10,.25),(15,.35),(20,.2),(25,-.04),(30,0)],
    }
    jump=[('root','translation',[[0,key_curve(lift,f),key_curve(forward,f)] for f in frames])]
    jump.extend((bone,'rotation',[quat((1,0,0),key_curve(keys,f)) for f in frames]) for bone,keys in angles.items())
    clip('Jump',jump)
    # Existing mesh, skin, materials, nodes and original animation descriptions are unchanged.
    for key in ('meshes','skins','materials','nodes'):
        if doc.get(key)!=original.get(key):raise AssertionError('Unexpected change: '+key)
    if doc['animations'][:len(original['animations'])]!=original['animations']:raise AssertionError('Existing clips changed')
    doc['buffers'][0]['byteLength']=len(blob)
    j=json.dumps(doc,separators=(',',':'),ensure_ascii=False).encode('utf8');j+=b' '*((-len(j))%4)
    blob.extend(b'\0'*((-len(blob))%4))
    payload=struct.pack('<I4s',len(j),b'JSON')+j+struct.pack('<I4s',len(blob),b'BIN\0')+blob
    destination.parent.mkdir(parents=True,exist_ok=True);destination.write_bytes(struct.pack('<4sII',b'glTF',2,len(payload)+12)+payload)


if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('source',type=Path);p.add_argument('destination',type=Path);a=p.parse_args();build(a.source,a.destination)

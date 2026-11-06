"""Check rig compatibility and GLB resource budgets for the thief study."""
import argparse,io,json,struct
from pathlib import Path
import numpy as np
from PIL import Image

def read(path):
    b=path.read_bytes();length=struct.unpack_from('<I',b,12)[0];return b,json.loads(b[20:20+length]),20+length+8

def accessor(b,d,base,index):
    a=d['accessors'][index];v=d['bufferViews'][a['bufferView']]
    width={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}[a['type']]
    dtype={5121:'u1',5123:'<u2',5125:'<u4',5126:'<f4'}[a['componentType']]
    if 'byteStride' in v:raise ValueError('Interleaved data is not handled by this check')
    return np.frombuffer(b,dtype=dtype,count=a['count']*width,offset=base+v.get('byteOffset',0)+a.get('byteOffset',0)).reshape(-1,width)

p=argparse.ArgumentParser();p.add_argument('reference',type=Path);p.add_argument('study',type=Path)
p.add_argument('--max-triangles',type=int,default=25000);p.add_argument('--max-texture-size',type=int,default=1024)
args=p.parse_args()
if args.max_triangles<=0 or args.max_texture_size<=0:raise ValueError('Budgets must be positive')
ref,rd,ro=read(args.reference);data,doc,off=read(args.study);rs=rd['skins'][0];skin=doc['skins'][0]
names=lambda d,s:[d['nodes'][i].get('name') for i in s['joints']]
if names(rd,rs)!=names(doc,skin) or len(skin['joints'])!=65:raise AssertionError('Joint names or order differ')
delta=float(np.max(np.abs(accessor(ref,rd,ro,rs['inverseBindMatrices'])-accessor(data,doc,off,skin['inverseBindMatrices']))))
if delta>1e-4:raise AssertionError(f'Inverse bind difference {delta}')
triangles=0
for mesh in doc['meshes']:
    for primitive in mesh['primitives']:
        triangles+=doc['accessors'][primitive['indices']]['count']//3
        attrs=primitive['attributes']
        if 'JOINTS_1' in attrs or 'WEIGHTS_1' in attrs:raise AssertionError('More than four joint influences')
        if 'WEIGHTS_0' in attrs:
            weights=accessor(data,doc,off,attrs['WEIGHTS_0'])
            if np.max(np.abs(weights.sum(axis=1)-1))>1e-4:raise AssertionError('Weights do not sum to one')
if triangles>args.max_triangles or len(doc['materials'])>4:raise AssertionError('Character budget exceeded')
for image in doc['images']:
    view=doc['bufferViews'][image['bufferView']];start=off+view.get('byteOffset',0)
    im=Image.open(io.BytesIO(data[start:start+view['byteLength']]))
    if im.width>args.max_texture_size or im.height>args.max_texture_size:raise AssertionError('Texture exceeds size budget')
print('joints',len(skin['joints']),'inverse bind max delta',delta,'triangles',triangles,'materials',len(doc['materials']),'images',len(doc['images']))

"""Pack experimental stocky character materials; not a final texture bake."""
import io,json,struct,sys
from pathlib import Path
import numpy as np
from PIL import Image

source=Path(sys.argv[1]);dest=Path(sys.argv[2]);raw=source.read_bytes();jlen=struct.unpack_from('<I',raw,12)[0];doc=json.loads(raw[20:20+jlen]);off=20+jlen+8;binary=bytearray(raw[off:]);oldviews=doc['bufferViews'];oldmats=doc['materials'];oldtextures=doc['textures'];oldimages=doc['images']
def image(matname,kind='base'):
 m=next(x for x in oldmats if x.get('name')==matname);ref=m['pbrMetallicRoughness']['baseColorTexture'] if kind=='base' else m['normalTexture'];im=oldimages[oldtextures[ref['index']]['source']];v=oldviews[im['bufferView']];return Image.open(io.BytesIO(binary[v.get('byteOffset',0):v.get('byteOffset',0)+v['byteLength']])).convert('RGBA')
def atlas(images,normal=False):
 out=Image.new('RGBA',(1024,1024),(128,128,255,255) if normal else (0,0,0,0))
 for i,im in enumerate(images):
  resized=np.array(im.resize((496,496),Image.Resampling.LANCZOS));tile=np.pad(resized,((8,8),(8,8),(0,0)),mode='edge');out.paste(Image.fromarray(tile),(512*(i%2),512*(i//2)))
 stream=io.BytesIO();out.save(stream,format='PNG',optimize=True);return stream.getvalue()
def single(im):
 out=io.BytesIO();im.resize((1024,1024),Image.Resampling.LANCZOS).convert('RGB').save(out,format='PNG',optimize=True);return out.getvalue()
payloads=[single(image('Human.body')),
          atlas([image('Human.male_casualsuit05'),image('Human.shoes01')]),
          atlas([image('Human.male_casualsuit05','normal'),image('Human.shoes01','normal')],True),
          atlas([image('Human.short01'),image('Human.eyebrow001'),image('Human.low-poly')])]
groups={'Human.body':(0,None),'Human.male_casualsuit05':(1,0),'Human.shoes01':(1,1),
        'Human.short01':(2,0),'Human.eyebrow001':(2,1),'Human.low-poly':(2,2),'trom2_dark_fabric':(3,None)}
done={}
for mesh in doc['meshes']:
 for primitive in mesh['primitives']:
  oldname=oldmats[primitive['material']]['name'];newmat,tile=groups[oldname];primitive['material']=newmat
  if tile is None:continue
  index=primitive['attributes']['TEXCOORD_0'];acc=doc['accessors'][index];v=oldviews[acc['bufferView']]
  if index in done:
   if done[index]!=tile:raise ValueError('Shared UV needs different atlas tiles')
   continue
  if acc['type']!='VEC2' or acc['componentType']!=5126 or 'byteStride' in v:raise ValueError('Unexpected UV storage')
  uv=np.frombuffer(binary,dtype='<f4',count=acc['count']*2,offset=v.get('byteOffset',0)+acc.get('byteOffset',0)).reshape(-1,2)
  uv[:,0]=(uv[:,0]*496+8+512*(tile%2))/1024;uv[:,1]=(uv[:,1]*496+8+512*(tile//2))/1024;done[index]=tile
imgviews={x['bufferView'] for x in oldimages}
for a in doc['accessors']:
 if a.get('bufferView') in imgviews or 'sparse' in a:raise ValueError('Unexpected shared image view or sparse accessor')
newviews=[];newbin=bytearray();mapping={}
for i,v in enumerate(oldviews):
 if i in imgviews:continue
 if v['buffer']!=0:raise ValueError('Multiple buffers')
 while len(newbin)%4:newbin.append(0)
 data=binary[v.get('byteOffset',0):v.get('byteOffset',0)+v['byteLength']];nv=dict(v);nv['byteOffset']=len(newbin);mapping[i]=len(newviews);newviews.append(nv);newbin.extend(data)
for a in doc['accessors']:
 if 'bufferView' in a:a['bufferView']=mapping[a['bufferView']]
images=[]
for name,data in zip(('skin','outfit','outfit_normal','hair_eyes'),payloads):
 while len(newbin)%4:newbin.append(0)
 idx=len(newviews);newviews.append({'buffer':0,'byteOffset':len(newbin),'byteLength':len(data)});newbin.extend(data);images.append({'name':name,'mimeType':'image/png','bufferView':idx})
doc['bufferViews']=newviews;doc['images']=images;doc['samplers']=[{'magFilter':9729,'minFilter':9987,'wrapS':33071,'wrapT':33071}];doc['textures']=[{'sampler':0,'source':i} for i in range(4)]
doc['materials']=[
 {'name':'stocky_skin','pbrMetallicRoughness':{'baseColorTexture':{'index':0},'metallicFactor':0,'roughnessFactor':.7}},
 {'name':'stocky_outfit','pbrMetallicRoughness':{'baseColorTexture':{'index':1},'metallicFactor':0,'roughnessFactor':.8},'normalTexture':{'index':2}},
 {'name':'stocky_hair_eyes','pbrMetallicRoughness':{'baseColorTexture':{'index':3},'metallicFactor':0,'roughnessFactor':.8},'alphaMode':'MASK','alphaCutoff':.35,'doubleSided':True},
 {'name':'trom2_dark_fabric','pbrMetallicRoughness':{'baseColorFactor':[.018,.02,.022,1],'metallicFactor':0,'roughnessFactor':.85},'doubleSided':True}]
doc['buffers'][0]['byteLength']=len(newbin);j=json.dumps(doc,separators=(',',':'),ensure_ascii=False).encode();j+=b' '*((-len(j))%4);newbin.extend(b'\0'*((-len(newbin))%4));chunks=struct.pack('<I4s',len(j),b'JSON')+j+struct.pack('<I4s',len(newbin),b'BIN\0')+newbin;dest.write_bytes(struct.pack('<4sII',b'glTF',2,len(chunks)+12)+chunks)

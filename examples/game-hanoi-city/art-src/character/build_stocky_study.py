"""Build the CC0 MakeHuman starting point for the thief study in Blender."""
import argparse,sys,pathlib,bpy
p=argparse.ArgumentParser();p.add_argument('--mpfb-source',type=pathlib.Path,required=True);p.add_argument('--userdir',type=pathlib.Path,required=True)
args=p.parse_args(sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else sys.argv[1:])
root=args.userdir.resolve();root.mkdir(parents=True,exist_ok=True)
sys.path.insert(0,str((args.mpfb_source/'src').resolve()))
bpy.utils.extension_path_user=lambda *args,**kwargs: str(root)
bpy.ops.preferences.addon_enable(module='mpfb')
from mpfb.services.humanservice import HumanService
from mpfb.services.targetservice import TargetService
info=TargetService.get_default_macro_info_dict()
info.update(gender=1.0,age=.56,muscle=.55,weight=.78,height=.52)
info['race']={'asian':1.0,'caucasian':0.0,'african':0.0}
body=HumanService.create_human(macro_detail_dict=info)
rig=HumanService.add_builtin_rig(body,'game_engine')
assets=root/'data'
skin=assets/'skins/young_asian_male/young_asian_male.mhmat'
HumanService.set_character_skin(str(skin),body,skin_type='GAMEENGINE')
for path,kind in [('clothes/male_casualsuit05/male_casualsuit05.mhclo','Clothes'),('clothes/shoes01/shoes01.mhclo','Clothes'),('eyes/low-poly/low-poly.mhclo','Eyes'),('eyebrows/eyebrow001/eyebrow001.mhclo','Eyebrows'),('hair/short01/short01.mhclo','Hair')]:
    item=assets/path
    if item.exists():
        print('adding',kind,item,flush=True)
        HumanService.add_mhclo_asset(str(item),body,asset_type=kind,material_type='GAMEENGINE',subdiv_levels=0,set_up_rigging=True)
    else:raise FileNotFoundError(f'Required MakeHuman asset missing: {item}')
for obj in bpy.context.scene.objects:
    if obj.type=='MESH':print('MESH',obj.name,len(obj.data.vertices),'mods',[m.name for m in obj.modifiers],flush=True)
bpy.ops.wm.save_as_mainfile(filepath=str(root/'stocky_study.blend'))
print('SAVED',flush=True)

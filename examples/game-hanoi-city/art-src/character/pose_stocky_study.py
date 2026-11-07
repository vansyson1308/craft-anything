import bpy,sys
from pathlib import Path
from mathutils import Vector,Matrix
root=Path(sys.argv[1]);bpy.ops.wm.open_mainfile(filepath=str(root/'stocky_study.blend'))
rig=next(o for o in bpy.context.scene.objects if o.type=='ARMATURE')
for side in ('l','r'):
    target=Vector((1 if side=='l' else -1,0,0))
    for prefix in ('upperarm','lowerarm','hand'):
        name=f'{prefix}_{side}';p=rig.pose.bones.get(name)
        if p is None:print('missing',name);continue
        bpy.context.view_layer.update()
        direction=(p.tail-p.head).normalized();q=direction.rotation_difference(target)
        head=p.head.copy();p.matrix=Matrix.Translation(head)@q.to_matrix().to_4x4()@Matrix.Translation(-head)@p.matrix
        bpy.context.view_layer.update()
        print(name,'direction',tuple(round(x,3) for x in (p.tail-p.head).normalized()),flush=True)
rig.scale=(1.10,1.10,1.10)
bpy.ops.wm.save_as_mainfile(filepath=str(root/'stocky_posed.blend'))

# Visual review — revision 02

Draft: technical compatibility evidence; visual acceptance is pending. See [README.md](README.md) for measured limitations.

| Item | Cycles preview | Bevy WebGL |
|---|---|---|
| animals/cho_co.glb | ![Cycles](previews/cho_co.jpg) | ![Bevy](review/cho_co.jpg) |
| buildings/mat_tien_tang_01.glb | ![Cycles](previews/mat_tien_tang_01.jpg) | ![Bevy](review/mat_tien_tang_01.jpg) |
| buildings/mat_tien_tang_02.glb | ![Cycles](previews/mat_tien_tang_02.jpg) | ![Bevy](review/mat_tien_tang_02.jpg) |
| buildings/mat_tien_tang_03.glb | ![Cycles](previews/mat_tien_tang_03.jpg) | ![Bevy](review/mat_tien_tang_03.jpg) |
| buildings/mat_tien_tang_04.glb | ![Cycles](previews/mat_tien_tang_04.jpg) | ![Bevy](review/mat_tien_tang_04.jpg) |
| buildings/mat_tien_tret_01.glb | ![Cycles](previews/mat_tien_tret_01.jpg) | ![Bevy](review/mat_tien_tret_01.jpg) |
| buildings/mat_tien_tret_02.glb | ![Cycles](previews/mat_tien_tret_02.jpg) | ![Bevy](review/mat_tien_tret_02.jpg) |
| buildings/mat_tien_tret_03.glb | ![Cycles](previews/mat_tien_tret_03.jpg) | ![Bevy](review/mat_tien_tret_03.jpg) |
| buildings/mat_tien_tret_04.glb | ![Cycles](previews/mat_tien_tret_04.jpg) | ![Bevy](review/mat_tien_tret_04.jpg) |
| landmarks/cau_the_huc.glb | ![Cycles](previews/cau_the_huc.jpg) | ![Bevy](review/bridge.jpg) |
| landmarks/den_ngoc_son.glb | ![Cycles](previews/den_ngoc_son.jpg) | ![Bevy](review/den_ngoc_son.jpg) |
| landmarks/thap_but_dai_nghien.glb | ![Cycles](previews/thap_but_dai_nghien.jpg) | ![Bevy](review/thap_but_dai_nghien.jpg) |
| landmarks/thap_rua.glb | ![Cycles](previews/thap_rua.jpg) | ![Bevy](review/thap_rua.jpg) |
| people/long_cho.glb | ![Cycles](previews/long_cho.jpg) | ![Bevy](review/long_cho.jpg) |
| people/minh.glb | ![Cycles](previews/minh.jpg) | ![Bevy](review/minh.jpg) |
| people/nguoi_di_duong_01.glb | ![Cycles](previews/nguoi_di_duong_01.jpg) | ![Bevy](review/nguoi_di_duong_01.jpg) |
| people/nguoi_di_duong_02.glb | ![Cycles](previews/nguoi_di_duong_02.jpg) | ![Bevy](review/nguoi_di_duong_02.jpg) |
| people/nguoi_di_duong_03.glb | ![Cycles](previews/nguoi_di_duong_03.jpg) | ![Bevy](review/nguoi_di_duong_03.jpg) |
| people/trom_01.glb | ![Cycles](previews/trom_01.jpg) | ![Bevy](review/trom_01.jpg) |
| street/bo_via.glb | ![Cycles](previews/bo_via.jpg) | ![Bevy](review/bo_via.jpg) |
| street/cot_bien_ten_pho.glb | ![Cycles](previews/cot_bien_ten_pho.jpg) | ![Bevy](review/cot_bien_ten_pho.jpg) |
| street/den_duong_01.glb | ![Cycles](previews/den_duong_01.jpg) | ![Bevy](review/den_duong_01.jpg) |
| street/dep_to_ong.glb | ![Cycles](previews/dep_to_ong.jpg) | ![Bevy](review/dep_to_ong.jpg) |
| street/hang_rao_ho.glb | ![Cycles](previews/hang_rao_ho.jpg) | ![Bevy](review/hang_rao_ho.jpg) |
| street/so_tay.glb | ![Cycles](previews/so_tay.jpg) | ![Bevy](review/so_tay.jpg) |
| street/thung_hang.glb | ![Cycles](previews/thung_hang.jpg) | ![Bevy](review/thung_hang.jpg) |
| street/xe_may_01_chay.glb | ![Cycles](previews/xe_may_01_chay.jpg) | ![Bevy](review/xe_may_01_chay.jpg) |
| trees/xa_cu.glb | ![Cycles](previews/xa_cu.jpg) | ![Bevy](review/xa_cu.jpg) |
| materials/via_he_da_xam | ![Cycles](previews/material_via_he_da_xam.jpg) | ![Bevy](review/material_via_he_da_xam.jpg) |
| materials/duong_nhua | ![Cycles](previews/material_duong_nhua.jpg) | ![Bevy](review/material_duong_nhua.jpg) |
| materials/co | ![Cycles](previews/material_co.jpg) | ![Bevy](review/material_co.jpg) |
| materials/gach_lat_ho | ![Cycles](previews/material_gach_lat_ho.jpg) | ![Bevy](review/material_gach_lat_ho.jpg) |
| materials/nuoc_ho | ![Cycles](previews/material_nuoc_ho.jpg) | ![Bevy](review/material_nuoc_ho.jpg) |

## Full-manifest cameras

All four use 41 manifest entries. The lineup shows an unresolved gate/placement obstruction.

### lineup

![lineup](review/lineup.jpg)

### thehuc

![thehuc](review/thehuc.jpg)

### bridge

![bridge](review/bridge.jpg)

### tree

![tree](review/tree.jpg)

## Isolated animation review

Only Minh, thief, dog and bridge are enabled for the isolated lineup. This makes actors visible without the unresolved gate overlap. The following frames do not certify absence of clipping through entire clips.

### Idle_Loop

![Idle_Loop](review/lineup_isolated.jpg)

### Sprint_Loop

![Sprint_Loop](review/clip_Sprint_Loop.jpg)

### Crouch_Fwd_Loop

![Crouch_Fwd_Loop](review/clip_Crouch_Fwd_Loop.jpg)

### Punch_Cross

![Punch_Cross](review/clip_Punch_Cross.jpg)

### Roll

![Roll](review/clip_Roll_mid.jpg)

### Death01

![Death01](review/clip_Death01_later.jpg)

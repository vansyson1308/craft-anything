# ASSETS.md: quy chuẩn tài nguyên 3D cho Hanoi City

Tài liệu này dành cho người (hoặc agent) làm tài nguyên 3D cho game **Hanoi City: Giải Cứu Cún**.
Game chạy trong trình duyệt **máy tính (PC, laptop)** (Rust + Bevy, WebAssembly, WebGPU, có WebGL2 dự phòng),
trên bản đồ thật quận Hoàn Kiếm lấy từ OpenStreetMap. Làm đúng quy chuẩn này thì tài nguyên được
đưa vào game mà không phải sửa tay.

**Đích chất lượng:** góc phố Đinh Tiên Hoàng nhìn ra cầu Thê Húc lúc nắng chiều: vỉa hè lát đá
xám, bó vỉa đá, cây xà cừ thân lớn, hàng rào sắt thấp ven hồ, cột biển tên phố sơn đỏ trắng, biển
tên phố nền xanh chữ trắng, dãy nhà vàng kiểu Pháp bên kia hồ. Chân thực, không hoạt hình.

---

## 1. Cách game dùng tài nguyên (giống cách GTA V được đưa lên web)

Bản GTA V chạy trên web (10/2026) dựa trên ba điều: code WebAssembly, đồ họa WebGPU, và **tài
nguyên được tải dần theo khu vực người chơi đang đứng**, không tải cả thế giới một lúc. Game này
làm giống hệt:

- **Tải theo khu:** thành phố chia thành ô 160 m × 160 m. Chỉ những ô quanh người chơi được tải.
  Vì vậy mỗi tài nguyên phải là **một file riêng**, không gộp cả phố vào một file.
- **Mức chi tiết (LOD):** mỗi vật có 2–3 bản, gần dùng bản đẹp, xa dùng bản thô (cây xa nữa thì dùng
  ảnh billboard). Không có LOD thì máy tính phổ thông không chạy nổi cả quận.
- **Nén:** phần nén cho web (texture, mô hình) là việc của game khi build. Người làm tài nguyên chỉ
  cần nộp PNG/JPG và GLB chuẩn.
- **Ánh sáng nướng sẵn:** bản WebGL2 dự phòng không tính được che khuất ánh sáng thời gian thực,
  nên ambient occlusion phải được **nướng sẵn vào texture** (kênh R của ORM, mục 3).
- **Tự nhận và dự phòng:** game thấy file đúng tên trong thư mục thì dùng, không thấy thì dùng
  bản hiện tại. Nộp từng món một cũng được.

## 2. Quy tắc chung cho mọi file

| Mục | Quy định |
|---|---|
| Định dạng | **glTF 2.0 nhị phân `.glb`**, một vật mỗi file. Không FBX, OBJ, Blender. **Không nộp ảnh render thay cho mô hình.** |
| Đơn vị | **Mét**. 1 đơn vị = 1 m. Người cao 1,65–1,80 m. |
| Trục | **+Y hướng lên**, **+Z là mặt trước** (chuẩn glTF, xuất từ Blender bằng exporter mặc định là đúng). |
| Hướng trên bản đồ | Với công trình đặt đúng chỗ thật: **−Z là hướng Bắc, +X là hướng Đông**. |
| Gốc toạ độ (pivot) | Nhân vật, động vật, xe, vật dụng: **giữa hai chân / giữa đáy, ở mặt đất (y = 0)**. Công trình: **tâm mặt bằng, ở mặt đất**. Mô-đun mặt tiền: góc **dưới trái** của mặt tường. |
| Tên file | Chữ thường, không dấu, nối bằng `_`, đúng tên ở mục 6. LOD: `ten.glb` (LOD0), `ten_lod1.glb`, `ten_lod2.glb`. |
| Biến đổi | Đã áp dụng hết (scale 1, rotation 0) ở nút gốc. Không để camera, đèn hay vật thừa trong file. |
| Kiểm tra | File phải qua [glTF Validator](https://github.khronos.org/glTF-Validator/) không lỗi. |

## 3. Vật liệu và texture

- **PBR metallic-roughness** chuẩn glTF, mỗi vật **tối đa 2 vật liệu** (nhân vật tối đa 4: da, áo, quần/giày, tóc).
- Mỗi vật liệu gồm:
  - `baseColor` (sRGB, không có bóng đổ hay ánh sáng vẽ sẵn, trừ AO nhẹ),
  - `normal` (chuẩn **OpenGL, +Y**),
  - **ORM** gộp một ảnh: R = ambient occlusion (nướng sẵn), G = roughness, B = metallic.
  - Đèn, cửa sổ sáng, biển hiệu phát sáng: thêm `emissive`.
- **Vật lớn** (cầu, công trình, mô-đun nhà): vật liệu lặp theo mét trên UV0, còn AO nướng tách thành một ảnh
  xám riêng trên UV1 (`occlusionTexture.texCoord = 1`). Game hỗ trợ đúng 2 bộ UV.
- Kích thước texture là luỹ thừa của 2: **tối đa 2048** (công trình lớn), 1024 (nhân vật, xe), 512 (vật nhỏ). Tối thiểu 4 px texture cho mỗi cm ở khoảng cách 2 m.
- Kính, nước, lá cây: lá dùng `alphaMode: MASK` (không dùng BLEND cho lá). Kính dùng BLEND.
- Vật liệu nền lặp (vỉa hè, đường, cỏ…): ảnh **lặp liền mạch**, kèm ghi chú mỗi ảnh phủ bao nhiêu mét.

## 4. Ngân sách đa giác (tam giác)

| Loại | LOD0 (gần) | LOD1 | LOD2 (xa) |
|---|---|---|---|
| Nhân vật chính, tên trộm | ≤ 25.000 | ≤ 8.000 | ≤ 2.000 |
| Người đi đường | ≤ 12.000 | ≤ 4.000 | ≤ 1.000 |
| Chó | ≤ 8.000 | ≤ 2.500 | — |
| Xe máy (kèm người lái) | ≤ 15.000 | ≤ 4.000 | ≤ 800 |
| Công trình biểu tượng | ≤ 60.000 | ≤ 15.000 | ≤ 3.000 |
| Mô-đun mặt tiền nhà ống | ≤ 4.000 | ≤ 800 | (game tự dùng tường phẳng) |
| Cây lớn | ≤ 15.000 | ≤ 3.000 | **billboard**: 2 mặt phẳng chéo + ảnh cây |
| Vật dụng đường phố | ≤ 2.000 | ≤ 500 | — |

## 5. Nhân vật: khung xương bắt buộc

Game có sẵn **43 động tác** (đi, chạy, chạy nước rút, đi lom khom, đấm, lăn, trúng đòn, ngã, nhặt
đồ, ngồi…) dùng chung một khung xương. Nhân vật đúng khung xương này sẽ tự động có đủ các động tác.

- Cây xương: nút gốc tên **`Armature`**, bên trong là **`root`**, rồi **`pelvis`**, đúng thứ tự cha–con như
  bản mẫu `assets/people/minh/person.gltf` trong repo. **Cách chắc chắn nhất: bọc (skin) mô hình mới lên
  chính khung xương của file mẫu đó**, giữ nguyên tên, vị trí và hướng từng xương.
- Đủ **65 xương, đúng tên** (phân biệt hoa thường):

  ```text
  root pelvis spine_01 spine_02 spine_03 neck_01 Head
  clavicle_l upperarm_l lowerarm_l hand_l
  index_01_l index_02_l index_03_l index_04_leaf_l middle_01_l middle_02_l middle_03_l middle_04_leaf_l
  pinky_01_l pinky_02_l pinky_03_l pinky_04_leaf_l ring_01_l ring_02_l ring_03_l ring_04_leaf_l
  thumb_01_l thumb_02_l thumb_03_l thumb_04_leaf_l
  clavicle_r upperarm_r lowerarm_r hand_r
  index_01_r index_02_r index_03_r index_04_leaf_r middle_01_r middle_02_r middle_03_r middle_04_leaf_r
  pinky_01_r pinky_02_r pinky_03_r pinky_04_leaf_r ring_01_r ring_02_r ring_03_r ring_04_leaf_r
  thumb_01_r thumb_02_r thumb_03_r thumb_04_leaf_r
  thigh_l calf_l foot_l ball_l ball_leaf_l thigh_r calf_r foot_r ball_r ball_leaf_r
  ```

- Tư thế nghỉ: **T-pose**, mặt quay **+Z**, tối đa 4 xương ảnh hưởng mỗi đỉnh.
- Tóc, mũ, kính, khẩu trang: được để chung file, nhưng phải skin vào cùng khung xương (tóc theo `Head`).
- **Nếu không dùng được khung xương trên**, nhân vật phải kèm sẵn trong file **12 động tác, đúng tên**:
  `Idle_Loop`, `Walk_Loop`, `Jog_Fwd_Loop`, `Sprint_Loop`, `Crouch_Fwd_Loop`, `Crouch_Idle_Loop`,
  `Punch_Cross`, `Roll`, `Hit_Chest`, `Death01`, `Interact`, `Sitting_Idle_Loop`.
- **Không dùng khuôn mặt người có thật.** Người Việt, trang phục đời thường Hà Nội.

## 6. Danh sách tài nguyên và tên file

Thư mục nộp: `assets/hq/<nhóm>/<ten>.glb`. Có thể tạo thêm biến thể `_02`, `_03`…

### Nhóm A: nhân vật và động vật (`hq/people/`, `hq/animals/`)

| Tên file | Mô tả |
|---|---|
| `minh.glb` | **Nhân vật chính**: nam sinh viên 20 tuổi, tóc đen ngắn, áo khoác jean đen bên ngoài áo phông trắng, quần jean tối, giày sneaker trắng xám, đồng hồ tay. |
| `trom_01.glb`, `trom_02.glb` | Tên trộm chó: áo khoác gió tối màu, mũ lưỡi trai, khẩu trang vải, dép hoặc giày cũ. Một người gầy, một người to. |
| `tu_long.glb` | Trùm phụ "Tư Lồng": trung niên, áo sơ mi hoa mở cúc, dây chuyền, bụng hơi to. |
| `bay_thong_long.glb` | Trùm "Bảy Thòng Lọng": cao to, áo ba lỗ đen, hình xăm tay, râu, mũ bảo hiểm nửa đầu. |
| `co_hang.glb` | Cô bán bánh mì: ngoài 40, áo chống nắng, nón lá. |
| `nguoi_di_duong_01…06.glb` | Người đi dạo hồ: nữ váy trắng áo đen, ông cụ mũ lưỡi trai, cặp đôi, học sinh áo đồng phục, người tập thể dục. |
| `cho_co.glb` | **Chó cỏ Việt** lông vàng, tai dựng, đuôi cong. Khung xương riêng, **kèm động tác trong file**: `Idle`, `Walk`, `Run`, `Sit`, `Bark`. Pivot giữa 4 chân, dài khoảng 0,7 m. |
| `cho_phu_quoc.glb`, `cho_bac_ha.glb` | Như trên, giống chó Phú Quốc (lông ngắn sẫm, xoáy lưng) và Bắc Hà (lông dài sáng). |
| `long_cho.glb` | Lồng sắt nhốt chó (0,9 × 0,7 × 0,7 m) có cửa mở được: nút cửa tên `door`, bản lề ở cạnh trái. |
| `thong_long.glb` | Cây thòng lọng bắt chó (gậy dài 1,5 m có vòng dây), pivot ở tay cầm. |

### Nhóm B: công trình biểu tượng (`hq/landmarks/`)

Đặt đúng chỗ thật. Mặt bằng theo bản đồ (Đông–Tây × Bắc–Nam) để tham khảo, **chiều cao làm theo thực tế**.

| Tên file | Mô tả | Mặt bằng trên bản đồ |
|---|---|---|
| `cau_the_huc.glb` | Cầu Thê Húc: cầu gỗ cong sơn đỏ, lan can con tiện, nối bờ với đảo Ngọc Sơn. | dài khoảng 46 m (gồm đoạn dẫn), rộng 2,5–3 m |
| `den_ngoc_son.glb` | Cụm đền Ngọc Sơn: Đắc Nguyệt Lâu, Trấn Ba Đình, nhà chính, cây cối trên đảo. | khoảng 27 × 27 m |
| `thap_but_dai_nghien.glb` | Tháp Bút, Đài Nghiên và cổng ở lối vào đền. | gốc ở điểm đầu cầu chạm bờ; −Z chỉ dọc cầu về phía đảo, cụm tháp ở phía +Z |
| `thap_rua.glb` | Tháp Rùa trên gò nhỏ giữa hồ, ba tầng, tường vôi rêu phong. | khoảng 4,5 × 6,3 m |
| `nha_tho_lon.glb` | Nhà thờ Lớn: tháp đôi gothic, tường đá xám đen, cửa hoa hồng. | khoảng 63 × 46 m |
| `nha_hat_lon.glb` | Nhà hát Lớn: kiến trúc Pháp màu vàng kem, mái vòm, hàng cột. | khoảng 83 × 52 m |
| `o_quan_chuong.glb` | Ô Quan Chưởng: cửa ô gạch, mái cong, vọng lâu trên đỉnh, có lối đi qua. | khoảng 11,5 × 17,8 m |
| `cho_dong_xuan.glb` | Chợ Đồng Xuân: mặt tiền 5 mái nhọn, chữ "CHỢ ĐỒNG XUÂN" trên mặt tiền. | khoảng 105 × 100 m |
| `ga_long_bien.glb` | Ga Long Biên: nhà ga trên cao, tường vàng. | khoảng 44 × 20 m |
| `cau_long_bien_nhip.glb` | **Một nhịp** cầu Long Biên (dàn thép sẫm gỉ, đường ray giữa, hai làn xe máy hai bên), **lặp được**: hai đầu khớp nhau. | nhịp dài 75–80 m, rộng khoảng 10 m |
| `ho_guom_ke.glb` | Một đoạn kè hồ dài 10 m, lặp được: bờ đá, hàng rào sắt thấp, luống cây cảnh. | 10 m |

### Nhóm C: nhà phố (`hq/buildings/`): mô-đun mặt tiền

Game dựng 15.000 ngôi nhà theo đúng mặt bằng thật, rồi **ốp mặt tiền bằng mô-đun**. Vì vậy cần
mô-đun lặp được, không cần từng ngôi nhà:

| Tên file | Kích thước (rộng × cao) | Mô tả |
|---|---|---|
| `mat_tien_tret_01…08.glb` | **4,0 × 3,6 m** | Tầng trệt nhà ống: cửa hàng mở cửa (hàng hoá, đèn), cửa cuốn đóng, quán cà phê ghế nhựa, tiệm vàng, hàng mã, quán phở. Kèm `emissive` cho đèn trong tiệm. Biển hiệu **để trống** (game tự in tên quán thật lên đó, xem mục 7). |
| `mat_tien_tang_01…10.glb` | **4,0 × 3,3 m** | Một tầng lầu: cửa sổ chớp xanh, ban công sắt có chậu cây, điều hoà, dây điện, quần áo phơi. Kèm `emissive` cho cửa sổ sáng đèn. |
| `mat_tien_phap_tret_01…03.glb`, `mat_tien_phap_tang_01…03.glb` | 4,0 × 4,2 m | Nhà kiểu Pháp quanh hồ: tường vàng, cửa vòm, cửa chớp gỗ. |
| `mai_bon_nuoc.glb`, `mai_tum.glb`, `mai_ngoi_01.glb` | theo vật | Đồ trên mái: bồn nước inox, tum cầu thang, mái ngói nhỏ. |

Mô-đun: mặt tường nằm trong mặt phẳng **z = 0**, nhìn ra **+Z**, chi tiết nhô ra tối đa 0,6 m
(ban công), **hai cạnh trái phải khớp liền** khi đặt cạnh nhau.

### Nhóm D: đường phố (`hq/street/`)

| Tên file | Mô tả |
|---|---|
| `cot_bien_ten_pho.glb` | Cột sơn sọc đỏ trắng, cao khoảng 3 m, kèm biển tên phố nền xanh viền trắng, có logo Hà Nội. **Mặt chữ để trống** và tách thành vật liệu riêng tên `sign_text`. Game đặt tấm chữ tên phố thật của nó ngay trước mặt biển: mặt biển ở x từ −0,49 đến 0,49 m, y từ 2,43 đến 2,73 m, z = 0,03, nhìn ra +Z (như bản lô 02); giữ đúng vị trí này. |
| `den_duong_01.glb` | Cột đèn đường Hà Nội, có `emissive`. |
| `hang_rao_ho.glb` | Đoạn hàng rào sắt thấp ven hồ dài 2 m, lặp được. |
| `bo_via.glb` | Đoạn bó vỉa đá granite dài 1 m, lặp được. |
| `ghe_nhua_xanh.glb`, `ghe_nhua_do.glb`, `ban_nhua.glb` | Ghế đẩu nhựa, bàn nhựa vỉa hè. |
| `xe_may_01…04.glb` | Xe máy số và tay ga **chung chung, không logo hãng**. Có bản đậu (`_dau`) và bản đang chạy kèm người lái đội mũ bảo hiểm (`_chay`). |
| `xe_dap_tho.glb`, `ganh_hang_rong.glb`, `xe_banh_mi.glb` | Xe đạp chở hàng, gánh hàng rong, xe bánh mì của cô Hằng. |
| `thung_rac.glb`, `chau_cay.glb`, `cot_dien_day.glb` | Thùng rác, chậu cây, cột điện chằng chịt dây. |
| `dep_to_ong.glb`, `so_tay.glb`, `thung_hang.glb` | Đồ nhặt được trong game: đôi dép tổ ong (ném để đánh lạc hướng), sổ ghi chép của bọn trộm (manh mối), thùng hàng nhỏ (chiến lợi phẩm). Pivot giữa đáy, dài 0,25–0,6 m. |
| `den_long_01…03.glb` | Đèn lồng phố Hàng Mã (đỏ, vàng), `emissive`, treo được trên dây. |
| `toa_tau.glb`, `dau_may.glb` | Toa tàu khách và đầu máy kiểu Đường sắt Việt Nam (xanh lam sọc trắng/vàng), **không logo thật**, dài khoảng 20 m. |

### Nhóm E: cây (`hq/trees/`)

`xa_cu.glb` (xà cừ thân lớn ven hồ), `sau.glb` (cây sấu), `lieu_ru.glb` (liễu rủ ven hồ),
`bang.glb` (bàng), `cay_bui_01…03.glb`. Mỗi cây đủ LOD0, LOD1, kèm `ten_billboard.png` (ảnh cây
nền trong suốt 512 × 1024) cho tầm xa. Lá dùng `alphaMode: MASK`.

### Nhóm F: vật liệu nền lặp (`hq/materials/<ten>/`)

Mỗi thư mục gồm `basecolor.png`, `normal.png`, `orm.png`, 2048 × 2048, lặp liền mạch, kèm
`info.json` ghi số mét mỗi ảnh phủ:

`via_he_da_xam` (vỉa hè lát đá xám như ảnh mẫu), `bo_via_granite`, `duong_nhua` (có vết bẩn),
`duong_nhua_vach` (vạch kẻ trắng), `gach_lat_ho` (lối dạo ven hồ), `co`, `dat_goc_cay`,
`nuoc_ho` (chỉ cần `normal.png` sóng nhẹ, lặp), `tuong_vua_vang`, `tuong_vua_kem`, `ngoi_do`, `be_tong_mai`.

### Nhóm G: ảnh 2D (`hq/2d/`)

`man_hinh_cho_ngang.jpg` (1920 × 1080), `man_hinh_cho_doc.jpg` (1080 × 1920): ảnh màn hình chờ,
chất lượng như ảnh mẫu. `icon.png` (512 × 512). `bau_troi_hoang_hon.hdr` (tuỳ chọn, HDRI bầu trời
chiều, 2048 × 1024).

## 7. Chữ trong game

Tên phố và tên quán là chữ thật, lấy từ bản đồ, do game tự in. Vì vậy **biển hiệu và biển tên phố
để trống**, không vẽ chữ vào texture. Riêng chữ "CHỢ ĐỒNG XUÂN" trên chợ thì được vẽ sẵn.

## 8. Những thứ không được có

- **Logo, tên thương hiệu thật**: ngân hàng (BIDV, Vietcombank…), hãng xe (Honda, Yamaha…), chuỗi cửa
  hàng, Đường sắt Việt Nam. Chỉ dùng tên giả hoặc để trống.
- **Khuôn mặt hay hình dáng người có thật**, người nổi tiếng.
- **Bất cứ thứ gì lấy từ game khác** (GTA hay bất kỳ game nào): mô hình, texture, âm thanh.
- Ảnh chụp có bản quyền dùng làm texture. Chỉ dùng ảnh tự tạo, hoặc ảnh CC0 có ghi nguồn.

## 9. Cách nộp

1. Đặt đúng cấu trúc `hq/<nhóm>/…` ở trên, nén một file `.zip`, hoặc thả thẳng vào
   `examples/game-hanoi-city/assets/hq/` trong repo.
2. Kèm **`hq/manifest.json`**, mỗi tài nguyên một dòng:

   ```json
   {
     "version": 1,
     "assets": [
       {
         "file": "people/minh.glb",
         "lods": ["people/minh_lod1.glb", "people/minh_lod2.glb"],
         "kind": "character",
         "size_m": [0.55, 1.74, 0.32],
         "triangles": [24000, 7800, 1900],
         "textures": 3,
         "skeleton": "armature-65",
         "animations": [],
         "license": "Tự tạo, cho phép dùng công khai trong dự án mã nguồn mở",
         "made_with": "tên công cụ hoặc agent"
       }
     ]
   }
   ```

   `skeleton` là `"armature-65"` nếu dùng khung xương mục 5, hoặc `"own"` kèm danh sách `animations`.

   Cách khai báo các loại khác trong `manifest.json`:

   - **Vật liệu nền** (nhóm F): một dòng cho cả thư mục, ví dụ `{"file": "materials/via_he_da_xam"}`.
     Trong thư mục có `basecolor.png`, `normal.png`, `orm.png` và `info.json` dạng `{"metres": 2.0}`
     (số mét một ảnh phủ).
   - **Cây** (nhóm E): `{"file": "trees/xa_cu.glb", "lods": ["trees/xa_cu_lod1.glb"], "size_m": [rộng, cao, sâu]}`.
     Ảnh `trees/xa_cu_billboard.png` nằm cạnh file, không cần khai báo. `size_m` dùng để dựng
     billboard đúng cỡ.
   - **Mô-đun mặt tiền** (nhóm C): mỗi mô-đun một dòng, kèm `lods` nếu có. Bản LOD0 dùng trong vòng
     45 m, LOD1 xa hơn. Game chỉ ốp mặt tiền khi có **ít nhất một** `mat_tien_tret_xx` **và một**
     `mat_tien_tang_xx`.
3. **Thứ tự ưu tiên** (làm theo đúng thứ tự này):
   1. `minh.glb`
   2. `cau_the_huc.glb`
   3. `cho_co.glb`
   4. `trom_01.glb`
   5. `xa_cu.glb`
   6. vật liệu `via_he_da_xam` và `duong_nhua`
   7. `cot_bien_ten_pho.glb`
   8. `hang_rao_ho.glb` và `bo_via.glb`
   9. `mat_tien_tret_01…04` và `mat_tien_tang_01…04`
   10. `den_ngoc_son.glb`
   11. `thap_rua.glb`
   12. `xe_may_01_chay.glb`
   13. `nguoi_di_duong_01…03`
   14. `long_cho.glb`
   15. `o_quan_chuong.glb`

   Mười lăm món này đủ để góc phố Đinh Tiên Hoàng – Thê Húc trong game giống ảnh mẫu.

## 10. Khi nhận, game sẽ kiểm tra tự động

- glTF Validator không lỗi; đúng tên file; có trong `manifest.json`.
- Kích thước thật (người 1,6–1,9 m, cửa đi khoảng 2,1 m) và pivot đúng chỗ.
- Số tam giác và cỡ texture trong ngân sách.
- Nhân vật: đủ 65 xương đúng tên, hoặc đủ 12 động tác đúng tên.
- Không có logo trong ảnh texture (kiểm tra bằng mắt trên ảnh chụp trong game).
- Chạy thử trong trình duyệt cỡ điện thoại và máy tính, chụp ảnh đặt cạnh ảnh mẫu để so.

## 11. Hàng tạm đang dùng (cần thay, theo thứ tự ảnh hưởng)

Đây là những thứ hiện chỉ được dựng tạm bằng khối hình học hoặc mượn từ bộ CC0. Mỗi món có tên file
đúng như mục 6; nộp file nào thì game tự thay món đó.

| Đang thấy trong game | Thiếu gì | File cần nộp |
|---|---|---|
| Nhà là khối hộp dán ảnh cửa sổ lặp lại | Mặt tiền nhà ống thật: cửa hàng, ban công, mái hiên, dây điện | `buildings/mat_tien_tret_01…08`, `buildings/mat_tien_tang_01…10` |
| Tháp Rùa là một khối hộp cao 17,5 m trên đảo | Toà tháp ba tầng thật (khoảng 8 m) trên gò | `landmarks/thap_rua.glb` |
| Cầu Thê Húc là một dải phẳng màu đỏ | Cầu gỗ cong có lan can | `landmarks/cau_the_huc.glb` |
| Đền Ngọc Sơn, Nhà thờ Lớn, Nhà hát Lớn, Ô Quan Chưởng, Chợ Đồng Xuân là khối hộp theo mặt bằng | Công trình thật | các file nhóm B |
| Cây là thân trụ với hai quả cầu xanh | Cây xà cừ, sấu, bàng, liễu | nhóm E |
| Mặt hồ là một màu xanh phẳng | Sóng gợn nhẹ | `materials/nuoc_ho/normal.png` |
| Vỉa hè, mặt đường, cỏ là ảnh CC0 chung chung | Đá xám Hà Nội, nhựa đường có vạch | nhóm F |
| Minh và bọn trộm là nhân vật mẫu CC0, quần áo tô màu | Người Việt, trang phục đời thường | nhóm A |
| Chó là mô hình cáo CC0 tô lại màu | Chó cỏ Việt | `animals/cho_co.glb` |
| Lồng chó là khối kính mờ; xe máy là khối hộp | Lồng sắt, xe máy có người lái | `people/long_cho.glb`, `street/xe_may_01_chay.glb` (…`_04_chay`) |
| Tàu hoả là các khối hộp | Toa tàu (game chưa tự thay: sẽ làm khi có file) | `street/toa_tau.glb` |
| Đồ nhặt được (dép tổ ong, sổ ghi chép, thùng hàng) là khối màu | Vật nhỏ thật, mỗi món dưới 2.000 tam giác | `street/dep_to_ong.glb`, `street/so_tay.glb`, `street/thung_hang.glb` |

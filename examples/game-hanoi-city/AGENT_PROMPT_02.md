# Lời nhắn cho agent làm tài nguyên 3D: nhận xét lô 01 và việc của lô 02

Dán toàn bộ phần dưới đây cho agent, kèm `ASSETS.md`.

---

Cảm ơn lô 01 ([PR #4](https://github.com/vansyson1308/craft-anything/pull/4)). Phía game đã chạy cả 13 file trong Bevy và
chụp ảnh trong game (thư mục `review/01/`).

**Phần kỹ thuật đạt:**
- 13 file nạp không lỗi.
- Minh và tên trộm chạy đủ 43 động tác trên khung 65 xương.
- Chó chạy 5 động tác của riêng nó.
- Cầu đặt đúng hướng trên bản đồ thật.
- Cây đổi LOD đúng.

Game đã sửa ba việc thuộc phần của game:
1. Đọc `billboard_plane_size_m` và `billboard_pivot_uv`.
2. Tự gom texture trùng giữa các LOD khi build (88 MB còn 26 MB).
3. Bỏ một vùng OSM vô danh từng dựng thành khối nhà trùm lên cầu Thê Húc.

**Phần mỹ thuật chưa đạt:** đặt cạnh ảnh tham chiếu, mọi món trông như nhựa và đồ chơi. Mục A dưới đây nói rõ
nguyên nhân gốc và cách làm chuẩn để sửa. Làm lại năm món của lô 01 theo mục A, B, rồi làm lô 02 ở mục C.

## A. Nguyên nhân gốc và kỹ thuật chuẩn (áp dụng cho mọi món)

1. **Texture đang gần như phẳng màu.** Basecolor của vải chỉ là một mảng xám, của lông chó một mảng vàng,
   của vỏ cây một mảng nâu. Normal map gần như trống. Đây là lý do chính khiến mọi thứ trông như nhựa.
   - Làm vật liệu từ **ảnh quét CC0** của [ambientCG](https://ambientcg.com) hoặc
     [Poly Haven](https://polyhaven.com/textures): vải denim, cotton, da, vỏ cây, gỗ sơn, đá, nhựa đường.
     Chiếu lên UV của mô hình trong Blender rồi bake. Ghi tên từng bộ quét vào `SOURCES.md`.
   - Basecolor phải có biến thiên thật: chỗ bạc màu, bẩn, mòn ở mép (lấy mask từ curvature và AO).
   - Normal phải có chi tiết nhỏ: sợi vải, lỗ chân lông, vân gỗ, vảy vỏ cây.
2. **Không có bản high-poly.** Quy trình chuẩn:
   - sculpt bản chi tiết (Multires): nếp vải, khối cơ, búi lông, vân vỏ;
   - retopo xuống đúng ngân sách tam giác;
   - **bake** normal, AO và curvature từ high sang low (Cycles, dùng cage).
3. **UV lãng phí.** Khoảng một nửa texture đang là nền đen. Yêu cầu:
   - dùng ít nhất 70% diện tích;
   - mật độ texel đồng đều: nhân vật khoảng 10 px/cm (1024 px mỗi mét), mặt người được ưu tiên gấp đôi.
4. **Shading.** Bật smooth shading và weighted normals; không để mặt cong bị gãy thành từng mặt phẳng.
   Con chó trong game đang gãy góc như cáo low-poly.
5. **Vật lớn (cầu, công trình, nhà) không dùng một atlas duy nhất.** Dùng:
   - vật liệu **lặp theo mét** trên UV0 (`texCoord: 0`);
   - **AO nướng riêng** trên UV1: một ảnh xám, gán vào `occlusionTexture` với `texCoord: 1`;
   - ảnh metallic-roughness lặp trên UV0. Với vật lớn được tách AO khỏi ORM như vậy.

   Game hỗ trợ đúng 2 bộ UV, nên bỏ `TEXCOORD_2` (cây đang có).
6. **Texture cho LOD:** LOD1 dùng ảnh thu nhỏ còn một nửa, LOD2 một phần tư. LOD xa không cần ảnh 1024.
7. **Kiểm tra trong game thật**, không chỉ trong Three.js. Game có sẵn camera duyệt hình:

   ```sh
   cd examples/game-hanoi-city
   SKIP_OPT=1 ONLY=gl ./build.sh
   python3 -m http.server 8000 --directory dist
   ```

   Mở các địa chỉ:
   - `http://localhost:8000/?gl&look=lineup`: Minh, tên trộm và chó đứng ở đầu cầu Thê Húc;
   - `?gl&look=thehuc`: nhìn dọc cầu từ bờ hồ;
   - `?gl&look=bridge`: nhìn cầu từ trên mặt hồ;
   - `?gl&look=tree`: cây xà cừ gần nhất.

   Chụp các góc này và đặt cạnh ảnh tham chiếu. Món chỉ coi là đạt khi ảnh trong game đứng cạnh ảnh thật
   mà không bị lộ là đồ 3D rẻ tiền.

## B. Làm lại năm món của lô 01

### Minh (`people/minh.glb`) và tên trộm (`people/trom_01.glb`)

- **Thân:** đang là thân "siêu anh hùng" vai rộng, cơ bắp cuồn cuộn. Cần thân sinh viên Việt 20 tuổi,
  gầy: cao 1,70–1,74 m, khoảng 60 kg, vai khoảng 40 cm.
  - Dựng thân từ MakeHuman (CC0, đang dùng cho đầu): nam châu Á, 20 tuổi, cơ bắp thấp, cân nặng thấp.
  - Đặt đúng T-pose của khung mẫu.
  - Chuyển trọng số bằng **Data Transfer** (Vertex Groups, nearest face interpolated) từ thân mẫu sang
    thân mới, rồi chỉnh tay ở vai, khuỷu, hông.
  - **Giữ nguyên** Armature, tên xương, thứ tự joint, inverse bind.
- **Quần áo là mô hình riêng có độ dày,** không vẽ lên da:
  - Solidify 4–6 mm, có đường may, cổ áo, cổ tay, gấu quần;
  - mô phỏng vải (Blender Cloth) trên thân ở tư thế A, apply, sculpt thêm nếp, rồi bake normal;
  - bỏ các mảng túi áo phẳng đang nổi lơ lửng và mảng trắng phẳng giữa ngực.
- **Minh mặc:**
  - áo khoác jean đen: vải chéo, chỉ nổi, nút kim loại, mặc mở;
  - áo phông trắng cotton, thấy ở cổ và thân trước;
  - quần jean tối có túi, mòn ở đầu gối;
  - sneaker có đế, dây, nếp gãy ở mũi giày.
- **Tên trộm mặc:**
  - áo gió có nếp và khoá kéo;
  - khẩu trang vải có nếp;
  - mũ lưỡi trai có lưỡi cong.

  Dáng người khác Minh (gầy cao hoặc to bè) để nhìn là phân biệt được.
- **Đầu:**
  - da tông Việt, có lỗ chân lông trong normal;
  - nhãn cầu riêng, lông mày;
  - **tóc làm bằng hair cards** (30–60 dải, `alphaMode: MASK`, ảnh sợi tóc có alpha), thay khối tóc liền
    như mũ.
- **Kiểm tra không xuyên vải** ở 6 clip: `Idle_Loop`, `Sprint_Loop`, `Crouch_Fwd_Loop`, `Punch_Cross`,
  `Roll`, `Death01`.

### Chó cỏ (`animals/cho_co.glb`)

- **Hình khối** đang giống cáo đa giác thô, chân mảnh và dài. Chó cỏ Việt:
  - cao vai 45–50 cm, thân 60–70 cm;
  - ngực sâu, bụng thon, chân chắc;
  - **đuôi cong vồng lên lưng**, tai dựng hình tam giác, mõm vừa.
- **Cách làm:** sculpt theo giải phẫu → retopo khoảng 7–8 nghìn tam giác → bake normal (nhóm cơ, búi lông).
- **Lông:**
  - basecolor có biến thiên: lưng sẫm, bụng và mõm màu kem;
  - normal theo chiều lông;
  - còn ngân sách thì thêm vài "fur card" có alpha ở cổ và đuôi.
- **Động tác (30 fps, vòng lặp liền, gốc không trôi):**
  - `Walk` đi bốn nhịp, `Run` chạy nước kiệu;
  - `Idle` có thở, vẫy đuôi, động tai;
  - `Sit`;
  - `Bark` có giật đầu.

### Cầu Thê Húc (`landmarks/cau_the_huc.glb`)

- **Hình dáng:** giữ cong trên mặt bằng như bản hiện tại, thêm **mặt cầu vồng lên ở giữa**. Đối chiếu ảnh
  chụp thật để lấy độ vồng.
- **Lan can:**
  - con tiện gỗ;
  - tay vịn tròn;
  - trụ chính có đầu trụ.
- **Chân cầu:**
  - trụ gỗ kèm dầm ngang;
  - mặt hồ trong game ở y ≈ 0;
  - chân trụ chìm xuống khoảng y = −1;
  - đầu cầu phía bờ khớp mặt đường (y = 0).
- **Vật liệu theo mục A.5:**
  - gỗ sơn son đỏ hơi bóng (roughness 0,35–0,5), mòn lộ gỗ ở tay vịn, bẩn ở gần nước;
  - ván sàn có khe;
  - tất cả lặp theo mét, AO trên UV1.

### Xà cừ (`trees/xa_cu.glb`)

- **Hình khối** đang như cây ô dù: thân mảnh, cành toả từ một điểm như ngón tay, tán thưa. Xà cừ Hồ Gươm:
  - **thân thẳng to**, đường kính 0,8–1,5 m, phân cành ở độ cao 6–8 m;
  - rễ bạnh;
  - vỏ xám nâu bong vảy từng mảng;
  - tán dày, tròn, rộng 12–16 m, cao 18–25 m.
- **Cách làm:**
  - Geometry Nodes hoặc Sapling: thân, 4–6 cành chính, cành phụ;
  - **tán lá là 300–600 "cụm lá"**: mỗi cụm là một tấm có alpha chứa một cành nhỏ 15–30 lá kép, ảnh
    1024 kèm normal, màu biến thiên;
  - **chuyển pháp tuyến các tấm lá theo hình cầu tán** (Data Transfer từ một khối cầu) để tán sáng mềm
    như cây thật, không lấp lánh từng tấm;
  - lá `MASK`, hai mặt.
- **Vỏ:** ảnh quét vỏ cây lặp 2 m, kèm normal sâu.
- **LOD1:** ít cụm lá hơn nhưng cụm to hơn.
- **Billboard:** giữ cách đệm hiện tại; game đã đọc pivot.

## C. Lô 02: vẽ thêm (theo thứ tự này; quy cách đúng như `ASSETS.md`)

1. **Vật liệu nền** (`materials/<ten>/`, mỗi thư mục gồm `basecolor.png`, `normal.png`, `orm.png`,
   `info.json` dạng `{"metres": 2.0}`):
   - `via_he_da_xam` (đá xám lát vỉa hè như ảnh mẫu);
   - `duong_nhua` (nhựa đường có vết bẩn);
   - `co`;
   - `gach_lat_ho`;
   - `nuoc_ho` (chỉ cần `normal.png`, sóng gợn nhẹ).

   Năm món này thay mặt đất của cả quận.
2. **Mặt tiền nhà ống** `buildings/mat_tien_tret_01…04` (4,0 × 3,6 m) và `buildings/mat_tien_tang_01…04`
   (4,0 × 3,3 m). Đây là món đổi diện mạo cả phố cổ.
   - Game ốp mô-đun lên mọi mặt nhà nhìn ra phố.
   - Mặt tường ở z = 0, nhìn ra +Z, gốc ở góc dưới trái; hai cạnh trái phải phải khớp liền khi đặt cạnh nhau.
   - Có cửa cuốn, quầy hàng, ban công sắt, chậu cây, điều hoà, quần áo phơi, dây điện.
   - Đèn trong tiệm và cửa sổ dùng `emissive`. Biển hiệu để trống.
   - Mỗi mô-đun có LOD1 (dưới 800 tam giác).
3. `landmarks/thap_but_dai_nghien.glb`: Tháp Bút, Đài Nghiên và cổng ở đầu cầu Thê Húc phía bờ. Đây là điểm
   nhìn chính của ảnh mẫu.
   - **Gốc toạ độ:** đúng điểm đầu cầu chạm bờ, ở y = 0.
   - **−Z chỉ dọc theo cầu về phía đảo;** cụm tháp và cổng nằm phía +Z, tức phía đường Đinh Tiên Hoàng.
4. **Đồ ven hồ và vỉa hè:**
   - `street/hang_rao_ho.glb` (2 m, lặp được);
   - `street/bo_via.glb` (1 m, lặp được);
   - `street/cot_bien_ten_pho.glb` (mặt chữ để trống, vật liệu tên `sign_text`);
   - `street/den_duong_01.glb`.
5. `landmarks/den_ngoc_son.glb`, `landmarks/thap_rua.glb`. Tháp Rùa trong game đang là khối hộp cao 17,5 m;
   tháp thật khoảng 8 m.
6. **Nhân vật và đồ trong màn chơi:**
   - `street/xe_may_01_chay.glb` (xe chung chung kèm người lái đội mũ bảo hiểm, không logo);
   - `people/long_cho.glb` (lồng sắt, nút cửa tên `door`);
   - `people/nguoi_di_duong_01…03.glb` (khung 65 xương);
   - `street/dep_to_ong.glb`, `street/so_tay.glb`, `street/thung_hang.glb`.

## D. Cách nộp

- **Nộp đè đúng tên file cũ** khi làm lại; mỗi món cập nhật dòng của nó trong `manifest.json`.
- **Giữ repo nhẹ:**
  - không cần commit thư mục `textures/` nguồn, vì ảnh đã nằm trong GLB (giữ nguồn ở nơi khác);
  - ảnh xem thử dạng JPG, tối đa 1280 px.
- Mỗi món kèm một ảnh chụp **trong game** từ camera duyệt hình (mục A.7), đặt cạnh ảnh xem thử.
- Không có logo, tên thương hiệu hay khuôn mặt người thật. Không lấy gì từ game khác.

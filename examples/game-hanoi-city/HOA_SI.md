# Gửi họa sĩ: mọi thứ cần để làm tài nguyên cho Hanoi City: Giải Cứu Cún

Một file duy nhất: dán toàn bộ cho agent làm tài nguyên. Gồm bốn phần:

1. **Việc cần làm bây giờ** (lô 06): những gì còn thiếu để game hoàn chỉnh, theo thứ tự người chơi gặp, và ngân sách
   để game chạy mượt trên trình duyệt.
2. **Kỹ thuật chuẩn**: cách làm để hình trông thật, không như đồ nhựa.
3. **Ghi chú khi làm lại các món cũ** và **kỹ thuật cho đồ lặp nhiều lần**.
4. **Đặc tả kỹ thuật đầy đủ**: tên file, kích thước, khung xương, cách nộp.

Repo: `vansyson1308/craft-anything`, thư mục `examples/game-hanoi-city/`. Game chạy thử:
https://vansyson1308.github.io/craft-anything/

<!-- Tệp này được ghép tự động bằng tools/make_hoa_si.py; sửa ở các file nguồn. -->

# Phần 1. Việc cần làm bây giờ (lô 06)

## Đã xong, không làm lại

Đã có trong game và đang chạy trên trang:
- lồng chó (gỉ thành vệt, khoá, khay báo, dây buộc, khớp `door_hinge`: cửa mở 110° khi thả chó);
- đèn pin;
- đèn lồng Hàng Mã 01–03;
- kính đèn đường `lamp_glass`;
- cửa sổ `window_glow` và dải đèn quầy `shop_light` (tự tắt ban ngày, bật ban đêm);
- quầy hàng nhô ra không quá 0,6 m;
- bàn ghế nhựa;
- 4 tiếng sủa và tiếng rên của chó (phát 3D từ lồng);
- tiếng bấm nút và tiếng chuông khi có lời mẹo.

## Ngân sách để game chạy mượt trên trình duyệt

Game tải mọi thứ qua mạng rồi đẩy lên GPU từng phần mỗi khung hình. Lúc bước vào một phố mới, hơn
1,7 GB texture có thể cùng lúc xếp hàng chờ đẩy lên. Món càng nhẹ thì phố hiện ra càng nhanh, và máy yếu càng ít
giật. Giữ đúng các mức này:

| Loại | Tam giác LOD0 / LOD1 / LOD2 | Texture LOD0 / LOD1 / LOD2 | Vật liệu |
|---|---|---|---|
| Đồ nhỏ (ghế, chậu, bát, xe máy) | ≤ 2.000 / 500 / — | 512 / 256 px | 1 atlas cho cả món |
| Đồ vừa (xe đẩy, cột đèn, đèn lồng) | ≤ 4.000 / 1.200 / — | 1024 / 512 px | ≤ 2 |
| Mặt tiền (một mô-đun 4 m) | ≤ 6.000 / 1.500 / — | 1024 / 512 px, dùng chung giữa các biến thể | ≤ 3 (tường, khung, kính) |
| Công trình (Ô Quan Chưởng, chợ, nhà thờ) | ≤ 15.000 / 5.000 / 1.500 | 2048 / 1024 / 512 px | ≤ 4 |
| Người, chó | ≤ 12.000 / 4.000 / 1.200 | 1024 / 512 / 256 px | ≤ 3 |

- **Mỗi cấp LOD có texture của riêng nó và nhỏ hơn cấp trên.** Đừng dùng lại texture 1K cho LOD xa.
- **Xếp UV kín atlas, đạt ít nhất 70%.** Đèn lồng hiện mới dùng 15–19% atlas: phí ba phần tư bộ nhớ. Nếu không xếp kín
  được ở mật độ đã chọn, hạ atlas xuống một nấc (1024 → 512).
- **Tên vật liệu phát sáng phải đúng:**
  - `lamp_glass`: bóng đèn, chao đèn, đèn lồng;
  - `window_glow`: cửa sổ;
  - `shop_light`: dải đèn quầy.

  Game dựa vào tên này để tắt đèn ban ngày và bật ban đêm. Phần kim loại, gỗ, tường không bao giờ phát sáng.

## A. Chương 1 phải hoàn hảo (ưu tiên số 1)

| # | File | Cần gì | |
|---|---|---|---|
| A1 | `landmarks/o_quan_chuong.glb` | Xem mục A1 bên dưới. | [tự vào game] |
| A2 | `street/den_ven_ho.glb` | Hoàn thiện bản nghiên cứu ở PR #27: cột gang cao 3,8 m, bóng ở độ cao 3,5 m, gốc ở chân cột. Bóng đèn dùng vật liệu `lamp_glass`. Bake AO riêng cho món này, LOD1. Game đã có sẵn vị trí đèn quanh hồ. | [tự vào game] |
| A3 | `people/trom_02.glb` | Tên trộm thứ hai, to con hơn `trom_01`. Áo khoác khác màu, mũ lưỡi trai, cùng khung 65 xương. Chương 1 có hai tên. | [tự vào game] |
| A4 | `animals/cho_co.glb` (làm lại) | Chó cỏ Việt 12–18 kg, lông vàng. Lông chải theo hướng mọc, không gãy góc ở khớp. Thêm động tác `Wag` (vẫy đuôi, lặp) và `Jump` (chồm lên, hai chân trước chạm khoảng 1 m, tức tới đùi Minh). Đây là khoảnh khắc cảm xúc chính của game. Bản nghiên cứu ở PR #33 là điểm khởi đầu tốt. | [game nối sau: `Jump` khi được thả] |
| A5 | `landmarks/cau_the_huc.glb` (bổ sung) | Ban đêm cầu đang tối đen. Thêm các bóng đèn dọc lan can theo ảnh tham khảo, vật liệu `lamp_glass`. Đặt mỗi bóng một node rỗng tên `glow_01`, `glow_02`, … tại tâm bóng; game sẽ thắp đèn thật ở đó. | [game nối sau] |
| A6 | `buildings/mat_tien_tang_01…04.glb` (sửa) | Mỗi biến thể một kiểu cửa sổ sáng khác nhau. Hiện cả bốn biến thể cùng kiểu "trái sáng có rèm, phải tối", nên cả dãy phố lặp lại. Hãy trộn: ô tối, ô vàng ấm, ô trắng lạnh, ô có ánh tivi xanh, rèm đủ màu. Khoảng 40% ô sáng. | [tự vào game] |

### A1. Ô Quan Chưởng: cách làm

Bản phác ở PR #24 có đúng bố cục (vòm giữa, hai vòm bên, vọng lâu), nhưng còn ba chỗ chưa đạt.

1. **Màu sai.**
   - Cổng thật trát vữa vôi màu sáng, đã ngả xám và vàng theo năm tháng; gạch chỉ lộ ra ở chỗ vữa bong.
   - Làm vữa là lớp chính. Đặt mask gạch lộ theo AO và theo cạnh: mép vòm, chân tường, góc.
   - Thêm rêu xanh sẫm ở chân tường và mặt trên gờ, và vệt ố chảy dọc từ gờ mái xuống.
2. **Nặng.**
   - Mỗi LOD hiện khoảng 9 MB vì dùng lại ảnh quét 1K ở mọi cấp. Theo ngân sách ở trên: LOD0 2048, LOD1 1024, LOD2 512.
   - Gộp còn tối đa 4 vật liệu: vữa và gạch chung một vật liệu, ngói, gỗ, kim loại.
3. **Chi tiết.**
   - Mái ngói cong ở các góc (đầu đao), không phải mái chóp phẳng.
   - Bờ nóc có gờ.
   - Cửa gỗ hai cánh ở vòm giữa, mở hẳn vào trong.
   - Biển đá phía trên vòm để trống, không khắc chữ.

Lối đi giữa rộng khoảng 3 m, cao 3 m. Phía game sẽ mở đúng khoảng đó trên bản đồ đi bộ khi mô hình đạt, nên hai
vòm bên và tường vẫn chặn người chơi.

Kiểm tra: `?gl&look=lineup` và `?gl&time=night`, đi từ phố Ô Quan Chưởng xuyên qua cổng.

## B. Kể chuyện bằng đồ vật (đặt quanh lồng chó, không cần chữ)

| # | File | Cần gì | |
|---|---|---|---|
| B1 | `street/thong_long_roi.glb` | Cây thòng lọng bắt chó vứt dưới đất, dây thừng rối. ⚙ gậy, sculpt dây | [game nối sau] |
| B2 | `street/bat_nuoc_do.glb`, `street/vong_co.glb` | Bát inox lật úp; vòng cổ chó có thẻ tên để trống (game in tên chó lên). ⚙ | [game nối sau] |
| B3 | `street/to_roi_tim_cho.glb` | Tờ "Tìm chó lạc" dán trên cột điện, giấy nhàu, góc bong. Chữ để trống. | [game nối sau] |
| B4 | `decals/vet_banh_xe.png`, `decals/vet_ban_chan.png` | Vết bánh xe máy, dấu chân chó trên đất ẩm. PNG có kênh alpha, 1024 px, lặp được theo chiều dài. | [game nối sau] |

## C. Phố đông như ngoài đời

| # | File | Cần gì | |
|---|---|---|---|
| C1 | `street/xe_may_01_dau.glb` … `xe_may_04_dau.glb` | **Món đổi cảm giác phố nhiều nhất.** Game đã có sẵn chỗ dựng xe trước một nửa số cửa hàng, nhưng chưa có xe nào. Xe số, xe ga, xe cub, xe điện. Không logo, biển số trống. Gốc ở giữa hai bánh, đầu xe hướng +Z, chân chống ra. | [tự vào game] |
| C2 | `street/chau_cay.glb`, `street/thung_rac.glb` | Chậu sứ hoặc chậu nhựa có cây thật (lá dùng alpha mask), thùng rác nhựa xanh có nắp. | [tự vào game] |
| C3 | `street/day_dien_ngang.glb`, `street/cot_dien_day.glb`, `street/co_do.glb`, `street/dieu_hoa_cuc_nong.glb`, `buildings/mai_bon_nuoc.glb` | Dây điện võng ngang ngõ, cột điện, cờ đỏ sao vàng, cục nóng điều hoà, bồn nước inox trên mái. | [game nối sau] |
| C4 | `street/xe_banh_mi.glb`, `street/ganh_hang_rong.glb`, `street/xe_dap_tho.glb` | Xe bánh mì có tủ kính (của cô Hằng), gánh hàng rong, xe đạp chở hàng. | [game nối sau] |
| C5 | `materials/via_he_gach_do/` | Gạch đỏ 30 × 30 cm lát vỉa hè phố cổ: viên vỡ, viên lệch màu, vết dầu, rêu mép. | [game nối sau] |
| C6 | `buildings/mat_tien_tret_05…08.glb`, `mat_tien_tang_05…10.glb`, `mat_tien_phap_tret_01…03.glb`, `mat_tien_phap_tang_01…03.glb` | Thêm biến thể mặt tiền để phố bớt lặp. Tường nhiều màu (vàng, hồng, xanh ngọc, kem) có vết ố mưa; mái hiên bạt; cửa cuốn có biển hiệu để trống. Mặt tiền kiểu Pháp có cửa chớp xanh và ban công sắt. Dùng `window_glow` và `shop_light`, quầy nhô ra ≤ 0,6 m. | [tự vào game] |
| C7 | `street/den_long_01…03.glb` (sửa) | Xếp lại UV cho kín atlas (≥ 70%) hoặc hạ xuống 512 px. Thân đèn ở LOD0 dùng 24 cạnh quanh trục thay cho 12, để hết gãy góc. Dây treo dày hơn một chút (8 mm) cho dễ thấy. | [tự vào game] |

## D. Nhân vật và công trình cho các chương sau

- **[tự vào game]**:
  - người: `people/tu_long.glb`, `people/bay_thong_long.glb`, `people/co_hang.glb`;
  - công trình: `landmarks/cho_dong_xuan.glb`, `landmarks/nha_tho_lon.glb`, `landmarks/nha_hat_lon.glb`;
  - cây: `trees/sau.glb`, `trees/bang.glb`, `trees/lieu_ru.glb`, mỗi cây kèm một ảnh billboard như cây xà cừ đã có.
- **[game nối sau]**:
  - người và chó: `people/nguoi_di_duong_04…06.glb`, `animals/cho_phu_quoc.glb`, `animals/cho_bac_ha.glb`;
  - Long Biên và tàu: `landmarks/ga_long_bien.glb`, `landmarks/cau_long_bien_nhip.glb` (⚙ dàn thép),
    `street/toa_tau.glb`, `street/dau_may.glb` (⚙).

## E. Âm thanh còn thiếu

Âm thanh làm nên một nửa cảm giác "thật". Quy cách:
- OGG Vorbis, 44,1 hoặc 48 kHz;
- tiếng ngắn mono, tiếng nền và nhạc stereo;
- tiếng ngắn đo theo đỉnh: −1 dBTP;
- tiếng nền và nhạc khoảng −16 LUFS;
- nền nhiễu dưới −50 dB;
- không độn im lặng vào cuối file;
- file lặp phải nối liền: cắt ở điểm qua 0, crossfade 200 ms.

Chỉ dùng nguồn **CC0** hoặc tự làm, và ghi từng file vào `SOURCES.md`. Nếu cần tải từ trang có đăng nhập, nhờ
người giao việc tải hộ bản WAV hoặc FLAC.

| Nhóm | File |
|---|---|
| Tiếng nền | `pho_dem_loop.ogg` (phố đêm: xe xa, côn trùng, tivi nhà ai), `pho_ngay_loop.ogg` (còi, rao hàng, xe máy), `ho_guom_loop.ogg` (gió, nước, chim) |
| Nhân vật | `buoc_chan_gach_01…04.ogg`, `buoc_chan_nhua_01…04.ogg` (nhựa đường), `lan.ogg`, `dam_trung.ogg`, `nem_dep.ogg`, `dep_trung.ogg` |
| Chó | `cho_mung.ogg` (rên mừng, sủa vui khi được thả), và một tiếng sủa sạch thay cho `cho_sua_02.ogg` (nền nhiễu −49 dB, hơi cao) |
| Trộm | `trom_nghi.ogg` ("Ai đấy?"), `trom_phat_hien.ogg` ("Đứng lại!"), `trom_mat_dau.ogg` ("Chạy đâu rồi?"). Giọng nam, tiếng Việt, tự thu hoặc do người đọc đồng ý cho dùng. |
| Vật | `long_mo.ogg` (cửa sắt kẽo kẹt rồi lách cách), `trom_bi_troi.ogg`, `xe_may_chay_loop.ogg`, `coi_xe.ogg`, `tau_coi.ogg`, `tau_chay_loop.ogg` |
| Nhạc | `nhac_lan.ogg` (lén lút, nhẹ, lặp), `nhac_duoi.ogg` (bị đuổi, dồn dập), `nhac_thoat.ogg` (thoát được, ngắn) |

Game sẽ nối toàn bộ: âm thanh 3D theo vị trí, nhạc chuyển theo trạng thái lén hay bị đuổi, tiếng bước chân theo
mặt đường.

## F. Hình 2D

- `2d/man_hinh_cho_ngang.jpg` (1920 × 1080): tranh màn hình chờ. Minh và Bơ trên phố Hàng Mã ban đêm, đèn lồng
  đỏ, ánh sáng điện ảnh. Không dùng ảnh người thật.
- `2d/icon_cho.png`, `2d/icon_dep.png`, `2d/icon_balo.png`, `2d/icon_tim.png` (128 px, nền trong): biểu tượng
  cho thanh trạng thái.
- `2d/logo.png`: chữ "GIẢI CỨU CÚN" kiểu biển hiệu phố cổ.

## Thứ tự nộp (mỗi đợt một PR nhỏ)

1. **A1, C1, A3.** Cổng thoát của Chương 1, xe máy dựng kín phố, tên trộm thứ hai.
2. **A2, A5, A6, C2, C7** và âm thanh nhóm Tiếng nền, Nhân vật, Vật.
3. **A4** (chó và động tác), **B**, **C3–C6**.
4. **D**, nhạc, **F**.

## Kiểm tra trước khi nộp

- glTF Validator: 0 lỗi, 0 cảnh báo.
- Đúng ngân sách tam giác và texture ở trên.
- Emissive kiểm trong Blender với World strength = 0: chỉ các vật liệu `lamp_glass`, `window_glow`, `shop_light`
  được sáng.
- Chụp trong game (build như Phần 2, ý 7), nộp vào `review/<số lô>/`, cả ban ngày (`&time=day`) lẫn ban đêm
  (`&time=night`):
  - `?gl&look=hangma`: phố Hàng Mã, đèn lồng, mặt tiền;
  - `?gl&look=streetlife`: đồ vỉa hè, xe máy;
  - `?gl&look=lineup`: nhân vật;
  - `?gl&look=cage`, thêm `&door=open`: lồng chó;
  - Chương 1 từ đầu tới cổng Ô Quan Chưởng.
- Không logo hãng, không chữ thương hiệu thật, không mặt người thật, không lấy hình hay tiếng của game khác.

# Phần 2. Kỹ thuật chuẩn (áp dụng cho mọi món)

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

# Phần 3. Ghi chú khi làm lại và đồ lặp nhiều lần

## Ghi chú khi làm lại các món cũ

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
- **Vật liệu theo Phần 2, ý 5:**
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

## Kỹ thuật cho đồ lặp nhiều lần

- **Dựng đồ cứng bằng code:** với vật có hình khối rõ ràng, nên dựng bằng
  [sand3d](https://github.com/Sandflow-ai/sand3d) (Apache-2.0, chạy trên Blender `bpy`, xuất GLB). Những vật đó
  là ghế đẩu, bàn nhựa, xe đẩy, cục nóng điều hoà, bồn nước, cột đèn, lồng chó, khung đèn lồng.
  - Mỗi món là một chương trình ngắn, các phần có tên.
  - Cùng chương trình cho ra cùng một mô hình, nên dễ sửa và dễ làm biến thể (đổi kích thước, số chân, màu).
  - Sau đó vẫn UV, gán vật liệu quét CC0 và bake AO như Phần 2.
  - Ghi `sand3d` cùng phiên bản trong `made_with` và `SOURCES.md`.
  - Không dùng cho người, chó, cây: những thứ đó vẫn sculpt và retopo.

- **Texture dùng chung:** gom các món cùng chất liệu (nhựa, sơn xe, kim loại) vào một atlas 1024 px cho
  cả nhóm. Như vậy game vẽ hàng trăm chiếc xe với rất ít lần đổi vật liệu.
- **Biến thể màu không cần hình mới:** cùng một khung xe, đổi màu thân bằng atlas, nộp thành
  `xe_may_0N_dau.glb` riêng.
- **Mặt người nhìn:** ghế, bàn, xe nhìn từ ngang tầm mắt 1,6 m ở 3–5 m. Mép bo tròn (bevel nhỏ rồi bake
  normal), nhựa có độ bóng khác nhau giữa chỗ mới và chỗ mòn, xe có bụi bám ở dưới.
- **Kiểm tra trong game:** build như Phần 2, ý 7, rồi mở `?gl&look=streetlife` (bộ bàn
  ghế gần nhất, ngang tầm mắt; thêm `&time=night` để xem ban đêm), hoặc chơi Chương 1. Đi dọc vài con phố, chụp lại và nộp kèm trong `review/04/`.
- Không logo hãng xe thật, không chữ của thương hiệu có thật, không lấy hình từ game khác.
- Mỗi đợt nộp nhỏ một PR. Với đồ phố, đợt đầu nên là xe máy dựng (Phần 1, mục C1): chỉ riêng chúng đã đổi hẳn cảm giác của phố.

# Phần 4. Đặc tả kỹ thuật đầy đủ

Trong phần này, "mục N" là mục N của Phần 4.

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
| `ghe_nhua_xanh.glb`, `ghe_nhua_do.glb`, `ban_nhua.glb` | Ghế đẩu nhựa (cao 0,25–0,45 m), bàn nhựa vỉa hè (khoảng 0,6 × 0,6 × 0,5 m). Pivot giữa đáy. Game xếp 3–4 ghế quanh một bàn trước khoảng 1/5 số cửa hàng. |
| `xe_may_01…04.glb` | Xe máy số và tay ga **chung chung, không logo hãng**. Có bản đậu (`_dau`) và bản đang chạy kèm người lái đội mũ bảo hiểm (`_chay`). Bản `_dau`: dựng chân chống, hơi nghiêng, dài khoảng 1,9 m; pivot ở mặt đất giữa hai bánh, **đầu xe hướng +Z**. Game dựng 1–3 chiếc, đầu quay vào tường, trước khoảng một nửa số cửa hàng. |
| `xe_dap_tho.glb`, `ganh_hang_rong.glb`, `xe_banh_mi.glb` | Xe đạp chở hàng, gánh hàng rong, xe bánh mì của cô Hằng. |
| `thung_rac.glb`, `chau_cay.glb`, `cot_dien_day.glb` | Thùng rác, chậu cây (pivot giữa đáy; game đặt sát tường cạnh cửa hàng), cột điện chằng chịt dây. |
| `day_dien_ngang.glb` | Một bó dây điện, dây cáp chùng võng, dài 8 m, căng ngang ngõ ở độ cao 5–7 m, lặp được. |
| `co_do.glb` | Cờ đỏ sao vàng cắm cán nghiêng trên mặt tiền, có xương hoặc morph để bay nhẹ. Pivot ở chân cán. |
| `dieu_hoa_cuc_nong.glb`, `bien_hieu_dung.glb` | Cục nóng điều hoà treo tường; biển hiệu dựng trên vỉa hè (mặt chữ để trống, vật liệu `sign_text`). |
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

`via_he_da_xam` (vỉa hè lát đá xám như ảnh mẫu), `via_he_gach_do` (gạch đỏ lát vỉa hè phố cổ, có viên vỡ, mòn, vết dầu), `bo_via_granite`, `duong_nhua` (có vết bẩn),
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

## 12. Ban đêm (Chương 1 "Đêm mất Bơ")

Chương 1 giờ chơi vào ban đêm. Game tự lo ánh trăng, đèn đường chiếu sáng thật (theo vị trí các cột đèn đã
nộp), đèn lồng Hàng Mã, mặt đường ướt phản chiếu đèn, đèn pin trong tay Minh. Thử bằng `?time=night`, ví dụ
`?gl&time=night&look=lineup`. Ban đêm làm lộ ra những chỗ dưới đây, nên cần nộp hoặc sửa:

| Đang thấy ban đêm | Cần gì | File |
|---|---|---|
| Cả cột đèn đường phát sáng: vật liệu kim loại của nó có `emissiveFactor` 0,65 kèm `emissiveTexture` | Chỉ **chao đèn** phát sáng. Tách thành vật liệu riêng tên `lamp_glass` (emissive vàng ấm, khoảng 2.700 K). Thân, cần, đế: emissive = 0. Bóng đèn nằm ở khoảng (0; 4,45; 1,15) m, game đặt nguồn sáng ở đó. | `street/den_duong_01.glb` (cả `_lod1`) |
| Mọi chi tiết kim loại của mặt tiền nhà phát sáng nhẹ (cùng lỗi emissive 0,65 trên vật liệu "joinery metal") | Kim loại: emissive = 0. Cửa sổ và cửa hàng dùng vật liệu riêng `window_glow`. Emissive map của nó: khoảng 40% ô cửa sáng vàng ấm, rèm che một phần, số còn lại tối. Cửa cuốn đóng thì tối. | `buildings/mat_tien_tret_*`, `buildings/mat_tien_tang_*` |
| Đèn lồng Hàng Mã là quả cầu màu | Đèn lồng vải lụa đỏ và vàng, khung tre, tua rua; có emissive bên trong; pivot ở móc treo trên đỉnh | `street/den_long_01…03.glb` |
| Đường dạo quanh Hồ Gươm tối om: chưa có cột đèn | Cột đèn gang trang trí ven hồ, cao khoảng 3,8 m, một chao đèn trên đỉnh (vật liệu `lamp_glass`, sáng ở khoảng 3,5 m). Pivot giữa đế, đối xứng. Game đặt mỗi 25 m dọc đường dạo, và chỉ bật đèn khi đã có file này. | `street/den_ven_ho.glb` (+ `_lod1`) |
| Đèn pin chỉ có ánh sáng, không có vật | Đèn pin cầm tay nhỏ (dài 0,15 m, dưới 1.500 tam giác). Pivot ở chỗ nắm, mặt kính nhìn ra +Z, kính có emissive. Game gắn vào xương `hand_r`. | `people/den_pin.glb` |
| (tuỳ chọn) Đường ướt đều một màu | Bản đồ vũng nước: ảnh xám 2048×2048 lặp được, trắng ở chỗ đọng nước. Game giảm độ nhám ở đó. | `materials/duong_nhua/wet_mask.png` |

Cách kiểm tra trước khi nộp: mở file trong Blender, chọn Viewport Shading > Rendered (Eevee), tắt World (độ sáng
0). Chỉ những chỗ nói trên được phát sáng.

# Lời nhắn cho họa sĩ (agent làm tài nguyên): lô 06, mọi thứ còn thiếu

Dán toàn bộ phần dưới đây cho agent. Lô này thay cho lô 05: ở đây chỉ còn **những gì chưa có trong game**,
sắp theo thứ tự người chơi gặp. Mỗi món ghi rõ:
- **[tự vào game]**: nộp đúng tên file là game dùng ngay;
- **[game nối sau]**: phía game sẽ viết code để dùng nó khi bạn nộp.

---

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

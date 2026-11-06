# Lời nhắn cho họa sĩ (agent làm tài nguyên): lô 05, bộ tài nguyên để game hoàn chỉnh

Dán toàn bộ phần dưới đây cho agent, kèm `ASSETS.md`, `AGENT_PROMPT_02.md` (mục A: kỹ thuật chuẩn) và
`DESIGN.md`. Lời nhắn này gom **mọi thứ còn thiếu** sau lần thiết kế lại lối chơi (PR #14). Danh sách chi
tiết của lô 03 và lô 04 vẫn đúng; ở đây chúng được sắp lại theo đúng thứ tự người chơi gặp.

---

## Cách hai bên làm việc

Game chạy trên trình duyệt theo cách các đội lớn đưa game AAA lên web: **họa sĩ làm đủ bộ tài nguyên, kỹ sư
lo cho nó chạy mượt trên web**. Phía game đã có:
- chỗ đặt từng món theo bản đồ thật;
- LOD theo khoảng cách;
- đóng gói và gộp texture;
- ánh sáng ngày và đêm;
- cột sáng dẫn đường, tầm nhìn của trộm, lời mẹo.

Còn thiếu là **hình và tiếng**. Hiện game có 44 file hình thật (kể cả texture nền); phần còn lại vẫn là khối hộp, quả cầu, hoặc trống
không. Game **chưa có một âm thanh nào**.

## Dựng đồ cứng bằng sand3d

[sand3d](https://github.com/Sandflow-ai/sand3d) (Apache-2.0) cho phép viết vật thể 3D thành chương trình Python
ngắn trên Blender `bpy`:
- mỗi phần có tên;
- có khớp (joint) cho phần chuyển động;
- xuất thẳng ra GLB;
- cùng chương trình luôn cho ra đúng một mô hình.

Dùng nó cho **mọi vật có hình khối rõ ràng** (đánh dấu ⚙ trong bảng). Quy trình mỗi món:

1. Viết chương trình sand3d, mỗi món một file `art-src/sand3d/<ten>.py` trong `examples/game-hanoi-city/`.
   Đặt ở đó, **không** đặt trong `assets/hq`. Kích thước tính bằng mét theo bảng dưới.
2. Phần chuyển động dùng `s3d.joint` với đúng tên trong bảng. Ví dụ cửa lồng chó là `door_hinge`. Game sẽ đọc
   khớp trong GLB để mở cửa khi người chơi giữ E.
3. Biến thể (màu, kích thước, số chân) làm bằng tham số của cùng chương trình, xuất thành các file `_02`, `_03`.
4. Trong cùng phiên Blender: UV, gán vật liệu từ ảnh quét CC0, bevel nhỏ rồi bake normal và AO (mục A của
   AGENT_PROMPT_02), tạo LOD1 (khoảng 1/3 số tam giác).
5. Xuất GLB vào `assets/hq/...`, cập nhật `manifest.json`, ghi `sand3d <phiên bản>` vào `made_with`, chạy glTF
   Validator.

Không dùng sand3d cho người, chó, cây, vải: những thứ đó vẫn sculpt, retopo và bake.

## A. Chương 1 phải hoàn hảo trước (người chơi nào cũng gặp)

| # | File | Cần gì | |
|---|---|---|---|
| A1 | `landmarks/o_quan_chuong.glb` | Cửa ô gạch cổ, mái cong, vọng lâu, lối đi xuyên qua; đây là lối thoát của Chương 1. Mặt bằng khoảng 11,5 × 17,8 m. Gạch quét CC0, rêu, vết ố. ⚙ cho khối chính, sculpt cho ngói và chi tiết. | [tự vào game] |
| A2 | `people/long_cho.glb` (làm lại) | Lồng sắt 0,9 × 0,7 × 0,7 m, song sắt sơn tróc gỉ, khoá. Phần `door` gắn khớp `door_hinge` ở cạnh trái, mở 0–110°. ⚙ | [game nối sau: mở cửa] |
| A3 | `street/den_long_01…03.glb` | Đèn lồng Hàng Mã (đỏ, vàng, hình quả trám), khung tre, vải lụa, tua rua, vật liệu `lamp_glass` bên trong. ⚙ khung + vải mỏng. | [game nối sau] |
| A4 | `street/den_duong_01.glb`, `buildings/mat_tien_*` (sửa) | Chao đèn dùng `lamp_glass`, cửa sổ và cửa hàng dùng `window_glow`. Trên bản đang chạy, cả phần kim loại tự phát sáng. PR #11 (bản nháp) tắt nó đi nhưng chưa có vật liệu thay thế, nên ban đêm sẽ tối hẳn: cần cả hai việc cùng lúc. **Quầy hàng của mặt tiền tầng trệt nhô ra không quá 0,6 m**: hiện quầy nhô khoảng 1 m, nhân vật và camera lấn vào. | [tự vào game] |
| A5 | `street/den_ven_ho.glb` | Cột đèn gang ven Hồ Gươm, cao khoảng 3,8 m. ⚙ | [tự vào game] |
| A6 | `people/den_pin.glb` | Đèn pin cầm tay, gắn xương `hand_r`. ⚙ | [tự vào game] |
| A7 | `people/trom_02.glb` | Tên trộm thứ hai (to con), cùng khung 65 xương. Chương 1 có 2 tên. | [tự vào game] |
| A8 | `animals/cho_co.glb` (làm lại) | Lông có hướng, không gãy góc. Thêm động tác `Wag` (vẫy đuôi) và `Jump` (nhảy chồm lên chân Minh khi được cứu): đây là khoảnh khắc cảm xúc chính của game. | [game nối sau: Jump] |

## B. Kể chuyện bằng đồ vật (không cần chữ)

Đặt quanh lồng chó và dọc đường dẫn tới đó, để người chơi tự đoán chuyện gì đã xảy ra (DESIGN.md, giai đoạn 3).

| # | File | Cần gì | |
|---|---|---|---|
| B1 | `street/thong_long_roi.glb` | Cây thòng lọng bắt chó vứt dưới đất, dây thừng rối. ⚙ gậy, sculpt dây | [game nối sau] |
| B2 | `street/bat_nuoc_do.glb`, `street/vong_co.glb` | Bát inox lật úp; vòng cổ chó có thẻ tên trống (game in tên chó lên thẻ). ⚙ | [game nối sau] |
| B3 | `street/to_roi_tim_cho.glb` | Tờ "Tìm chó lạc" dán trên cột điện, giấy nhàu, góc bong. Chữ để trống, game in lên. | [game nối sau] |
| B4 | `decals/vet_banh_xe.png`, `decals/vet_ban_chan.png` | Vết bánh xe máy, dấu chân chó trên đất ẩm; PNG có kênh alpha, 1024 px, lặp được theo chiều dài. | [game nối sau] |

## C. Phố đông như ngoài đời (lô 04, làm tiếp)

| # | File | |
|---|---|---|
| C1 | `street/xe_may_01…04_dau.glb`: xe dựng, đầu hướng +Z, không logo. ⚙ khung, bánh, gương | [tự vào game] |
| C2 | Bàn ghế nhựa (đã có bản nháp): đạt UV 70%, bake AO | [tự vào game] |
| C3 | `street/chau_cay.glb`, `street/thung_rac.glb` (⚙ thùng) | [tự vào game] |
| C4 | `street/day_dien_ngang.glb`, `street/cot_dien_day.glb`, `street/co_do.glb`, `street/dieu_hoa_cuc_nong.glb`, `buildings/mai_bon_nuoc.glb` (⚙ trừ dây) | [game nối sau] |
| C5 | `street/xe_banh_mi.glb` (xe đẩy của cô Hằng, tủ kính), `street/ganh_hang_rong.glb`, `street/xe_dap_tho.glb` (⚙) | [game nối sau] |
| C6 | `materials/via_he_gach_do/`: gạch đỏ vỉa hè phố cổ | [game nối sau] |
| C7 | Mặt tiền `mat_tien_tret_05…08`, `mat_tien_tang_05…10`, kiểu Pháp `mat_tien_phap_*`: tường nhiều màu, mái hiên, cửa cuốn có biển hiệu trống, `window_glow` | [tự vào game] |

## D. Nhân vật và công trình cho các chương sau

- **[tự vào game]**
  - người: `people/tu_long.glb`, `people/bay_thong_long.glb`, `people/co_hang.glb`;
  - công trình: `landmarks/cho_dong_xuan.glb`, `landmarks/nha_tho_lon.glb`, `landmarks/nha_hat_lon.glb`.
- **[game nối sau]**
  - người và chó: `people/nguoi_di_duong_04…06.glb`, `animals/cho_phu_quoc.glb`, `animals/cho_bac_ha.glb`;
  - Long Biên và tàu: `landmarks/ga_long_bien.glb`, `landmarks/cau_long_bien_nhip.glb` (⚙ dàn thép),
    `street/toa_tau.glb`, `street/dau_may.glb` (⚙).
- **[tự vào game]** cây `trees/sau.glb`, `trees/bang.glb`, `trees/lieu_ru.glb` (sculpt, có billboard).

## E. Âm thanh (game đang câm hoàn toàn)

Âm thanh làm nên một nửa cảm giác "thật". Thư mục `assets/hq/audio/`:
- định dạng OGG Vorbis, 44,1 kHz;
- tiếng ngắn là mono, nhạc và tiếng nền là stereo;
- chuẩn hoá về khoảng −16 LUFS;
- file lặp phải nối liền không có tiếng tách.

Chỉ dùng nguồn **CC0** hoặc tự làm, và ghi từng file vào `SOURCES.md` (nguồn, tác giả, giấy phép).

| Nhóm | File |
|---|---|
| Tiếng nền | `pho_dem_loop.ogg` (phố đêm: xe xa, côn trùng, tivi nhà ai), `pho_ngay_loop.ogg` (còi, rao hàng, xe máy), `ho_guom_loop.ogg` (gió, nước, chim) |
| Nhân vật | `buoc_chan_gach_01…04.ogg`, `buoc_chan_nhua_01…04.ogg` (nhựa đường), `lan.ogg`, `dam_trung.ogg`, `nem_dep.ogg`, `dep_trung.ogg` |
| Chó | `cho_sua_01…03.ogg`, `cho_rit.ogg` (rên trong lồng, để người chơi nghe mà tìm), `cho_mung.ogg` |
| Trộm | `trom_nghi.ogg` ("Ai đấy?"), `trom_phat_hien.ogg` ("Đứng lại!"), `trom_mat_dau.ogg` ("Chạy đâu rồi?"); giọng nam, tiếng Việt, tự thu hoặc người đọc đồng ý cho dùng |
| Vật | `long_mo.ogg` (sắt kẽo kẹt), `trom_bi_troi.ogg`, `xe_may_chay_loop.ogg`, `coi_xe.ogg`, `tau_coi.ogg`, `tau_chay_loop.ogg` |
| Nhạc | `nhac_lan.ogg` (lén lút, nhẹ, lặp), `nhac_duoi.ogg` (bị đuổi, dồn dập), `nhac_thoat.ogg` (thoát được, ngắn) |
| Giao diện | `ui_bam.ogg`, `ui_meo.ogg` (lúc hiện lời mẹo) |

Game sẽ nối toàn bộ phần âm thanh: âm thanh 3D theo vị trí, nhạc chuyển theo trạng thái lén hay bị đuổi.

## F. Hình 2D

- `2d/man_hinh_cho_ngang.jpg` (1920 × 1080): tranh màn hình chờ. Minh và Bơ trên phố Hàng Mã ban đêm, đèn
  lồng đỏ, ánh sáng điện ảnh. Không dùng ảnh người thật.
- `2d/icon_cho.png`, `2d/icon_dep.png`, `2d/icon_balo.png`, `2d/icon_tim.png` (128 px, nền trong): biểu tượng
  cho thanh trạng thái.
- `2d/logo.png`: chữ "GIẢI CỨU CÚN" kiểu biển hiệu phố cổ.

## Thứ tự nộp (mỗi đợt một PR)

1. Mục A (Chương 1 hoàn hảo) và E (âm thanh). Hai thứ này đổi cảm giác game nhiều nhất.
2. Mục B và C.
3. Mục D và F.

## Kiểm tra trước khi nộp

- glTF Validator: 0 lỗi, 0 cảnh báo.
- Emissive kiểm trong Blender với World strength = 0.
- Chụp trong game, nộp vào `review/05/`:
  - `?gl&look=streetlife` (đồ vỉa hè);
  - `?gl&look=lineup` (nhân vật);
  - `?gl&look=cage`, thêm `&door=open` và `&time=night` (lồng chó; không cần bản vá camera riêng);
  - Chương 1 cả ngày (`&time=day`) lẫn đêm.
- Không logo hãng, không chữ thương hiệu thật, không mặt người thật, không lấy hình hay tiếng của game khác.

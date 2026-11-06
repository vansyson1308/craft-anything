# QC lô 05 (PR #16): kết quả duyệt và cách làm cho đạt

Gửi họa sĩ. Phía game đã mở từng file, đo âm thanh và xem ảnh kiểm tra.

Tóm tắt:
- lồng chó **đạt về kỹ thuật, chưa duyệt được về hình** vì chưa có ảnh cận cảnh;
- đèn pin **chưa đạt về hình**;
- tiếng chó **chưa đạt**, cần làm lại;
- tiếng giao diện **đạt**.

Ảnh của bạn còn xác nhận một điều có ích cho phía game: cột sáng, la bàn, lời mẹo và vùng tầm nhìn đỏ của trộm hiện
đúng trên máy có card đồ hoạ.

## 1. Lồng chó `people/long_cho.glb`

**Đạt:**
- 0,9 × 0,68 × 0,73 m;
- nút `door` nằm dưới khớp `door_hinge` (revolute, trục −Y, 0–110°);
- normal, ORM, LOD1;
- Validator sạch.

**Cần làm thêm:**
1. **Một vật liệu, một atlas.** Hiện lồng có 2 vật liệu, mỗi cái một bộ normal và ORM riêng: 5 ảnh, 1,8 MB,
   4 lần vẽ cho một cái lồng. Gộp mọi phần về **một** vật liệu với **một** atlas 1024 px (basecolor, normal,
   ORM). Hai phần khác màu thì tách bằng vùng UV, không tách bằng vật liệu.
   - Lý do: game sẽ có nhiều lồng, và mỗi vật liệu thêm là một lần đổi trạng thái GPU.
2. **Chi tiết kể chuyện ngay trên lồng**, dựng chung vào mô hình:
   - ổ khoá móc treo ở cửa; nó nằm trong `door`, quay theo cửa;
   - khay đáy lót tờ báo cũ, có vệt bẩn;
   - một sợi dây buộc tạm;
   - vết gỉ chảy dọc từ các mối hàn xuống (dùng mask từ AO và curvature, chảy theo hướng −Y).
3. **Ảnh duyệt cận cảnh** trong game:
   - đứng cách 1,5 m, ngang tầm mắt, ban ngày và ban đêm;
   - thêm 4 góc render Cycles (trước, sau, chéo, cửa mở 110°).

   Ảnh hiện có đứng xa và bị cột sáng vàng phủ lên, nên lồng trông như vàng đồng: đó là ánh của cột sáng, không
   phải màu lồng.

## 2. Đèn pin `people/den_pin.glb`

**Đạt:** gốc ở chỗ cầm, kính quay ra +Z, `lamp_glass` phát sáng, LOD1, Validator sạch.

**Chưa đạt về hình:** trong ảnh Blender, thân là một ống trụ đều có các khấc tròn nổi bằng hình khối. Nó trông
giống tay cầm cao su hơn là đèn pin. Kính là một đĩa phẳng màu be, không có chao phản quang. Làm lại theo cấu
trúc của một đèn pin nhôm thật:

| Phần | Kích thước | Cách làm |
|---|---|---|
| Đầu đèn (bezel) | Ø 38 mm, dài 35 mm, **to hơn thân** rồi thuôn dần vào thân | sand3d: một profile tròn xoay (revolve) có bậc thuôn |
| Chao phản quang | lõm hình nón bên trong đầu, sâu 15 mm | kim loại bóng: metallic 1, roughness 0,08; vật liệu riêng `reflector` |
| Kính | đĩa Ø 32 mm, lùi vào 2 mm so với mép đầu | `lamp_glass`: emissive trắng ấm (1,0; 0,92; 0,78), phát sáng mạnh hơn ở tâm (emissive texture dạng hình tròn mờ dần ra mép) |
| Thân | Ø 26 mm, dài 90 mm | nhôm anod đen: base (0,03; 0,03; 0,035), metallic 1, roughness 0,35. **Vân nhám (knurling) chỉ làm bằng normal map**, ô quả trám 1 mm, không dựng bằng hình khối |
| Nút bấm | cao su, Ø 10 mm, ở cạnh thân, cách đầu 25 mm | vật liệu cao su: roughness 0,8 |
| Đuôi | nắp vặn Ø 27 mm, dài 18 mm, có lỗ xỏ dây | khoen nhỏ |
| Mòn | mép đầu và đuôi tróc lớp anod, lộ nhôm bạc | mask từ curvature: chỗ lồi → base (0,8; 0,8; 0,82), roughness 0,3 |

Tổng dài khoảng 150 mm, giữ nguyên gốc ở giữa thân. Một atlas 512 px là đủ.

## 3. Âm thanh

Đo bằng FFmpeg (phổ tần, mức nhiễu nền, độ to):

| File | Kết quả | Đánh giá |
|---|---|---|
| `ui_bam.ogg`, `ui_meo.ogg` | sạch, −16 LUFS | **Đạt** |
| `cho_sua_01…03.ogg` | Phổ bị **cắt ở khoảng 4 kHz** (giống thu qua máy MP3, đúng như nguồn ghi chú). Có các vạch nhiễu dựng đứng 12–16 kHz (tiếng lách tách). Tiếng sủa chỉ dài 0,25 s, phía sau là 0,45 s im lặng được độn vào. | **Chưa đạt**: nghe sẽ đục như qua điện thoại |
| `cho_rit.ogg` | Cũng cắt ở khoảng 3,5 kHz. **Tiếng xì nền phủ suốt 0–20 kHz** (nền nhiễu −22 dB). Có tiếng tích tắc đều mỗi khoảng 50 ms. | **Chưa đạt** |
| `long_mo.ogg` | Có dải cao (tốt), nhưng nền nhiễu −18,8 dB | **Gần đạt**: cần khử nhiễu |

**Cách làm cho đạt:**

1. **Chọn nguồn sạch.**
   - Bản ghi CC0 thu bằng micro thật, 44,1 hoặc 48 kHz, WAV hoặc FLAC. Không dùng bản đã qua MP3 hay
     điện thoại.
   - Tiêu chí loại: mở phổ (Audacity: *Analyze → Plot Spectrum*, hoặc `ffmpeg -lavfi showspectrumpic`). Tiếng
     sủa phải có năng lượng rõ tới ít nhất 8 kHz. Phổ bị cắt phẳng ở 4 kHz hay 11 kHz là dấu của nén mất dữ liệu.
   - Chó cỏ Việt cỡ vừa (12–18 kg): tiếng sủa có tần số gốc khoảng 400–700 Hz. Tránh tiếng chó nhỏ chói (trên
     1 kHz) hoặc chó to ồm (dưới 250 Hz).
2. **Làm sạch theo thứ tự:**
   1. Cắt lấy đoạn sạch.
   2. Khử lách tách: FFmpeg `adeclick`, hoặc Audacity *Click Removal*.
   3. Khử nhiễu bằng hồ sơ nhiễu lấy từ đoạn im lặng của chính bản ghi: Audacity *Noise Reduction* 6–12 dB,
     hoặc FFmpeg `afftdn` với `nr=12` và hồ sơ đã học (`sn`). Không khử quá tay, kẻo tiếng bị "méo nước".
   4. Lọc cắt thấp 80 Hz.
   5. Fade vào 5 ms, fade ra 20–60 ms.
   6. Chuẩn hoá đỉnh thật (true peak) về −1 dBTP.
3. **Không độn im lặng vào file.** Đo độ to xong thì cắt im lặng ở đuôi.
   - Tiếng ngắn (dưới 1 s): đo theo đỉnh và độ to ngắn hạn (short-term LUFS, khoảng −14 đến −18). Đừng dựa
     vào độ to tích hợp, vì độ to tích hợp không đo đúng được tiếng ngắn.
   - Game tự trộn mức âm lượng.
4. **Nhiều biến thể.**
   - Ít nhất 5 tiếng sủa khác nhau: sủa đơn, sủa đôi, sủa xa, gầm gừ, sủa mừng. Game chọn ngẫu nhiên và
     xê dịch cao độ ±5%, nên không bị lặp.
   - Tiếng rên `cho_rit` phải nối vòng liền (người chơi nghe nó lặp đi lặp lại khi đến gần lồng). Cắt ở chỗ
     sóng qua điểm 0 và crossfade 200 ms giữa đuôi với đầu.
5. **Tự kiểm trước khi nộp.** Mỗi file nộp kèm ảnh phổ (`review/05/audio/<ten>.png`), và ghi vào `SOURCES.md`:
   - nền nhiễu (`astats` → *Noise floor*): phải dưới −50 dB;
   - tần số cao nhất còn năng lượng.

## 4. Phía game sẽ làm

- **Đã sửa:** khung **`?look=streetlife` ra màn hình đen**. Đúng là lỗi camera của phía game: camera đứng cách bộ
  bàn ghế 4,2 m, mà ở chỗ đó phố hẹp nên điểm ấy nằm trong toà nhà bên kia. Giờ camera lùi vào (4,2 → 3,6 → 3,0
  → 2,5 m) tới khi đường từ bàn tới mắt thoáng hẳn, nếu không được thì chọn bộ bàn ghế kế tiếp.
- **Đã sửa:** **tư thế cầm đèn pin.** Trước đây gốc đèn đặt ở cổ tay. Giờ đèn nằm giữa lòng bàn tay
  (`hand_r`: X −0,035, Y 0,075 m) và nghiêng xuống, nên khi đi, đèn chĩa ra trước và hơi chúc xuống, theo hướng
  ánh sáng. Bạn chỉ cần giữ quy ước: gốc ở giữa thân đèn, kính quay ra +Z.
- **Cửa lồng mở theo khớp `door_hinge`**, cùng âm thanh 3D: tiếng rên của chó dẫn người chơi tới lồng, tiếng sủa
  mừng khi lồng mở. Phía game sẽ nối khi có bộ âm thanh đạt.

## 5. Đề xuất nộp

- Có thể gộp ngay vào game: lồng chó (sau khi gộp một vật liệu) và hai tiếng giao diện.
- Làm lại rồi nộp tiếp: đèn pin và các tiếng chó.
- Sau đó tiếp tục phần còn lại của `HOA_SI.md`, Phần 1, mục A: Ô Quan Chưởng, đèn lồng, `lamp_glass` /
  `window_glow`, quầy hàng nông 0,6 m, tên trộm thứ hai, động tác của chó.

---

# Lần 2: bản sửa lồng chó và đèn pin (PR #16, commit `0cad9fe`)

## Kết quả

| Món | Đo được | Kết luận |
|---|---|---|
| Lồng chó `people/long_cho.glb` | 1 vật liệu, 1 atlas 1024 px (LOD1: 512 px); 1.664 / 424 tam giác; khớp `door_hinge` 0–110° quanh −Y; có ổ khoá đi theo cửa, khay báo cũ, vết gỉ | **Đạt, đã đưa vào game** |
| Đèn pin `people/den_pin.glb` | đầu loe, chao phản quang kim loại bóng (roughness 0,08), kính phát sáng, vân nhám bằng normal, nút cao su, nắp đuôi; 1.136 / 400 tam giác, ảnh 512 px | **Đạt, đã đưa vào game** |
| Tiếng chó, tiếng cửa | bạn đã tự đánh giá là chưa đạt | Chưa đưa vào game; vẫn nằm trên PR #16 |

File GLB 3,8 MB là do ảnh PNG; bước đóng gói (`tools/pack_hq.py`) chuyển sang JPEG nên bản web của lồng chỉ còn 148 KB.

## Phía game đã làm

- **Cửa lồng mở thật theo khớp của bạn.** Game đọc `s3d_joint` trong extras của nút, giữ phần dưới nút đó trên một bản lề
  riêng, và khi chó được thả thì cửa quay 0 → 110° trong 0,9 giây. Mọi món sau này có khớp `revolute` cũng chạy được theo
  cách này, không cần sửa code.
- **Lồng không còn đè lên bó vỉa.** Khi chỗ đặt lồng cắt ngang mép đường, game dời lồng lên vỉa hè (tối đa 1,2 m) và dời
  luôn con chó bên trong. Bạn không cần nâng lồng 0,15 m nữa.
- **Góc duyệt riêng cho lồng**, thay cho hai bản vá camera tạm (`qc-*-harness.patch`):
  - `?gl&look=cage`: lồng trên đường ven hồ, nhìn từ 1,5 m ngang tầm mắt;
  - thêm `&door=open`: cửa mở hết 110°;
  - thêm `&time=night`: lồng được rọi bằng đúng ánh đèn pin của Minh, nên ảnh đêm không còn tối đen.

## Còn nên làm ở lồng (không chặn việc đưa vào game)

1. **Gỉ chảy thành vệt, không loang đều.** Hiện gỉ là các đốm cam rải đều khắp thanh, trông như hoa văn. Gỉ thật bắt đầu
   ở mối hàn, chỗ giao nhau và mép dưới (nơi nước đọng), rồi chảy xuống thành vệt dài.
   - Lấy mask từ AO (chỗ khuất, chỗ giao) nhân với curvature lõm.
   - Kéo dài mask theo −Y: làm mờ có hướng (directional blur) xuống dưới 3–6 cm.
   - Phủ thêm một lớp noise nhỏ để mép vệt không đều.
   - Thanh trên cùng gần như sạch; thanh đáy và khay gỉ nhiều nhất.
2. **Sợi dây buộc tạm** ở cửa (mục 1.2 lần trước) vẫn thiếu. Một sợi nilon hoặc dây dù quấn 2–3 vòng quanh thanh cửa và
   khung là đủ, dùng chung atlas.

## Đèn lồng (lô 06)

Thư mục làm việc của bạn nằm trên máy Windows, phía game không mở được. Để QC được, cứ đẩy lên **một PR nháp** kèm ảnh
duyệt, kể cả khi chưa đạt; ghi rõ chỗ nào chưa xong.

Về mật độ UV (texel density) chưa đều:
- **Mục tiêu:** đèn lồng được nhìn ở 2–6 m, nên đặt khoảng 512 px/m cho LOD0. Mọi đảo UV phải gần cùng mật độ đó (lệch
  không quá ±15%).
- **Cách đo:** với mỗi đảo, mật độ = cạnh atlas (px) × √(diện tích UV / diện tích 3D). Trong Blender có thể dùng add-on
  Texel Density Checker, hoặc tự tính bằng `bmesh` trong script sand3d.
- **Cách sửa:**
  1. *UV → Average Islands Scale* để các đảo cùng tỉ lệ.
  2. Đóng gói lại với khoảng đệm 8 px.
  3. Nếu mật độ chung quá thấp: tăng atlas lên 1024 px, hoặc dùng một vải lụa/giấy dó lặp lại (tileable). Khi dùng texture
     lặp, xếp UV cho thẳng theo nan tre, để hoa văn chạy dọc theo nan.
- **Đường nối (seam)** đặt ở chỗ khuất: mặt sau, chỗ nan tre đè lên vải, đáy đèn.

## Âm thanh: lấy nguồn khi không đăng nhập được

- **Cách chắc nhất:** người giao việc đăng nhập Freesound hộ, lọc giấy phép **Creative Commons 0**, tải 5–8 file
  WAV hoặc FLAC (44,1–48 kHz), rồi để vào một thư mục cho bạn. Bạn chỉ làm sạch và QC theo mục 3 ở trên.
- **Wikimedia Commons** cho tải thẳng, không cần đăng nhập, nhưng tiếng chó CC0 rất ít. Phía game tìm được một file:
  `File:Bark (avk).wav` (CC0, 0,9 s). Các file khác phần lớn là CC BY-SA, **không dùng** vì không hợp với giấy phép CC0 của
  tài nguyên.
- Đừng tổng hợp tiếng chó bằng máy: tiếng rên nhân tạo nghe ra ngay. Tiếng giao diện tổng hợp thì vẫn được, như bạn đã làm.

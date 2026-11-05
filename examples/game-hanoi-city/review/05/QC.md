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

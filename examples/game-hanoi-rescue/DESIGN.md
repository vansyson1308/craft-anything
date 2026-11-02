# Giải Cứu Cún: Hà Nội — tài liệu thiết kế

Bản này là khung để anh viết lại nội dung. Mọi chữ trong game (cốt truyện, thoại, tên chó, vật phẩm,
cửa hàng) nằm trong một file: `src/content.rs`. Sửa ở đó là game đổi theo, không phải đụng vào code chơi.

## 1. Một câu

Minh, 20 tuổi, sinh viên năm hai, lẻn vào phố cổ Hoàn Kiếm ban đêm để giải cứu chú cún Bơ và những
con chó bị "Hội Thòng Lọng" bắt trộm: vào, tìm lồng, mở, dắt chó chạy ra điểm thoát trước khi bị tóm
hay trời sáng.

## 2. Học từ đâu, học cái gì (không chép)

| Học từ | Bài học | Áp dụng |
|---|---|---|
| Game extraction (Escape from Duckov và thể loại) | Vòng **vào → nhặt → thoát kịp hoặc mất hết** tạo hồi hộp; mỗi chuyến thành công phải thấy mình mạnh lên; căn cứ là nơi thấy tiến bộ. Lời chê phổ biến: lặp lại, may rủi quyết định quá nhiều | Mỗi chuyến có rủi ro thật (mất balo), căn cứ = phòng trọ có những chú chó đã cứu chạy quanh; **mỗi chương thêm một cơ chế mới** chống nhàm; bảo đảm luôn có đủ đồ tối thiểu để may rủi không phá ván |
| Game thế giới mở theo nhiệm vụ (dòng GTA) | Nhân vật có động cơ rõ; kẻ thù có thứ bậc (lính → đầu sỏ → trùm); mỗi nhiệm vụ diễn ra ở một địa danh nổi tiếng; một bản đồ dùng lại cho nhiều nhiệm vụ; hoạt động phụ giữa các nhiệm vụ chính; giọng hài | Hội Thòng Lọng: Thằng Còi (canh gác) → Tư Lồng (đầu sỏ) → Bảy Thòng Lọng (trùm); 5 chương ở 5 địa danh; "Đi tuần phố" là hoạt động phụ để kiếm tiền nâng cấp |
| Sách giáo khoa thiết kế game | **Dạy → thử → biến tấu** cho mỗi cơ chế; đường cong độ khó **răng cưa** (tăng trong chương, hạ khi sang cơ chế mới, có quãng nghỉ); giữ người chơi trong "vùng flow" (không quá dễ, không quá khó); phản hồi rõ cho mọi hành động | Bảng độ khó ở mục 5; mỗi đòn đánh của địch đều báo trước (vòng đỏ); bị phát hiện có thanh cảnh giác tăng dần, không bị tóm "oan" |

## 3. Vòng chơi

```
Phòng trọ ──► Chuẩn bị (xem nhiệm vụ, điểm thoát) ──► Vào phố
   ▲                                                     │
   │   thoát kịp: giữ tiền, đồ, chó đã cứu               ▼
   └──── nâng cấp ◄── kết quả ◄── tìm lồng, nhặt đồ, né/đánh/lừa bọn trộm ──► chạy ra điểm thoát (đứng 3 giây)
                              bị tóm hoặc hết giờ: mất balo
```

- **Lén** (đẩy cần điều khiển nhẹ, hoặc giữ Shift): đi chậm, không gây tiếng động, khó bị thấy.
- **Đấu trí:** ném dép tổ ong gây tiếng động để dụ bọn trộm đi chỗ khác; lẻn ra sau lưng để trói
  không cần đánh; trốn vào đám đông phố Tạ Hiện; dụ chúng ra đường cho xe máy tông hoặc lên đường tàu
  lúc tàu đến.
- **Giao tranh:** đánh bằng gậy (3 đòn là ngã, đánh lén mạnh gấp rưỡi), ném dép làm choáng, lăn né
  đòn. Kẻ ngã thì trói lại.
- **Cứu chó:** giữ nút "Mở lồng" 1,5 giây (có tiếng động). Chó đi theo, làm mình chậm hơn một chút.
- **Thoát:** đứng trong vùng thoát 3 giây. Mỗi chuyến chỉ mở vài điểm thoát.

## 4. Cốt truyện (5 chương)

| Chương | Địa danh | Mục tiêu | Cơ chế mới |
|---|---|---|---|
| 1. Đêm mất Bơ | Phố Hàng Mã | Cứu 1 chú chó, thoát ở Ô Quan Chưởng | Di chuyển, lén, mở lồng, thoát |
| 2. Tin đồn phố Tạ Hiện | Tạ Hiện | Lấy cuốn sổ của bọn trộm, cứu 1 chó | Ném dép, trốn trong đám đông |
| 3. Chợ Đồng Xuân | Chợ Đồng Xuân | Cứu 2 chó trong một chuyến | Mua "Dây xích đôi" ở cửa hàng; xe máy |
| 4. Phố đường tàu | Đường tàu | Trói Tư Lồng, lấy chìa khóa kho | Đầu sỏ biết lao tới; tàu hỏa chạy qua |
| 5. Cầu Long Biên | Kho bên cầu | Cứu Bơ, thoát qua cầu Long Biên | Trùm Bảy Thòng Lọng, cả hội báo động |

Sau chương 5: phòng trọ thành "Trạm cứu hộ cún", "Đi tuần phố" mở mãi với độ khó tăng dần.

## 5. Đường cong độ khó

| | Ch.1 | Ch.2 | Ch.3 | Ch.4 | Ch.5 | Đi tuần cấp n |
|---|---|---|---|---|---|---|
| Số tên trộm | 2 | 4 | 5 | 6 + Tư Lồng | 7 + Bảy | 3 + n (tối đa 10) |
| Tầm nhìn (ô) | 5 | 6 | 6,5 | 7 | 7,5 | 6 + 0,25n |
| Góc nhìn | 70° | 75° | 80° | 85° | 90° | 80° |
| Thời gian | 7 phút | 6,5 | 6,5 | 6 | 6 | 6 |
| Gọi thêm người giữa trận | không | có | có | có | có | có |

Răng cưa: mỗi chương mở đầu bằng cơ chế mới ở áp lực thấp (ít địch quanh chỗ học), rồi thử lại
gần điểm thoát. "Đi tuần phố" giữa các chương là quãng nghỉ có thưởng.

## 6. Điều khiển (ưu tiên điện thoại)

| | Điện thoại | Máy tính |
|---|---|---|
| Đi / lén | Cần điều khiển nửa trái màn hình (đẩy nhẹ = lén) | WASD hoặc mũi tên, giữ Shift để lén |
| Đánh | Nút tròn lớn | J |
| Ném dép | Nút "Dép" | K |
| Lăn né | Nút "Lăn" | Space |
| Mở lồng / trói | Nút ngữ cảnh, giữ | Giữ E |
| Tạm dừng | Nút góc trên | Esc |

Màn hình dọc và ngang đều chơi được; khung nhìn tự co giãn, nút tránh tai thỏ.

## 7. Phạm vi bản đầu (không phình to)

Có: một quận, 5 chương, đi tuần vô hạn, cửa hàng 5 món, sổ cún, lưu game trên máy.
Chưa có: âm thanh, nhiều quận, chơi nhiều người, nhân vật mặc đồ. Hình vẽ bằng code (không tải ảnh),
nên game rất nhẹ.

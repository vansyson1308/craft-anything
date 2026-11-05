# Hanoi City: Giải Cứu Cún — thiết kế lại cho vui và dễ hiểu

Tài liệu này soi lại bản 3D theo bốn giai đoạn thiết kế game:
- ý tưởng;
- tâm lý người chơi và cơ chế;
- kể chuyện và thế giới;
- cấp độ và cân bằng.

Ba cuốn sách tham khảo: *The Art of Game Design* (Jesse Schell), *Theory of Fun* (Raph Koster), *Level Up!*
(Scott Rogers). Tài liệu thiết kế gốc của lõi game nằm ở `../game-hanoi-rescue/DESIGN.md`.

Ký hiệu: **[đã làm]** có trong game rồi, **[tiếp theo]** là việc kế tiếp, **[cần hình]** chờ agent vẽ.

## Chẩn đoán (vì sao chơi thấy chán)

Lần chơi thử của chủ dự án:
- "buổi đêm tối chả nhìn thấy gì";
- "chơi game cũng chả có hướng dẫn".

Soi theo lăng kính thì ra ba lỗi gốc:
1. **Không biết đi đâu.** Mục tiêu chỉ là một dòng chữ ở góc ("Cứu chó 0/1"). Không có gì trong thế giới chỉ
   đường, nên người chơi lang thang và chán (rơi khỏi "dòng chảy" về phía buồn chán).
2. **Lén lút mà không thấy mắt kẻ địch.** Lõi game có tầm nhìn hình quạt của bọn trộm, và mẹo trong game còn
   nhắc tới nó, nhưng bản 3D không vẽ ra. Người chơi bị phát hiện mà không hiểu vì sao: quá khó một cách vô lý
   (rơi về phía lo âu). Cơ chế đúng nhưng động lực học hỏng, nên cảm xúc hồi hộp không bao giờ đến.
3. **Đêm tối đến mức không đọc được cảnh.** Đêm đẹp trên ảnh, nhưng người chơi cần thấy đường, thấy trộm,
   thấy lồng.
4. **Đi bộ quá xa trước khi có việc.** Chương 1 xuất phát ở bờ hồ, còn lồng chó ở Hàng Mã, cách
   **1.101 m** (đo được bằng la bàn mới).
   - Chương 4 và 5 cũng xa tương tự.
   - Vài phút chạy qua phố vắng là phần chán nhất.
   - **[đã làm]** Mỗi chương giờ xuất phát cách khu nhiệm vụ 150 m, ở phía hồ. Có test bảo đảm mục tiêu
     đầu tiên cách chưa tới 450 m trên bản đồ thật.

## Giai đoạn 1: Hạt giống ý tưởng

**Vòng lặp cốt lõi (một lượt, 5–7 phút):**

> Quan sát (đọc tầm nhìn của trộm) → Lẻn tới (đi lén, vòng sau lưng, dụ bằng dép) → Giải cứu (giữ E mở lồng)
> → Thoát (dắt chó chạy ra cổng, có thể bị đuổi) → Nhận thưởng (tiền, nâng cấp, chương mới) → lượt sau khó
> hơn.

Mỗi bước phải tự vui:
- lẻn qua sát mép tầm nhìn là khoảnh khắc "suýt nữa";
- lúc chó lao ra theo mình là phần thưởng cảm xúc;
- chạy thoát khi bị đuổi là cao trào.

**3C:**
- **Nhân vật:** Minh đi, chạy, lén, lăn, đánh, ném dép; có 43 động tác thật.
  - **[tiếp theo]** phản hồi khi va chạm (rung camera nhẹ khi trúng đòn đã có).
  - **[tiếp theo]** chó chạy theo nhảy nhót khi được cứu.
- **Camera:** sau lưng, ngang tầm mắt, không xuyên tường **[đã làm]**. Lăn chuột để lùi ra xem toàn cảnh.
- **Điều khiển:** WASD + chuột, Shift lén, E tương tác **[đã làm]**.

**X-Factor:** *phố thật Hoàn Kiếm*.
- Mọi con phố, toà nhà, hồ, cầu đúng bản đồ thật, đúng tên phố.
- Câu chuyện rất Hà Nội: cứu chó khỏi bọn trộm chó.
- Chạy ngay trên trình duyệt, không cần cài.

Người chơi Việt nhận ra phố mình từng đi; người nước ngoài thấy một Hà Nội "đi được".

## Giai đoạn 2: Tâm lý và cơ chế

**MDA, tư duy ngược từ cảm xúc:**

| Muốn người chơi thấy | Động lực cần có | Cơ chế |
|---|---|---|
| Hồi hộp khi lẻn qua | Thấy rõ ranh giới an toàn và nguy hiểm, rồi mạo hiểm sát nó | **[đã làm]** Tầm nhìn của từng tên trộm vẽ trên mặt đất: vàng nhạt khi yên, cam khi nghi, đỏ khi săn. Dấu **?** hoặc **!** trên đầu. Shift làm chậm tốc độ bị phát hiện, đám đông che mình (lõi game đã có). |
| Không bao giờ lạc | Luôn biết bước tiếp theo | **[đã làm]** Cột sáng mọc lên ở mục tiêu, nhìn thấy từ xa qua mái nhà: vàng là chó, sổ hay trùm; xanh là lối thoát. Cột sáng còn soi sáng mặt đất quanh nó. Dòng la bàn trên cùng màn hình: "↖ Lồng chó · 85 m". |
| Hiểu luật mà không phải đọc hướng dẫn dài | Học đúng lúc cần | **[đã làm]** Mẹo ngắn hiện **một lần**, đúng khoảnh khắc: lúc vào game, lần đầu có trộm gần, lần đầu đứng cạnh lồng, lần đầu bị phát hiện, khi đã cứu được chó, khi máu thấp. |
| Ấm lòng khi cứu được chó | Có một khoảnh khắc riêng | **[tiếp theo]** Chó lao ra quấn chân, máy quay khựng 0,5 giây, tiếng sủa. **[cần hình]** Chó có hoạt ảnh vẫy đuôi. |
| Tự hào | Thành tích nhìn thấy được | Lõi đã có: tiền thưởng, nâng cấp, chương. **[tiếp theo]** Bảng kết quả chấm sao: không bị phát hiện, thời gian, cứu thêm chó. |

**Bốn kiểu người chơi:**
- **Achievers (thành tích):** sao mỗi chương, chế độ tự do nhiều cấp, nâng cấp ở cửa hàng **[đã có, tiếp theo:
  sao]**.
- **Explorers (khám phá):** cả quận Hoàn Kiếm đi được.
  - **[tiếp theo]** "Góc phố có thật": đứng ở một điểm nổi tiếng (Tháp Rùa, Nhà thờ Lớn, chợ Đồng Xuân) thì hiện
    một dòng giới thiệu ngắn về nơi đó; gom đủ thì có thưởng.
- **Socializers (giao lưu):** **[tiếp theo]** ảnh chụp kết quả để chia sẻ, kèm tên phố đã đi qua.
- **Killers (cạnh tranh):** **[tiếp theo]** chạy nhanh nhất mỗi chương, lưu kỷ lục trên máy.

## Giai đoạn 3: Kể chuyện và thế giới

**Kể chuyện qua môi trường:** không thêm chữ. Đặt đồ vật để người chơi tự đoán.
- **[cần hình]** Quanh mỗi lồng: dây thòng lọng vứt dưới đất, bát nước đổ, vệt bánh xe máy.
- **[cần hình]** Vòng cổ có thẻ tên chó rơi trên đường dẫn tới lồng, như dấu vết dẫn đường.
- **[cần hình]** Tờ "Tìm chó lạc" dán trên cột điện.

**Hành trình anh hùng** (5 chương):

| Bước | Trong game |
|---|---|
| Thế giới bình thường | Minh, sinh viên, sống cùng Bơ (cảnh mở đầu) |
| Tiếng gọi | Bơ bị bắt (cảnh mở đầu) |
| Người dẫn dắt | Cô Hằng bánh mì |
| Bước qua ngưỡng cửa | Chương 1 "Đêm mất Bơ": lần đầu đối mặt bọn trộm |
| Thử thách, đồng minh, kẻ thù | Chương 2–3: Tạ Hiện, chợ Đồng Xuân, cuốn sổ, cứu thêm chó |
| Vào hang sâu nhất | Chương 4: phố đường tàu, trùm Tư Lồng |
| Thử thách lớn nhất | Chương 5: kho dưới chân cầu Long Biên, trùm Bảy Thòng Lọng |
| Trở về với phần thưởng | Cứu được Bơ, qua cầu Long Biên lúc trời sáng |

**[tiếp theo]** Ánh sáng theo hành trình:
- chương 1 đêm khuya;
- chương 2–3 chiều tối;
- chương 4 nửa đêm;
- chương 5 rạng sáng: thoát ra lúc mặt trời mọc trên sông Hồng.

## Giai đoạn 4: Cấp độ và cân bằng

**Dòng chảy (độ khó tăng theo kỹ năng):**

| Chương | Trộm | Tầm nhìn | Thêm gì mới |
|---|---|---|---|
| 1 | 2 | 10 m, 70° | Chỉ học lẻn và mở lồng |
| 2 | 4 | 12 m | Ném dép để dụ, lấy sổ |
| 3 | 5 | 13 m | Hai lồng, xe máy trên đường |
| 4 | 6 | 14 m | Trùm biết lao tới, tàu hoả |
| 5 | 7 | 15 m | Cả hội báo động, trùm cuối |

Bảng này lấy từ số liệu thật trong lõi game. **[tiếp theo]** Nếu thua một chương hai lần liền, lần sau bớt
một tên trộm và thêm 30 giây: giữ người chơi ở trong dòng chảy, không để họ bỏ cuộc.

**Dẫn đường bằng ánh sáng, không bằng mũi tên:**
- **[đã làm]** Cột sáng là "ngọn hải đăng" của màn chơi.
- **[đã làm]** Ban đêm, khoảng một phần ba số cửa hàng hắt đèn ra vỉa hè. Đèn đường soi các trục chính.
  Ánh sáng tự nhiên vẽ ra lối đi.
- **[đã làm]** Đêm đã được nâng sáng để đọc được cảnh:
  - trăng sáng hơn, ánh sáng nền nhiều hơn;
  - một quầng sáng dịu quanh Minh;
  - đèn pin rọi trước mặt.
- **[tiếp theo]** Đèn lồng Hàng Mã treo dọc đường dẫn tới lồng của Chương 1.
  - **[cần hình]** Mô hình đèn lồng thật thay quả cầu.

## Đo lường "đã vui chưa"

Mời 3–5 người chưa chơi bao giờ, chỉ đưa địa chỉ trang, không giải thích gì. Ghi lại:
- bao lâu thì họ hiểu phải đi đâu (mục tiêu: dưới 10 giây);
- bao lâu thì họ lần đầu lẻn qua được một tên trộm;
- có cứu được chó trong lần đầu không;
- có muốn chơi chương 2 không.

Lời nhắn cho agent tester (`TESTER_PROMPT`, đã gửi chủ dự án) dùng đúng các câu hỏi này.

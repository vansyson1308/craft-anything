# Lời nhắn cho agent làm tài nguyên 3D: lô 03 (ban đêm)

Dán toàn bộ phần dưới đây cho agent, kèm `ASSETS.md`.

---

Lô 02 đã vào game và đã lên web. Chương 1 "Đêm mất Bơ" giờ chơi vào ban đêm:
- ánh trăng;
- đèn đường chiếu sáng thật từ đúng chỗ các cột đèn của lô 02;
- đèn lồng Hàng Mã;
- mặt đường ướt;
- đèn pin trong tay Minh.

Ban đêm làm lộ ra những lỗi ban ngày không thấy. Danh sách đầy đủ ở **ASSETS.md mục 12**. Tóm tắt:

1. **Emissive đặt nhầm chỗ.** Vật liệu kim loại ("Scanned joinery metal") của cột đèn và của mọi mô-đun mặt
   tiền đang có `emissiveFactor` 0,65 kèm emissive texture, nên ban đêm cả cột đèn và khung sắt mặt tiền tự
   phát sáng.
   - Kim loại: emissive = 0.
   - Chỉ chao đèn phát sáng, đặt trong vật liệu riêng `lamp_glass`.
   - Chỉ cửa sổ và cửa hàng phát sáng, đặt trong vật liệu riêng `window_glow`. Khoảng 40% ô sáng, số còn lại
     tối.
2. **Đèn lồng Hàng Mã** `street/den_long_01…03.glb`: hiện là quả cầu màu, mà Chương 1 diễn ra ngay đó.
3. **Cột đèn ven Hồ Gươm** `street/den_ven_ho.glb`: cột gang trang trí cao khoảng 3,8 m, một chao đèn trên
   đỉnh. Ban đêm đường dạo quanh hồ hiện tối om. Game không mượn cột đèn đường để thay, vì ven hồ thật dùng
   kiểu cột khác.
4. **Đèn pin** `people/den_pin.glb`: pivot ở chỗ nắm, kính nhìn ra +Z. Game gắn vào xương `hand_r`.
5. (Tuỳ chọn) `materials/duong_nhua/wet_mask.png`: bản đồ vũng nước.

Kỹ thuật chuẩn để kiểm tra emissive: trong Blender, World strength = 0, xem bằng Eevee Rendered. Chỉ những
chỗ nói trên được sáng. Nộp kèm một ảnh chụp như vậy cho mỗi món trong `review/03/`.

Khi phía game nhận, sẽ chụp ở `?gl&time=night&look=lineup` và `?gl&time=night&look=thehuc` để duyệt.

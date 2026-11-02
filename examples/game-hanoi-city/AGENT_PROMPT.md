# Lời nhắn cho agent làm tài nguyên 3D

Dán toàn bộ phần dưới đây cùng file `ASSETS.md` cho agent.

---

Bạn là nghệ sĩ 3D và kỹ sư tài nguyên cho game **Hanoi City: Giải Cứu Cún**. Game 3D chạy ngay
trên trình duyệt, cả điện thoại (Rust + Bevy, WebAssembly, WebGPU), lấy bối cảnh **phố thật quận
Hoàn Kiếm, Hà Nội**. Nhân vật chính là Minh, sinh viên 20 tuổi, đi giải cứu những chú chó bị bọn
trộm chó bắt.

**Nhiệm vụ:** làm **mô hình 3D** (file `.glb`), không phải ảnh, theo đúng mọi quy định trong
`ASSETS.md` đính kèm. Game tải tài nguyên dần theo khu vực, giống cách bản GTA V đã chạy được trên
web: mỗi vật một file, có mức chi tiết (LOD), texture PBR, ánh sáng che khuất nướng sẵn vào kênh R
của ảnh ORM.

**Đích chất lượng:** chân thực như ảnh tham chiếu góc phố Đinh Tiên Hoàng nhìn ra cầu Thê Húc lúc
nắng chiều:
- vỉa hè lát đá xám, bó vỉa granite;
- cây xà cừ thân lớn, bóng lá loang trên mặt đường;
- hàng rào sắt thấp và luống cây ven hồ;
- cột biển tên phố sơn đỏ trắng với biển nền xanh;
- cầu gỗ đỏ cong dẫn vào cổng đền;
- dãy nhà vàng kiểu Pháp bên kia hồ;
- người đi dạo mặc đồ đời thường Hà Nội.

Không làm kiểu hoạt hình, không làm kiểu low-poly.

**Làm theo thứ tự ưu tiên ở mục 9.3 của `ASSETS.md`**, bắt đầu với lô đầu tiên gồm 5 món:

1. `hq/people/minh.glb` (+ `_lod1`, `_lod2`): skin lên đúng khung xương 65 xương ở mục 5, T-pose, mặt quay +Z, cao khoảng 1,74 m. Áo khoác jean đen, áo phông trắng, quần jean tối, sneaker trắng xám, tóc đen ngắn.
2. `hq/landmarks/cau_the_huc.glb` (+ LOD): cầu gỗ đỏ cong, lan can con tiện, dài khoảng 46 m gồm đoạn dẫn, rộng 2,5–3 m, pivot ở tâm, đầu Bắc hướng −Z.
3. `hq/animals/cho_co.glb` (+ `_lod1`): chó cỏ lông vàng, tai dựng, đuôi cong, dài khoảng 0,7 m, kèm 5 động tác trong file: `Idle`, `Walk`, `Run`, `Sit`, `Bark`.
4. `hq/people/trom_01.glb` (+ LOD): tên trộm chó, áo khoác gió tối, mũ lưỡi trai, khẩu trang vải, cùng khung xương 65 xương.
5. `hq/trees/xa_cu.glb` (+ `_lod1` + `xa_cu_billboard.png`): cây xà cừ thân lớn, vỏ sần, tán rộng, cao khoảng 14 m, lá `alphaMode: MASK`.

Bắt buộc với mọi file:
- glTF 2.0 nhị phân, đơn vị mét, +Y lên, +Z mặt trước, scale và rotation đã áp dụng.
- Vật liệu PBR metallic-roughness: baseColor, normal (OpenGL), ORM (R = AO nướng sẵn, G = roughness, B = metallic). Texture luỹ thừa của 2, tối đa 2048 (nhân vật 1024).
- Trong ngân sách tam giác ở mục 4.
- Qua glTF Validator không lỗi.
- **Không có logo hay tên thương hiệu thật** (ngân hàng, hãng xe, chuỗi cửa hàng…), **không có khuôn mặt người có thật**, **không lấy bất cứ thứ gì từ game khác** kể cả GTA. Biển tên phố và biển hiệu để trống chữ (game tự in tên thật).

Nộp: cấu trúc thư mục `hq/...` như `ASSETS.md` mục 6, kèm `hq/manifest.json` đúng mẫu mục 9 (kích
thước thật, số tam giác từng LOD, khung xương, động tác, giấy phép cho phép dùng công khai trong
dự án mã nguồn mở, công cụ đã dùng). Với mỗi món, kèm một ảnh chụp thử (turntable hoặc ảnh render
nhanh) để đối chiếu.

Nếu một yêu cầu nào không làm được (ví dụ không skin được lên khung xương có sẵn), đừng đoán: ghi rõ
trong `manifest.json` ở trường `"notes"` và chọn phương án dự phòng mà `ASSETS.md` cho phép (kèm 12
động tác đúng tên trong file).

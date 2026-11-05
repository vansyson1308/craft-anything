# Lời nhắn cho agent làm tài nguyên 3D: lô 03 (vẽ nốt)

Dán toàn bộ phần dưới đây cho agent, kèm `ASSETS.md` và `AGENT_PROMPT_02.md` (mục A của file đó là kỹ thuật
chuẩn, vẫn áp dụng).

---

Lô 02 đã vào game và đã lên web: https://vansyson1308.github.io/craft-anything/

Chương 1 "Đêm mất Bơ" giờ chơi vào ban đêm. Game tự lo:
- ánh trăng;
- đèn đường chiếu sáng thật từ đúng chỗ các cột đèn của lô 02;
- mặt đường ướt phản chiếu đèn;
- đèn pin trong tay Minh.

Lô này **vẽ nốt mọi món còn thiếu**. Làm theo thứ tự ưu tiên dưới đây, vì phần đầu lộ ra ngay ở Chương 1.

Mỗi món có ghi chú:
- **[tự vào game]**: game đã có sẵn chỗ, nộp đúng tên file là món đó tự hiện ra, không cần sửa code;
- **[game nối sau]**: phía game sẽ nối khi nhận được file.

## 1. Ban đêm (ưu tiên cao nhất)

Ban đêm làm lộ ra những lỗi ban ngày không thấy. Chi tiết ở **ASSETS.md mục 12**.

1. **Sửa emissive đặt nhầm chỗ [tự vào game].**
   - Hiện tượng: vật liệu kim loại "Scanned joinery metal" của cột đèn và của cả 8 mô-đun mặt tiền đang có
     `emissiveFactor` 0,65 kèm emissive texture. Ban đêm cả cột đèn và khung sắt mặt tiền tự phát sáng.
   - Kim loại: emissive = 0.
   - Chỉ chao đèn phát sáng, trong vật liệu riêng tên `lamp_glass` (vàng ấm, khoảng 2.700 K).
   - Mặt tiền: chỉ cửa sổ và cửa hàng phát sáng, trong vật liệu riêng tên `window_glow`. Khoảng 40% ô sáng,
     rèm che một phần, số còn lại tối. Cửa cuốn đóng thì tối.
   - File cần nộp lại: `street/den_duong_01.glb`, `buildings/mat_tien_tret_01…04.glb`,
     `buildings/mat_tien_tang_01…04.glb` (cả `_lod1`).
2. **Cột đèn ven Hồ Gươm** `street/den_ven_ho.glb` (+ `_lod1`) **[tự vào game]**.
   - Cột gang trang trí cao khoảng 3,8 m, một chao đèn trên đỉnh (vật liệu `lamp_glass`, tâm sáng ở độ cao
     khoảng 3,5 m).
   - Pivot giữa đế, đối xứng.
   - Game đặt mỗi 25 m dọc đường dạo quanh hồ và chỉ bật đèn khi đã có file này. Hiện đường dạo ban đêm tối
     om.
3. **Đèn pin** `people/den_pin.glb` **[tự vào game]**.
   - Dài 0,15 m, dưới 1.500 tam giác, kính có emissive.
   - Pivot ở chỗ nắm, kính nhìn ra +Z.
   - Game gắn vào xương `hand_r` của Minh.
4. **Đèn lồng Hàng Mã** `street/den_long_01…03.glb` **[game nối sau]**.
   - Hiện là quả cầu màu, mà Chương 1 diễn ra ngay đó.
   - Đèn lồng vải lụa đỏ và vàng, khung tre, tua rua, emissive bên trong.
   - Pivot ở móc treo trên đỉnh.
5. (Tuỳ chọn) **Bản đồ vũng nước** `materials/duong_nhua/wet_mask.png` **[game nối sau]**: ảnh xám 2048,
   lặp được, trắng ở chỗ đọng nước.

## 2. Nhân vật còn thiếu, và nâng chất lượng lô 02

Khung xương và động tác theo ASSETS.md mục 5. Không dùng mặt người thật hay logo hãng.

6. **[tự vào game]** `people/trom_02.glb` (tên trộm to con), `people/tu_long.glb` (trùm phụ Chương 4),
   `people/bay_thong_long.glb` (trùm Chương 5), `people/co_hang.glb` (cô bán bánh mì).
7. **Làm lại cho đạt**, theo chính ghi chú tự đánh giá của lô 02 và mục A của AGENT_PROMPT_02:
   - `minh`, `trom_01`, `nguoi_di_duong_01…03`: UV đầy tối thiểu 70%, vải có sợi thật từ ảnh quét CC0,
     mặt có chi tiết;
   - `cho_co`: smooth shading, lông có hướng, không gãy góc như cáo low-poly.
8. **[game nối sau]** `people/nguoi_di_duong_04…06.glb`, `animals/cho_phu_quoc.glb`,
   `animals/cho_bac_ha.glb`, `people/thong_long.glb`.

## 3. Công trình biểu tượng

Đặt theo mặt bằng ở ASSETS.md mục 6, nhóm B. Chiều cao làm theo thực tế. Có LOD0, LOD1 và LOD2.

9. **[tự vào game]**:
   - `landmarks/o_quan_chuong.glb`: lối thoát của Chương 1, nên làm trước;
   - `landmarks/cho_dong_xuan.glb`: Chương 3;
   - `landmarks/nha_tho_lon.glb`;
   - `landmarks/nha_hat_lon.glb`.
10. **[game nối sau]**:
    - `landmarks/ga_long_bien.glb`;
    - `landmarks/cau_long_bien_nhip.glb`: một nhịp, lặp được;
    - `street/toa_tau.glb`, `street/dau_may.glb`: không logo thật;
    - `landmarks/ho_guom_ke.glb`.

## 4. Nhà phố

11. **[tự vào game]** thêm mô-đun cho phố đỡ lặp:
    - `buildings/mat_tien_tret_05…08.glb`: tiệm vàng, hàng mã, quán phở, quán cà phê ghế nhựa;
    - `buildings/mat_tien_tang_05…10.glb`;
    - nhà kiểu Pháp quanh hồ: `buildings/mat_tien_phap_tret_01…03.glb`, `buildings/mat_tien_phap_tang_01…03.glb`.

    Áp dụng ngay quy tắc `window_glow` ở mục 1.
12. **[game nối sau]** `buildings/mai_bon_nuoc.glb`, `buildings/mai_tum.glb`, `buildings/mai_ngoi_01.glb`.

## 5. Cây, vật liệu nền, đồ phố

13. **[tự vào game]** `trees/sau.glb`, `trees/bang.glb`, `trees/lieu_ru.glb`. Mỗi cây có LOD1 và
    billboard, như xà cừ của lô 02.
14. **[tự vào game]** `materials/be_tong_mai/`, `materials/ngoi_do/`. Mỗi bộ gồm basecolor, normal, orm,
    2048 px, lặp liền, kèm `info.json`.

    **[game nối sau]** `materials/bo_via_granite/`, `duong_nhua_vach/`, `dat_goc_cay/`, `tuong_vua_vang/`,
    `tuong_vua_kem/`.
15. **[tự vào game]** `street/xe_may_02_chay.glb`, `street/xe_may_03_chay.glb`, `street/xe_may_04_chay.glb`.
    Có người lái đội mũ bảo hiểm, không logo hãng.
16. **[game nối sau]**:
    - bản đậu `street/xe_may_01…04_dau.glb`;
    - `street/ghe_nhua_xanh.glb`, `street/ghe_nhua_do.glb`, `street/ban_nhua.glb`;
    - `street/xe_dap_tho.glb`, `street/ganh_hang_rong.glb`, `street/xe_banh_mi.glb`;
    - `street/thung_rac.glb`, `street/chau_cay.glb`, `street/cot_dien_day.glb`.
17. **[game nối sau]** ảnh 2D: `2d/man_hinh_cho_ngang.jpg`, `2d/icon.png`.

## Kỹ thuật chuẩn và cách kiểm tra

- Kỹ thuật làm hình: mục A của AGENT_PROMPT_02.md.
  - Vật liệu từ ảnh quét CC0 (ambientCG, Poly Haven).
  - Sculpt rồi bake normal, AO và curvature xuống bản low.
  - UV đầy ít nhất 70%.
  - Vật lớn dùng vật liệu lặp theo mét trên UV0 và AO riêng trên UV1.
  - LOD1 dùng ảnh nửa cỡ, LOD2 dùng ảnh một phần tư.
- Kiểm tra emissive: trong Blender, đặt World strength = 0 rồi xem bằng Eevee Rendered. Chỉ chao đèn, cửa sổ
  và đèn lồng được sáng.
- Kiểm tra trong game thật: build theo mục A.7 của AGENT_PROMPT_02 rồi mở các địa chỉ:
  - ban ngày: `?gl&look=lineup`, `?gl&look=thehuc`, `?gl&look=tree`;
  - ban đêm: `?gl&time=night&look=lineup`, `?gl&time=night&look=thehuc`.

  Chụp ảnh và nộp kèm trong `review/03/`, mỗi món một ảnh ngày và một ảnh đêm.
- Cập nhật `manifest.json` và `SOURCES.md` (nguồn và giấy phép từng ảnh quét). Chạy glTF validator cho mọi
  file.
- Không logo hãng thật, không mặt người thật, không lấy hình từ game khác.
- Nộp theo từng đợt nhỏ, mỗi đợt một PR, theo thứ tự ưu tiên ở trên. Đợt đầu nên gồm mục 1 và
  `o_quan_chuong`.

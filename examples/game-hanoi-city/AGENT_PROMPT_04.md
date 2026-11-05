# Lời nhắn cho agent làm tài nguyên 3D: lô 04 (phố đông đúc như ngoài đời)

Dán toàn bộ phần dưới đây cho agent, kèm `ASSETS.md` và `AGENT_PROMPT_02.md` (mục A: kỹ thuật chuẩn).
Lô này làm song song với phần còn lại của lô 03 (`AGENT_PROMPT_03.md`). Phần ban đêm và Ô Quan Chưởng
của lô 03 vẫn làm trước.

---

## Vì sao có lô này

Đặt ảnh chụp game cạnh ảnh phố Hà Nội thật thì thấy rõ: điều làm phố trông thật trước hết là **mật độ đồ
vật trên vỉa hè và mặt tiền**, chứ không phải độ chi tiết của từng toà nhà. Hiện phố trong game còn trống.
Những thứ còn thiếu:
- xe máy dựng kín vỉa hè, đầu quay vào nhà;
- ghế nhựa xanh đỏ quanh bàn thấp trước quán, xe đẩy bán hàng;
- chậu cây, cờ đỏ sao vàng cắm trên mặt tiền;
- cục nóng điều hoà, bồn nước inox trên mái;
- bó dây điện võng ngang ngõ;
- vỉa hè gạch đỏ mòn vỡ, lá rụng trên đường.

Game đã có sẵn chỗ cho các món này. Món **[tự vào game]** hiện ra ngay khi nộp đúng tên file:
- xe máy, bàn ghế, chậu cây và thùng rác được rải theo từng ô cửa hàng dọc mọi phố, theo bản đồ thật;
- một nửa số cửa hàng có xe máy dựng trước cửa, một phần năm có bàn ghế nhựa.

Mỗi món dùng hàng trăm lần trong một khu phố, nên phải **nhẹ và đẹp ở khoảng cách 3–10 m**: camera trong
game ở ngay sau lưng nhân vật, ngang tầm mắt người.

## Danh sách (theo thứ tự ảnh hưởng)

| # | File | Ghi chú | |
|---|---|---|---|
| 1 | `street/xe_may_01_dau.glb` … `street/xe_may_04_dau.glb` | Xe số, xe ga, xe cub cũ, xe điện; không logo hãng; màu đa dạng (trắng, đỏ, đen, xanh); có gương, biển số trống. Pivot giữa hai bánh, đầu xe hướng +Z. LOD0 dưới 8.000 tam giác, LOD1 dưới 2.000. | [tự vào game] |
| 2 | `street/ghe_nhua_xanh.glb`, `street/ghe_nhua_do.glb`, `street/ban_nhua.glb` | Ghế đẩu nhựa có lỗ hoa văn ở mặt, hơi bạc màu; bàn nhựa thấp. Dưới 800 tam giác mỗi món. | [tự vào game] |
| 3 | `materials/via_he_gach_do/` | Gạch đỏ 30 × 30 cm lát vỉa hè phố cổ: viên vỡ, viên lệch màu, vết dầu xe máy, rêu ở mép. Theo mục 3 của ASSETS.md. | [game nối sau] |
| 4 | `street/chau_cay.glb`, `street/thung_rac.glb` | Chậu sứ hoặc nhựa, cây thật (lá dùng alpha mask); thùng rác nhựa xanh có nắp. | [tự vào game] |
| 5 | `street/day_dien_ngang.glb` | Bó dây điện võng ngang ngõ, lặp được. | [game nối sau] |
| 6 | `street/co_do.glb` | Cờ đỏ sao vàng trên cán nghiêng. | [game nối sau] |
| 7 | `street/dieu_hoa_cuc_nong.glb`, `buildings/mai_bon_nuoc.glb` | Cục nóng điều hoà treo tường; bồn nước inox trên mái. | [game nối sau] |
| 8 | `street/xe_banh_mi.glb`, `street/ganh_hang_rong.glb`, `street/xe_dap_tho.glb` | Xe đẩy bánh mì có tủ kính, gánh hàng rong, xe đạp chở hàng. | [game nối sau] |
| 9 | Mặt tiền lô 03 (`mat_tien_tret_05…08`, `mat_tien_tang_05…10`) | Cho thêm chi tiết đời thường lên chính mô-đun: cửa cuốn có biển hiệu (mặt chữ trống), mái hiên bạt, cục nóng điều hoà, dây phơi quần áo, cờ, chậu cây ban công. Tường nhiều màu: vàng, hồng, xanh ngọc, kem, có vết ố mưa. | [tự vào game] |

## Kỹ thuật cho đồ lặp nhiều lần

- **Dựng đồ cứng bằng code:** với vật có hình khối rõ ràng, nên dựng bằng
  [sand3d](https://github.com/Sandflow-ai/sand3d) (Apache-2.0, chạy trên Blender `bpy`, xuất GLB). Những vật đó
  là ghế đẩu, bàn nhựa, xe đẩy, cục nóng điều hoà, bồn nước, cột đèn, lồng chó, khung đèn lồng.
  - Mỗi món là một chương trình ngắn, các phần có tên.
  - Cùng chương trình cho ra cùng một mô hình, nên dễ sửa và dễ làm biến thể (đổi kích thước, số chân, màu).
  - Sau đó vẫn UV, gán vật liệu quét CC0 và bake AO như mục A của AGENT_PROMPT_02.
  - Ghi `sand3d` cùng phiên bản trong `made_with` và `SOURCES.md`.
  - Không dùng cho người, chó, cây: những thứ đó vẫn sculpt và retopo.

- **Texture dùng chung:** gom các món cùng chất liệu (nhựa, sơn xe, kim loại) vào một atlas 1024 px cho
  cả nhóm. Như vậy game vẽ hàng trăm chiếc xe với rất ít lần đổi vật liệu.
- **Biến thể màu không cần hình mới:** cùng một khung xe, đổi màu thân bằng atlas, nộp thành
  `xe_may_0N_dau.glb` riêng.
- **Mặt người nhìn:** ghế, bàn, xe nhìn từ ngang tầm mắt 1,6 m ở 3–5 m. Mép bo tròn (bevel nhỏ rồi bake
  normal), nhựa có độ bóng khác nhau giữa chỗ mới và chỗ mòn, xe có bụi bám ở dưới.
- **Kiểm tra trong game:** build như mục A.7 của AGENT_PROMPT_02, rồi mở `?gl&look=streetlife` (bộ bàn
  ghế gần nhất, ngang tầm mắt; thêm `&time=night` để xem ban đêm), hoặc chơi Chương 1. Đi dọc vài con phố, chụp lại và nộp kèm trong `review/04/`.
- Không logo hãng xe thật, không chữ của thương hiệu có thật, không lấy hình từ game khác.
- Mỗi đợt nộp nhỏ một PR. Đợt đầu nên là mục 1 và 2: chỉ riêng chúng đã đổi hẳn cảm giác của phố.

# Hanoi City: Giải Cứu Cún (3D)

Bản 3D của game giải cứu cún, trên **phố thật** quận Hoàn Kiếm: 15.000 tòa nhà dựng đúng chiều
cao tầng theo dữ liệu OpenStreetMap, đúng tên phố, đúng Hồ Gươm, đền Ngọc Sơn, cầu Thê Húc, chợ
Đồng Xuân, Ô Quan Chưởng, cầu Long Biên, phố đường tàu; **người thật** là nhân vật 3D tỉ lệ người
thật, có hoạt ảnh đi, chạy, lén, đấm, lăn, ngã. Biển hiệu là tên quán xá có thật trong phố cổ.
Viết bằng Rust (Bevy), chạy ngay trên trình duyệt (WebGL2), không server, không API.

*The dog-rescue extraction game in 3D on Hoàn Kiếm's real streets (OpenStreetMap), with animated
CC0 characters; Rust + Bevy, in the browser.*

## Cách làm (dùng gì của repo này)

| Bước | Dùng gì |
|---|---|
| Nghiên cứu "bản gốc" (thành phố) | Bản đồ OpenStreetMap, đổi sang dữ liệu game bằng `osm_import` (Rust, trong `../game-hanoi-rescue/examples/`) |
| Luật chơi | Lõi `hanoi-rescue` đã có test (bọn trộm nhìn, nghe, đuổi; lồng chó; xe máy; tàu; thoát), chạy trên bản đồ dựng từ dữ liệu thật (`Map::from_city`) |
| Kiểm tra | 31 bài test của lõi, trong đó có bài tự chơi thắng chương 1 trên phố thật; chạy thử trong Chromium |

## Chạy trên máy

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
./build.sh                                   # ra dist/ (SKIP_OPT=1 để bỏ bước wasm-opt, build nhanh hơn)
python3 -m http.server 8000 --directory dist  # mở http://localhost:8000
```

`#peek` ở cuối địa chỉ (`http://localhost:8000/#peek`) cho Minh bắt đầu ngay cạnh lồng chó, để xem
nhanh bọn trộm và chó.

## Điều khiển

| | Điện thoại | Máy tính |
|---|---|---|
| Đi (đẩy nhẹ là lén) | Cần điều khiển nửa trái | WASD, Shift để lén |
| Xoay camera | Kéo ở nửa phải | Kéo chuột |
| Đánh / ném dép / lăn | Nút tròn bên phải | J / K / Space |
| Mở lồng / trói | Nút cam khi đứng gần | Giữ E |

## Cấu trúc

| File | Làm gì |
|---|---|
| `src/world.rs` | Dựng phố 3D từ dữ liệu: nhà (hai dải tường: mặt tiền cửa hàng, các tầng có cửa sổ sáng đèn), mái bằng có bồn nước inox, đường nhựa, vỉa hè, hồ, công viên, cây, đường ray, cầu, đèn lồng Hàng Mã, biển hiệu |
| `src/actors.rs` | Nhân vật, chó, hoạt ảnh (ghép thư viện động tác vào xương nhân vật và tóc), lồng, thùng đồ, xe máy, tàu |
| `src/main.rs` | Vòng chơi, camera góc thứ ba, cảm ứng/chuột/phím, HUD, các màn hình, lưu game |
| `tools/` | Lấy bản đồ (`fetch_osm.sh`), dựng tài nguyên (`make_assets.py`), nguồn (`SOURCES.md`) |

Nguồn và giấy phép của mọi tài nguyên: [CREDITS.md](CREDITS.md).

## Giới hạn hiện tại

- Lần đầu tải khoảng 30–40 MB (wasm và tài nguyên; trình duyệt lưu lại cho lần sau).
- Chưa đo trên điện thoại thật; cần trình duyệt có WebGL2. Máy yếu có thể giật khi nhìn xa;
  phố ngoài 450 m được ẩn bớt.
- Chưa có âm thanh, chưa có người đi đường; người lái xe máy còn là khối đơn giản.
- "Chó" là mô hình cáo nhuộm màu chó cỏ (chưa có mô hình chó CC0 có hoạt ảnh).

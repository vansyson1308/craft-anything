# Giải Cứu Cún: Hà Nội

Một game "extraction" chạy ngay trên trình duyệt, ưu tiên điện thoại: lẻn vào quận Hoàn Kiếm ban đêm,
tìm những chú chó bị Hội Thòng Lọng nhốt trong lồng, mở lồng, dắt chúng chạy ra điểm thoát trước khi
bị tóm hay trời sáng. Viết hoàn toàn bằng Rust, biên dịch sang WebAssembly, vẽ bằng Canvas 2D; không
server, không API bên ngoài, không tải ảnh (mọi hình vẽ bằng code), cả game khoảng 400 KB.

*A mobile-first browser extraction game set in Hà Nội's Hoàn Kiếm district, written in Rust
(WebAssembly + Canvas 2D). No server, no external services, about 400 KB.*

- Thiết kế: [DESIGN.md](DESIGN.md) (vòng chơi, cốt truyện 5 chương, đường cong độ khó, điều khiển).
- Nội dung (cốt truyện, thoại, tên chó, cửa hàng, bảng độ khó): một file, [`src/content.rs`](src/content.rs).

## Chơi thử trên máy

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
./build.sh                                   # ra thư mục dist/
python3 -m http.server 8000 --directory dist  # mở http://localhost:8000
```

Trên điện thoại cùng mạng Wi-Fi: chạy server với `--bind 0.0.0.0` rồi mở địa chỉ IP của máy tính.

## Đưa lên GitHub Pages

Workflow `.github/workflows/game-pages.yml` build và đăng game mỗi khi thư mục này thay đổi trên
`main`. Chỉ cần bật một lần: **Settings → Pages → Build and deployment → Source: GitHub Actions**.
Game sẽ ở `https://<tên-tài-khoản>.github.io/<tên-repo>/`.

## Cấu trúc

| File | Làm gì |
|---|---|
| `src/map.rs` | Quận Hoàn Kiếm dựng từ danh sách đặc điểm: bàn cờ phố cổ và ngõ, Hồ Gươm, Tháp Rùa, Đền Ngọc Sơn, cầu Thê Húc, Nhà thờ Lớn, chợ Đồng Xuân, Nhà hát Lớn, Ô Quan Chưởng, phố đường tàu, sông Hồng, cầu Long Biên; tìm đường A*, tầm nhìn |
| `src/raid.rs` | Một chuyến: nhân vật, bọn trộm (tuần tra, nghe tiếng động, nhìn, đuổi, đánh, trùm biết lao tới), lồng chó, xe máy, tàu hỏa, dép ném, điểm thoát |
| `src/meta.rs` | Tiền, nâng cấp, sổ cún, tiến trình truyện; lưu trong trình duyệt (JSON có phiên bản) |
| `src/game.rs` | Các màn hình và nút cảm ứng (cần điều khiển, nút đánh/ném/lăn/ngữ cảnh), bố cục co giãn theo màn hình và tai thỏ |
| `src/render.rs` | Vẽ bản đồ và mọi thứ chuyển động |
| `src/web.rs` | Phần trình duyệt: canvas, sự kiện chạm/chuột/phím, lưu game, vòng lặp khung hình |
| `src/content.rs` | Toàn bộ chữ và số để chỉnh |

Mọi thứ trừ `web.rs` là Rust thuần, nên cả game chạy và được kiểm thử không cần trình duyệt:

```sh
cargo test                                         # 28 bài kiểm tra, có cả "tự chơi" chương 1
cargo run --example ascii all                      # in bản đồ ra chữ
cargo run --example snapshot -- 3 2 1 > frame.svg  # vẽ một khung hình ra SVG
```

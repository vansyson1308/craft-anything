# Hanoi City: Giải Cứu Cún (3D)

Bản 3D của game giải cứu cún, trên **phố thật** quận Hoàn Kiếm: 15.000 toà nhà dựng đúng chiều cao
tầng theo dữ liệu OpenStreetMap, đúng tên phố, đúng Hồ Gươm, đền Ngọc Sơn, cầu Thê Húc, chợ Đồng
Xuân, Ô Quan Chưởng, cầu Long Biên, phố đường tàu. Nhân vật 3D tỉ lệ người thật, có hoạt ảnh đi,
chạy, lén, đấm, lăn, ngã. Biển hiệu là tên quán xá có thật trong phố cổ. Viết bằng Rust (Bevy), chạy
ngay trên trình duyệt **máy tính**: WebGPU, hoặc WebGL2 khi trình duyệt chưa có WebGPU. Không server,
không API.

*The dog-rescue extraction game in 3D on Hoàn Kiếm's real streets (OpenStreetMap); Rust + Bevy, in
the desktop browser (WebGPU, WebGL2 fallback). Its real art is delivered separately into
`assets/hq/` to the spec in [ASSETS.md](ASSETS.md).*

Thiết kế lối chơi (vòng lặp, cảm xúc, dẫn đường, độ khó): [DESIGN.md](DESIGN.md).

## Hình và kỹ thuật: hai phần tách bạch

Game làm như cách bản GTA V từng được đưa lên web: người làm hình làm hình, game lo phần đưa hình
lên web.

- **Phần hình:** do người hoặc agent chuyên làm tài nguyên 3D làm theo [ASSETS.md](ASSETS.md), rồi
  thả vào `assets/hq/` (lời nhắn mẫu cho agent: [AGENT_PROMPT.md](AGENT_PROMPT.md)). Mục 11 của
  ASSETS.md liệt kê những thứ hiện còn là hàng tạm.
- **Phần kỹ thuật** (`src/hq.rs`, `src/facade.rs`, `src/mips.rs`, `src/render.rs`, `tools/pack_hq.py`):
  - khi build, đóng gói `assets/hq` cho web:
    - tách ảnh khỏi GLB, gộp ảnh trùng giữa các LOD và các món;
    - chuyển sang JPEG khi không có trong suốt, kể cả ảnh vật liệu nền (đổi tên ghi trong `renamed` của manifest);
    - lô 02: 306 MB còn 100 MB; lúc vào game tải khoảng 53 MB tài nguyên, phần còn lại tải theo khoảng cách;
  - đọc `hq/manifest.json`, món nào được nộp thì thay hàng tạm, món nào chưa có thì giữ hàng tạm;
  - đặt công trình đúng chỗ thật lấy từ bản đồ (tâm mặt bằng, cầu Thê Húc theo hướng cầu);
  - mức chi tiết theo khoảng cách (LOD0 → LOD1 → LOD2 → ảnh billboard cho cây);
  - mặt tiền nhà ghép từ mô-đun, chỉ dựng cho các khu phố gần camera, xa thì dùng tường phẳng;
  - đồ phố đặt theo dữ liệu bản đồ, cũng chỉ dựng cho các khu gần camera:
    - bó vỉa dọc mọi phố rộng từ 5 m, chừa chỗ ngã rẽ;
    - đèn đường mỗi 30 m trên các trục chính;
    - lan can quanh Hồ Gươm, chừa đầu cầu Thê Húc;
    - biển tên ở hai đầu mỗi phố, chữ in tên phố thật (`tools/make_street_names.py` vẽ atlas tên phố);
  - người đi dạo quanh hồ (`src/crowd.rs`), mặt hồ gợn sóng, đường dạo lát gạch;
  - đồ trên vỉa hè trước từng ô cửa hàng (`src/dressing.rs`):
    - xe máy dựng đầu vào tường, bàn ghế nhựa, chậu cây, thùng rác;
    - đặt theo bản đồ, chỉ trên vỉa hè còn trống;
    - món nào được nộp thì hiện ra món đó;
  - bờ Hồ Gươm có rào trên bản đồ đi lại, nên không ai bước qua lan can xuống nước;
  - ban đêm cho Chương 1 "Đêm mất Bơ" (`src/night.rs`):
    - trăng xanh thấp;
    - đèn đường, đèn ven hồ và đèn lồng Hàng Mã chiếu sáng thật, bật 20 ngọn gần người chơi nhất;
    - mỗi ngọn chỉ sáng khi đã có mô hình cột đèn của nó, nên không có đèn lơ lửng giữa trời;
    - khoảng một phần ba số cửa hàng hắt đèn ra vỉa hè;
    - mặt đường ướt phản chiếu đèn;
    - đèn pin trong tay Minh (mô hình đèn pin gắn vào tay khi agent nộp `people/den_pin.glb`);
  - mỗi bản LOD chỉ tải khi lần đầu cần tới; người chơi không lại gần một công trình thì không tải bản đẹp nhất của nó;
  - mipmap và lọc anisotropic cho mọi texture, tính dần từng hàng, mỗi khung hình tối đa 4 ms, để
    đi vào phố mới không bị khựng;
  - nhân vật dùng chung 43 động tác qua khung xương 65 xương, hoặc dùng động tác có sẵn trong file;
  - WebGPU có bầu trời vật lý, ánh sáng bầu trời, bóng đổ mềm nhiều tầng, SSAO, bloom, TAA.
    WebGL2 dùng bản gọn hơn.

## Chạy trên máy

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.130 --locked
./build.sh                                   # ra dist/gpu (WebGPU) và dist/gl (WebGL2)
python3 -m http.server 8000 --directory dist  # mở http://localhost:8000
```

Biến môi trường khi build:
- `ONLY=gpu` hoặc `ONLY=gl`: chỉ build một bản.
- `SKIP_OPT=1`: bỏ bước wasm-opt, build nhanh hơn.

Thêm vào cuối địa chỉ trang:
- `?gl`: dùng bản WebGL2.
- `?gpu`: thử lại WebGPU sau khi trang đã tự chuyển sang WebGL2.
- `?off=atmo,env,ssao,taa,bloom,shadows`: tắt từng hiệu ứng, cho máy yếu hoặc khi dò lỗi.
- `#peek`: Minh bắt đầu ngay cạnh lồng chó.
- `?speed=0.001` … `?speed=4`: chạy game chậm hoặc nhanh hơn thời gian thật. Dùng để quay video trên máy yếu.
  - Game tiến nhiều nhất 0,05 giây mỗi khung hình. Nếu máy vẽ mỗi khung hình mất T giây, chọn tốc độ khoảng
    1 / (24 × T).
  - Ví dụ máy không có card đồ hoạ vẽ mỗi khung hình khoảng 10 giây: quay ở `?speed=0.004`, rồi tua video
    nhanh lên 250 lần (`ffmpeg -i in.webm -vf setpts=0.004*PTS out.mp4`).
- `?time=night` hoặc `?time=day`: luôn là đêm hoặc ngày, kể cả với `?look=…`. Mặc định Chương 1 là đêm, còn lại là
  chiều muộn.
- `?look=lineup`, `?look=thehuc`, `?look=bridge`, `?look=tree`, `?look=streetlife`: camera duyệt hình cố định, không có menu. Dùng để
  chụp tài nguyên trong game và đặt cạnh ảnh tham chiếu. `lineup` cho Minh, tên trộm và chó đứng ở đầu cầu Thê Húc; `streetlife` nhìn bộ bàn ghế (hoặc món đồ vỉa hè) gần nhất từ lòng đường, ngang tầm mắt.

Nếu WebGPU mất GPU (lỗi driver hoặc trình duyệt), trang tự chuyển sang WebGL2 và nhớ lựa chọn đó
trong phiên.

`examples/webgpu_probe.rs` là cảnh Bevy nhỏ nhất (một khối lập phương), dùng để xem trình duyệt nào
chạy được Bevy trên WebGPU:

```sh
cargo build --release --target wasm32-unknown-unknown --no-default-features --features webgpu --example webgpu_probe
wasm-bindgen --target web --no-typescript --out-dir dist/probe target/wasm32-unknown-unknown/release/examples/webgpu_probe.wasm
```

Trang thử cần một `<canvas id="game">` và `import init from "./probe/webgpu_probe.js"; init();`.

## Điều khiển (máy tính)

| Việc | Phím, chuột |
|---|---|
| Đi | WASD hoặc mũi tên |
| Lén | Giữ Shift |
| Nhìn quanh | Bấm vào game để khoá chuột, rồi di chuột |
| Gần, xa | Con lăn chuột |
| Đánh | Chuột trái hoặc J |
| Ném dép | Chuột phải, Q hoặc K |
| Lăn | Space |
| Mở lồng, trói | Giữ E |
| Tạm dừng | Esc (trả lại con trỏ chuột) |

Camera không xuyên qua tường nhà. Chạm vào màn hình thì nút cảm ứng hiện ra, nhưng bản này làm cho
máy tính.

## Cấu trúc

| File | Làm gì |
|---|---|
| `src/world.rs` | Dựng phố 3D từ dữ liệu, đánh dấu chỗ của công trình, cây và mặt tiền cho tài nguyên thật |
| `src/hq.rs` | Đọc tài nguyên đã nộp (`assets/hq/`), đặt vào đúng chỗ, LOD, vật liệu nền |
| `src/facade.rs` | Cắt mặt nhà nhìn ra phố thành ô mô-đun; dựng và gỡ mô-đun theo khu |
| `src/dressing.rs` | Đồ trên vỉa hè trước cửa hàng: xe máy, bàn ghế nhựa, chậu cây, thùng rác |
| `src/render.rs` | Ánh sáng và hậu kỳ cho WebGPU và WebGL2, công tắc hiệu ứng |
| `src/night.rs` | Ban đêm: trăng, đèn đường, đèn lồng, đường ướt, đèn pin |
| `src/guide.rs` | Dẫn đường: cột sáng ở mục tiêu, la bàn, tầm nhìn của trộm trên mặt đất, dấu ? và !, lời mẹo đúng lúc |
| `src/mips.rs` | Mipmap và lọc anisotropic cho texture |
| `src/actors.rs` | Nhân vật, chó, hoạt ảnh, đồ vật |
| `src/main.rs` | Vòng chơi, camera góc thứ ba, chuột và phím, HUD, các màn hình, lưu game |
| `tools/` | Lấy bản đồ (`fetch_osm.sh`), dựng tài nguyên tạm (`make_assets.py`), nguồn (`SOURCES.md`) |

Nguồn và giấy phép của mọi tài nguyên: [CREDITS.md](CREDITS.md).

## Giới hạn hiện tại

- Phần lớn hình vẫn là hàng tạm cho tới khi có tài nguyên thật (ASSETS.md mục 11).
- Bản WebGPU chưa chạy thử được ở máy build: Chromium chạy GPU giả lập (SwiftShader) mất GPU ngay
  khi Bevy khởi động, kể cả với cảnh nhỏ nhất. Trang có cơ chế tự chuyển sang WebGL2 cho trường hợp
  này. Bản WebGL2 đã chạy thử.
- Chưa đo khung hình trên máy thật. Lần đầu tải khoảng 22 MB qua mạng (wasm khoảng 14 MB nén, build
  tối ưu tốc độ chứ không tối ưu dung lượng; tài nguyên khoảng 7 MB), trình duyệt lưu lại cho lần sau.
- Chưa có âm thanh và người đi đường.
- Phần nén tài nguyên cho web (KTX2, meshopt) sẽ làm khi có lô tài nguyên thật đầu tiên, để đo
  trên dữ liệu thật.

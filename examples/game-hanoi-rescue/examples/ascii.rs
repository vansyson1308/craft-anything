use hanoi_rescue::map::{H, Map, Tile, W};
fn main() {
    let m = Map::hoan_kiem();
    let (x0, x1, y0, y1) = match std::env::args().nth(1).as_deref() {
        Some("all") => (0, W, 0, H),
        _ => (30, 72, 34, 80),
    };
    for y in y0..y1 {
        let row: String = (x0..x1)
            .map(|x| match m.tile(x, y) {
                Tile::Street => '.',
                Tile::Alley => ',',
                Tile::Park => '"',
                Tile::Tree => 'T',
                Tile::Water => '~',
                Tile::River => '=',
                Tile::Building => '#',
                Tile::Market => 'M',
                Tile::Bridge => 'B',
                Tile::Truss => 'X',
                Tile::Island => 'i',
                Tile::Plaza => '_',
                Tile::Rail => '|',
                Tile::Crowd => 'c',
                Tile::Landmark => 'L',
            })
            .collect();
        println!("{y:2} {row}");
    }
}

//! Hoàn Kiếm, built from a short list of features: the old-quarter street grid, Hồ Gươm with its
//! islands and ring road, the landmarks, the railway and the river. One tile is one world unit.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{Rng, V2, hash2, v2};

pub const W: i32 = 100;
pub const H: i32 = 80;

pub const LAKE: V2 = v2(50.0, 57.0);
pub const LAKE_RX: f32 = 11.0;
pub const LAKE_RY: f32 = 14.0;
/// Centre line of the railway (tiles 10 and 11).
pub const RAIL_X: f32 = 11.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Street,
    Alley,
    Park,
    Tree,
    Water,
    River,
    Building,
    Market,
    Bridge,
    Truss,
    Island,
    Plaza,
    Rail,
    Crowd,
    Landmark,
}

impl Tile {
    pub fn walkable(self) -> bool {
        matches!(self, Tile::Street | Tile::Alley | Tile::Park | Tile::Bridge | Tile::Truss | Tile::Island | Tile::Plaza | Tile::Rail | Tile::Crowd)
    }
    pub fn blocks_sight(self) -> bool {
        matches!(self, Tile::Building | Tile::Market | Tile::Tree | Tile::Landmark)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExitId {
    QuanChuong,
    LongBien,
    NhaHat,
    Ga,
}

#[derive(Clone, Debug)]
pub struct Exit {
    pub id: ExitId,
    pub name: &'static str,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

impl Exit {
    pub fn contains(&self, p: V2) -> bool {
        p.x >= self.x0 && p.x <= self.x1 && p.y >= self.y0 && p.y <= self.y1
    }
    pub fn center(&self) -> V2 {
        v2((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }
}

#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    pub at: V2,
    /// Big labels name landmarks; small ones name streets.
    pub big: bool,
}

/// A line vehicles drive along. Closed routes loop; open ones restart at their first point.
#[derive(Clone, Debug)]
pub struct Route {
    pub pts: Vec<V2>,
    pub closed: bool,
    pub len: f32,
}

impl Route {
    pub fn new(pts: Vec<V2>, closed: bool) -> Route {
        let mut r = Route { pts, closed, len: 0.0 };
        r.len = r.segments().map(|(a, b)| a.dist(b)).sum();
        r
    }
    fn segments(&self) -> impl Iterator<Item = (V2, V2)> + '_ {
        let n = self.pts.len();
        let extra = if self.closed { 1 } else { 0 };
        (0..(n.saturating_sub(1) + extra)).filter_map(move |i| Some((*self.pts.get(i)?, *self.pts.get((i + 1) % n)?)))
    }
    /// The shortest distance from `p` to the route.
    pub fn dist(&self, p: V2) -> f32 {
        self.segments()
            .map(|(a, b)| {
                let ab = b - a;
                let t = if ab.dot(ab) > 1e-6 { ((p - a).dot(ab) / ab.dot(ab)).clamp(0.0, 1.0) } else { 0.0 };
                p.dist(a + ab * t)
            })
            .fold(f32::INFINITY, f32::min)
    }

    /// The point `s` along the route (wrapped) and the direction there.
    pub fn at(&self, s: f32) -> (V2, V2) {
        if self.len <= 0.0 {
            return (self.pts.first().copied().unwrap_or_default(), v2(1.0, 0.0));
        }
        let mut s = s.rem_euclid(self.len);
        for (a, b) in self.segments() {
            let l = a.dist(b);
            if s <= l && l > 0.0 {
                return (a.lerp(b, s / l), (b - a).norm());
            }
            s -= l;
        }
        (self.pts.last().copied().unwrap_or_default(), v2(1.0, 0.0))
    }
}

/// Where each chapter happens on a map, so the same game runs on any district.
#[derive(Clone, Debug, Default)]
pub struct Scenario {
    /// Per story chapter: x0, y0, x1, y1 in tiles.
    pub zones: Vec<[i32; 4]>,
    /// Per story chapter: where the player starts.
    pub spawns: Vec<V2>,
    pub free_spawns: Vec<V2>,
    /// Bơ's cage in the last chapter, and where the gang's boss waits.
    pub puppy_cage: V2,
    pub boss_home: V2,
    /// Where crates and thieves may be placed: x0, y0, x1, y1.
    pub play: [i32; 4],
}

pub struct Map {
    tiles: Vec<Tile>,
    pub w: i32,
    pub h: i32,
    /// Metres per tile.
    pub tile_m: f32,
    pub rail: Option<Route>,
    pub scenario: Scenario,
    pub exits: Vec<Exit>,
    pub labels: Vec<Label>,
    pub routes: Vec<Route>,
    /// Lanterns over Hàng Mã: position and whether red (else yellow).
    pub lanterns: Vec<(V2, bool)>,
}

/// How far a point is inside the lake ellipse grown by `grow` tiles (<= 1 is inside).
pub fn lake_ell(p: V2, grow: f32) -> f32 {
    let dx = (p.x - LAKE.x) / (LAKE_RX + grow);
    let dy = (p.y - LAKE.y) / (LAKE_RY + grow);
    dx * dx + dy * dy
}

pub fn center(x: i32, y: i32) -> V2 {
    v2(x as f32 + 0.5, y as f32 + 0.5)
}

impl Map {
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return Tile::Building;
        }
        self.tiles.get((y * self.w + x) as usize).copied().unwrap_or(Tile::Building)
    }
    pub fn tile_at(&self, p: V2) -> Tile {
        let (x, y) = p.tile();
        self.tile(x, y)
    }
    fn set(&mut self, x: i32, y: i32, t: Tile) {
        if x >= 0
            && y >= 0
            && x < self.w
            && y < self.h
            && let Some(c) = self.tiles.get_mut((y * self.w + x) as usize)
        {
            *c = t;
        }
    }
    fn fill(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, t: Tile) {
        self.fill_where(x0, y0, x1, y1, t, |_| true);
    }
    fn fill_where(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, t: Tile, when: impl Fn(Tile) -> bool) {
        for y in y0..y1 {
            for x in x0..x1 {
                if when(self.tile(x, y)) {
                    self.set(x, y, t);
                }
            }
        }
    }

    pub fn walkable_at(&self, p: V2) -> bool {
        self.tile_at(p).walkable()
    }

    /// Whether a circle of radius `r` at `p` overlaps only walkable tiles.
    pub fn circle_free(&self, p: V2, r: f32) -> bool {
        let (x0, y0) = v2(p.x - r, p.y - r).tile();
        let (x1, y1) = v2(p.x + r, p.y + r).tile();
        for y in y0..=y1 {
            for x in x0..=x1 {
                if self.tile(x, y).walkable() {
                    continue;
                }
                // Closest point of the tile to the circle's centre.
                let cx = p.x.clamp(x as f32, x as f32 + 1.0);
                let cy = p.y.clamp(y as f32, y as f32 + 1.0);
                if v2(cx, cy).dist(p) < r {
                    return false;
                }
            }
        }
        true
    }

    /// Moves a circle by `d`, sliding along walls. Returns the new position.
    pub fn slide(&self, p: V2, d: V2, r: f32) -> V2 {
        // Long moves (knockback, a dash) go in steps so nothing passes through a wall.
        let steps = (d.len() / 0.3).ceil().clamp(1.0, 12.0) as i32;
        let step = d * (1.0 / steps as f32);
        let mut q = p;
        for _ in 0..steps {
            let nx = v2(q.x + step.x, q.y);
            if self.circle_free(nx, r) {
                q = nx;
            }
            let ny = v2(q.x, q.y + step.y);
            if self.circle_free(ny, r) {
                q = ny;
            }
        }
        q
    }

    /// Whether nothing that blocks sight lies between `a` and `b`.
    pub fn line_of_sight(&self, a: V2, b: V2) -> bool {
        let d = b - a;
        let n = (d.len() / 0.25).ceil().clamp(1.0, 400.0) as i32;
        for i in 1..n {
            let p = a + d * (i as f32 / n as f32);
            if self.tile_at(p).blocks_sight() {
                return false;
            }
        }
        true
    }

    /// Steps from `from` to every tile (4-neighbour walk); -1 where unreachable.
    pub fn distances(&self, from: (i32, i32)) -> Vec<i32> {
        let mut dist = vec![-1i32; (self.w * self.h) as usize];
        let mut queue = std::collections::VecDeque::new();
        if self.tile(from.0, from.1).walkable() {
            if let Some(d) = dist.get_mut((from.1 * self.w + from.0) as usize) {
                *d = 0;
            }
            queue.push_back(from);
        }
        while let Some((x, y)) = queue.pop_front() {
            let here = dist.get((y * self.w + x) as usize).copied().unwrap_or(0);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !self.tile(nx, ny).walkable() {
                    continue;
                }
                if let Some(d) = dist.get_mut((ny * self.w + nx) as usize)
                    && *d < 0
                {
                    *d = here + 1;
                    queue.push_back((nx, ny));
                }
            }
        }
        dist
    }

    /// A* over walkable tiles (8 directions, no cutting corners). Returns tile centres after the
    /// start, ending at the goal; None if the goal can't be reached within `budget` expansions.
    pub fn path(&self, from: V2, to: V2, budget: usize) -> Option<Vec<V2>> {
        let start = from.tile();
        let goal = to.tile();
        if !self.tile(goal.0, goal.1).walkable() {
            return None;
        }
        if start == goal {
            return Some(vec![to]);
        }
        let key = |p: (i32, i32)| (p.1 * self.w + p.0) as u32;
        let h = |p: (i32, i32)| {
            let dx = (p.0 - goal.0).abs();
            let dy = (p.1 - goal.1).abs();
            10 * (dx + dy) - 6 * dx.min(dy)
        };
        // Cost so far and the tile we came from, only for tiles the search touched.
        let mut seen: std::collections::HashMap<u32, (i32, u32)> = std::collections::HashMap::new();
        let mut open = BinaryHeap::new();
        seen.insert(key(start), (0, u32::MAX));
        open.push(Reverse((h(start), start.0, start.1)));
        let mut expanded = 0usize;
        while let Some(Reverse((_, x, y))) = open.pop() {
            if (x, y) == goal {
                let mut out = Vec::new();
                let mut cur = goal;
                let mut guard = 0;
                while cur != start && guard < budget + 8 {
                    out.push(center(cur.0, cur.1));
                    let prev = seen.get(&key(cur)).map_or(u32::MAX, |e| e.1);
                    if prev == u32::MAX {
                        break;
                    }
                    cur = ((prev % self.w as u32) as i32, (prev / self.w as u32) as i32);
                    guard += 1;
                }
                out.reverse();
                if let Some(last) = out.last_mut() {
                    *last = to;
                }
                return Some(out);
            }
            expanded += 1;
            if expanded > budget {
                return None;
            }
            let here = seen.get(&key((x, y))).map_or(i32::MAX, |e| e.0);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let n = (x + dx, y + dy);
                if !self.tile(n.0, n.1).walkable() {
                    continue;
                }
                if dx != 0 && dy != 0 && (!self.tile(x + dx, y).walkable() || !self.tile(x, y + dy).walkable()) {
                    continue;
                }
                let cost = here.saturating_add(if dx != 0 && dy != 0 { 14 } else { 10 });
                let better = seen.get(&key(n)).is_none_or(|e| cost < e.0);
                if better {
                    seen.insert(key(n), (cost, key((x, y))));
                    open.push(Reverse((cost + h(n), n.0, n.1)));
                }
            }
        }
        None
    }

    /// A random walkable tile centre in the box whose tile passes `ok`.
    pub fn random_spot(&self, rng: &mut Rng, x0: i32, y0: i32, x1: i32, y1: i32, ok: impl Fn(Tile, V2) -> bool) -> Option<V2> {
        for _ in 0..400 {
            let x = rng.range(x0.max(0) as usize, x1.min(self.w) as usize) as i32;
            let y = rng.range(y0.max(0) as usize, y1.min(self.h) as usize) as i32;
            let t = self.tile(x, y);
            let p = center(x, y);
            if t.walkable() && ok(t, p) {
                return Some(p);
            }
        }
        None
    }

    /// Colour of each tile for the minimap, as RGBA bytes (self.w × self.h).
    pub fn minimap_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity((self.w * self.h * 4) as usize);
        for y in 0..self.h {
            for x in 0..self.w {
                let c = match self.tile(x, y) {
                    Tile::Street | Tile::Rail => 0x5a5e66,
                    Tile::Alley => 0x4a4d54,
                    Tile::Crowd => 0x7a5c5c,
                    Tile::Plaza => 0x6b6b70,
                    Tile::Park => 0x3e6b45,
                    Tile::Tree => 0x2f5536,
                    Tile::Water => 0x3f8f86,
                    Tile::River => 0x8a6f4d,
                    Tile::Bridge => 0xc0392b,
                    Tile::Truss => 0x8e5a3c,
                    Tile::Island => 0x6f8b55,
                    Tile::Market | Tile::Landmark => 0xc9b38a,
                    Tile::Building => 0x2b2c31,
                };
                out.extend_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 235]);
            }
        }
        out
    }

    /// The district.
    pub fn hoan_kiem() -> Map {
        let mut m = Map {
            tiles: vec![Tile::Building; (W * H) as usize],
            w: W,
            h: H,
            tile_m: 1.0,
            rail: Some(Route::new(vec![v2(RAIL_X, -14.0), v2(RAIL_X, H as f32 + 14.0)], false)),
            scenario: Scenario {
                zones: vec![[14, 2, 46, 26], [54, 14, 76, 34], [30, 0, 62, 17], [0, 10, 22, 72], [76, 0, 89, 12]],
                spawns: vec![v2(25.0, 39.0), v2(35.0, 39.0), v2(20.0, 39.0), v2(40.0, 47.0), v2(25.0, 39.0)],
                free_spawns: vec![v2(5.0, 47.0), v2(25.0, 39.0), v2(45.0, 73.0), v2(75.0, 47.0), v2(5.0, 3.0)],
                puppy_cage: v2(86.5, 8.5),
                boss_home: v2(84.5, 9.0),
                play: [0, 0, 89, 80],
            },
            exits: Vec::new(),
            labels: Vec::new(),
            routes: Vec::new(),
            lanterns: Vec::new(),
        };
        let mut rng = Rng::new(1010);

        // The street grid: the old quarter in the north, the French quarter in the south.
        for y in [2, 10, 18, 26, 34, 38] {
            m.fill(0, y, 91, y + 2, Tile::Street);
        }
        for x in [4, 14, 24, 34, 44, 54, 64, 74, 84] {
            m.fill(x, 0, x + 2, 40, Tile::Street);
        }
        for y in [46, 62, 72] {
            m.fill(0, y, 91, y + 2, Tile::Street);
        }
        for x in [4, 14, 24, 74, 84] {
            m.fill(x, 40, x + 2, 80, Tile::Street);
        }
        // Ngõ: narrow alleys through (or into) the old quarter's blocks.
        for bx in [14, 24, 34, 44, 54, 64, 74] {
            for by in [2, 10, 18, 26] {
                if !rng.chance(0.7) {
                    continue;
                }
                if rng.chance(0.5) {
                    let x = bx + 2 + rng.range(2, 7) as i32;
                    let (y0, y1) = if rng.chance(0.7) { (by + 2, by + 8) } else { (by + 2, by + 5) };
                    m.fill_where(x, y0, x + 1, y1, Tile::Alley, |t| t == Tile::Building);
                } else {
                    let y = by + 2 + rng.range(1, 5) as i32;
                    let (x0, x1) = if rng.chance(0.7) { (bx + 2, bx + 10) } else { (bx + 2, bx + 6) };
                    m.fill_where(x0, y, x1, y + 1, Tile::Alley, |t| t == Tile::Building);
                }
            }
        }

        // Hồ Gươm: water, a park ring with trees, then the ring road.
        for y in 30..H {
            for x in 26..74 {
                let p = center(x, y);
                if lake_ell(p, 0.0) <= 1.0 {
                    m.set(x, y, Tile::Water);
                } else if lake_ell(p, 3.0) <= 1.0 {
                    let h = hash2(x, y);
                    let by_water = lake_ell(p, 0.9) <= 1.0;
                    let outer = lake_ell(p, 2.0) > 1.0;
                    let tree = (by_water && h.is_multiple_of(4)) || (outer && h.is_multiple_of(3));
                    m.set(x, y, if tree { Tile::Tree } else { Tile::Park });
                } else if lake_ell(p, 5.2) <= 1.0 {
                    m.set(x, y, Tile::Street);
                }
            }
        }
        // Đền Ngọc Sơn's island and the red Thê Húc bridge; Tháp Rùa on its own islet.
        for y in 43..50 {
            for x in 49..56 {
                if center(x, y).dist(v2(52.0, 46.5)) <= 1.9 {
                    m.set(x, y, Tile::Island);
                }
            }
        }
        m.fill(51, 46, 53, 47, Tile::Landmark);
        m.fill_where(51, 39, 53, 45, Tile::Bridge, |t| matches!(t, Tile::Water | Tile::Park | Tile::Tree));
        m.set(49, 61, Tile::Landmark);

        // Landmarks.
        m.fill(36, 4, 54, 10, Tile::Market); // Chợ Đồng Xuân
        m.fill(26, 49, 32, 55, Tile::Landmark); // Nhà thờ Lớn
        m.fill(25, 55, 33, 58, Tile::Crowd); // its square, where people drink lemon tea
        m.fill(71, 64, 77, 70, Tile::Plaza); // in front of Nhà hát Lớn
        m.fill(77, 64, 84, 70, Tile::Landmark); // Nhà hát Lớn
        m.fill(76, 4, 89, 10, Tile::Plaza); // the yard by the bridge
        m.fill(79, 5, 85, 8, Tile::Landmark); // the gang's warehouse

        // Phố Tạ Hiện: crowded with plastic stools.
        m.fill_where(56, 18, 74, 20, Tile::Crowd, |t| t == Tile::Street);
        m.fill_where(64, 18, 66, 34, Tile::Crowd, |t| t == Tile::Street);

        // The railway, the dyke road, the riverbank and the Red River; Long Biên across it.
        m.fill(10, 0, 12, H, Tile::Rail);
        m.fill(6, 76, 14, H, Tile::Plaza);
        m.fill(10, 76, 12, H, Tile::Rail);
        m.fill(89, 0, 91, H, Tile::Street);
        m.fill(91, 0, 93, H, Tile::Park);
        m.fill(93, 0, W, H, Tile::River);
        m.fill(89, 5, W, 7, Tile::Truss);

        m.exits = vec![
            Exit { id: ExitId::QuanChuong, name: "Ô Quan Chưởng", x0: 86.5, y0: 17.6, x1: 91.0, y1: 20.4 },
            Exit { id: ExitId::LongBien, name: "Cầu Long Biên", x0: 95.0, y0: 4.8, x1: 100.0, y1: 7.2 },
            Exit { id: ExitId::NhaHat, name: "Nhà hát Lớn", x0: 71.0, y0: 64.5, x1: 76.5, y1: 69.5 },
            Exit { id: ExitId::Ga, name: "Ga Hà Nội", x0: 6.0, y0: 76.5, x1: 14.0, y1: 80.0 },
        ];

        let big = |text: &str, x, y| Label { text: text.to_string(), at: v2(x, y), big: true };
        let small = |text: &str, x, y| Label { text: text.to_string(), at: v2(x, y), big: false };
        m.labels = vec![
            big("HỒ GƯƠM", 50.0, 54.0),
            small("Tháp Rùa", 49.5, 63.2),
            small("Đền Ngọc Sơn", 52.0, 49.2),
            small("Cầu Thê Húc", 55.4, 41.6),
            big("NHÀ THỜ LỚN", 29.0, 48.6),
            big("CHỢ ĐỒNG XUÂN", 45.0, 7.2),
            big("NHÀ HÁT LỚN", 80.5, 63.6),
            big("Ô QUAN CHƯỞNG", 87.5, 16.8),
            big("CẦU LONG BIÊN", 96.0, 4.2),
            big("SÔNG HỒNG", 96.5, 40.0),
            small("Phố Hàng Mã", 22.0, 11.1),
            small("Phố Tạ Hiện", 69.0, 19.1),
            small("Phố Hàng Bạc", 40.0, 27.1),
            small("Phố Hàng Đào", 45.0, 31.0),
            small("Phố Mã Mây", 75.0, 30.0),
            small("Phố đường tàu", 11.0, 36.0),
            small("Phố Tràng Tiền", 76.0, 63.1),
            small("Phố Hàng Gai", 28.0, 39.1),
            small("Đinh Tiên Hoàng", 67.0, 50.0),
            small("Kho bãi", 82.0, 9.2),
        ];

        // Traffic: around the lake, along Tràng Tiền, the dyke road and Hàng Mã.
        let ring: Vec<V2> = (0..72)
            .map(|i| {
                let a = i as f32 / 72.0 * std::f32::consts::TAU;
                v2(LAKE.x + (LAKE_RX + 4.1) * a.cos(), LAKE.y + (LAKE_RY + 4.1) * a.sin())
            })
            .collect();
        m.routes = vec![
            Route::new(ring, true),
            Route::new(vec![v2(64.6, 63.0), v2(102.0, 63.0)], false),
            Route::new(vec![v2(90.0, -3.0), v2(90.0, 83.0)], false),
            Route::new(vec![v2(-3.0, 11.0), v2(89.6, 11.0)], false),
        ];

        let mut x = 15.0;
        let mut red = true;
        while x < 44.0 {
            m.lanterns.push((v2(x, 9.85), red));
            m.lanterns.push((v2(x + 0.6, 12.15), !red));
            red = !red;
            x += 1.3;
        }
        m
    }
}

// ---------------------------------------------------------------------------------------------
// Real districts, from OpenStreetMap data.

use crate::city::{City, Dm};

/// The real landmarks the story uses, by their names in the map data.
pub mod places {
    pub const HANG_MA: &str = "Phố Hàng Mã";
    pub const TA_HIEN: &str = "Phố Tạ Hiện";
    pub const LUONG_NGOC_QUYEN: &str = "Phố Lương Ngọc Quyến";
    pub const DONG_XUAN: &str = "Chợ Đồng Xuân";
    pub const PHUNG_HUNG: &str = "Phố Phùng Hưng";
    pub const LONG_BIEN: &str = "Cầu Long Biên";
    pub const QUAN_CHUONG: &str = "Ô Quan Chưởng";
    pub const OPERA: &str = "Nhà hát Lớn";
    pub const STATION: &str = "Ga Long Biên";
    pub const CATHEDRAL: &str = "Nhà thờ Lớn";
    pub const TURTLE_TOWER: &str = "Tháp Rùa";
    pub const NGOC_SON: &str = "Đền Ngọc Sơn";
    pub const LAKE: &str = "Hồ Hoàn Kiếm";
    pub const DINH_TIEN_HOANG: &str = "Phố Đinh Tiên Hoàng";
}

impl Map {
    /// Rasterises a real district: `tile_m` metres per tile.
    pub fn from_city(c: &City, tile_m: f32) -> Map {
        let tile_m = tile_m.clamp(0.5, 10.0);
        let [bx0, by0, bx1, by1] = c.bounds;
        let w = (((bx1 - bx0) as f32 / 10.0) / tile_m).ceil().clamp(16.0, 4000.0) as i32;
        let h = (((by1 - by0) as f32 / 10.0) / tile_m).ceil().clamp(16.0, 4000.0) as i32;
        let to = |p: &Dm| -> V2 { v2((p[0] - bx0) as f32 / 10.0 / tile_m, (p[1] - by0) as f32 / 10.0 / tile_m) };
        let ring = |pts: &[Dm]| -> Vec<V2> { pts.iter().map(to).collect() };
        let mut m = Map {
            tiles: vec![Tile::Plaza; (w * h) as usize],
            w,
            h,
            tile_m,
            rail: None,
            scenario: Scenario::default(),
            exits: Vec::new(),
            labels: Vec::new(),
            routes: Vec::new(),
            lanterns: Vec::new(),
        };

        for a in &c.parks {
            let mut rings = vec![ring(&a.outer)];
            rings.extend(a.holes.iter().map(|h| ring(h)));
            m.fill_rings(&rings, |_| Tile::Park);
        }
        for a in &c.water {
            let mut rings = vec![ring(&a.outer)];
            rings.extend(a.holes.iter().map(|h| ring(h)));
            m.fill_rings(&rings, |_| Tile::Water);
            for hole in &a.holes {
                m.fill_rings(&[ring(hole)], |_| Tile::Island);
            }
        }
        let crowd = [places::TA_HIEN, places::LUONG_NGOC_QUYEN];
        for r in c.roads.iter().filter(|r| !r.bridge) {
            let pts = ring(&r.pts);
            let half = r.w as f32 / 20.0 / tile_m;
            let busy = r.name.as_deref().is_some_and(|n| crowd.contains(&n));
            let t = if busy {
                Tile::Crowd
            } else if r.w >= 50 {
                Tile::Street
            } else {
                Tile::Alley
            };
            m.stamp_line(&pts, half.max(0.5), |old| if matches!(old, Tile::Water | Tile::River) { old } else { t });
        }
        for r in &c.rails {
            m.stamp_line(&ring(r), 1.5 / tile_m, |old| if matches!(old, Tile::Water | Tile::River) { old } else { Tile::Rail });
        }
        for b in &c.buildings {
            let mut rings = vec![ring(&b.outer)];
            rings.extend(b.holes.iter().map(|h| ring(h)));
            m.fill_rings(&rings, |_| Tile::Building);
        }
        for r in c.roads.iter().filter(|r| r.bridge) {
            let half = (r.w as f32 / 20.0 / tile_m).max(0.75);
            m.stamp_line(&ring(&r.pts), half, |_| Tile::Bridge);
        }
        for t in &c.trees {
            let p = to(t);
            let (x, y) = p.tile();
            if matches!(m.tile(x, y), Tile::Park) {
                m.set(x, y, Tile::Tree);
            }
        }

        // The railway the train runs on: the longest chain of joined rail lines.
        m.rail = longest_chain(&c.rails).map(|chain| Route::new(chain.iter().map(to).collect(), false));

        // Traffic on the main roads.
        let mut main: Vec<&crate::city::Road> =
            c.roads.iter().filter(|r| matches!(r.kind.as_str(), "primary" | "secondary" | "tertiary") && !r.bridge && r.pts.len() >= 3).collect();
        let len = |r: &crate::city::Road| {
            r.pts
                .windows(2)
                .map(|w| match w {
                    [a, b] => to(a).dist(to(b)),
                    _ => 0.0,
                })
                .sum::<f32>()
        };
        main.sort_by(|a, b| len(b).total_cmp(&len(a)));
        m.routes = main.iter().take(14).map(|r| Route::new(ring(&r.pts), false)).collect();

        // Names: landmarks big, each named street once.
        let at = |name: &str| c.place(name).or_else(|| c.road(name)).map(|p| to(&p));
        for (text, name) in [
            ("HỒ GƯƠM", places::LAKE),
            ("THÁP RÙA", places::TURTLE_TOWER),
            ("ĐỀN NGỌC SƠN", places::NGOC_SON),
            ("NHÀ THỜ LỚN", places::CATHEDRAL),
            ("NHÀ HÁT LỚN", places::OPERA),
            ("CHỢ ĐỒNG XUÂN", places::DONG_XUAN),
            ("Ô QUAN CHƯỞNG", places::QUAN_CHUONG),
            ("GA LONG BIÊN", places::STATION),
            ("CẦU LONG BIÊN", places::LONG_BIEN),
        ] {
            let pos = if name == places::LAKE {
                c.water.iter().find(|w| w.name.as_deref() == Some(places::LAKE)).and_then(|w| crate::city::centroid(&w.outer)).map(|p| to(&p))
            } else {
                at(name)
            };
            if let Some(p) = pos {
                m.labels.push(Label { text: text.to_string(), at: p, big: true });
            }
        }
        let mut named: Vec<&str> = Vec::new();
        for r in &c.roads {
            let Some(n) = r.name.as_deref() else { continue };
            if named.contains(&n) || r.pts.len() < 2 || r.w < 50 {
                continue;
            }
            named.push(n);
            if let Some(p) = r.pts.get(r.pts.len() / 2) {
                m.labels.push(Label { text: n.to_string(), at: to(p), big: false });
            }
        }

        // Lanterns over Hàng Mã.
        if let Some(r) = c.roads.iter().filter(|r| r.name.as_deref() == Some(places::HANG_MA)).max_by_key(|r| r.pts.len()) {
            let pts = ring(&r.pts);
            for (i, seg) in pts.windows(2).enumerate() {
                if let [a, b] = seg {
                    let n = (a.dist(*b) / 1.5).ceil().max(1.0) as i32;
                    for k in 0..n {
                        m.lanterns.push((a.lerp(*b, k as f32 / n as f32), (i as i32 + k) % 2 == 0));
                    }
                }
            }
        }

        // Exits and the story's places, by name; each nudged onto a walkable tile.
        let near = |m: &Map, p: V2| m.nearest_walkable(p);
        let exit = |m: &Map, id: ExitId, name: &'static str, p: Option<V2>, r: f32| {
            p.map(|p| {
                let q = near(m, p);
                Exit { id, name, x0: q.x - r, y0: q.y - r, x1: q.x + r, y1: q.y + r }
            })
        };
        let r = 6.0 / tile_m;
        let bridge_end = c
            .roads
            .iter()
            .filter(|rd| rd.name.as_deref() == Some(places::LONG_BIEN))
            .flat_map(|rd| rd.pts.iter())
            .filter(|p| p[0] < bx1 - 300 && p[1] > by0 + 300)
            .max_by_key(|p| p[0])
            .map(to);
        let exits = [
            exit(&m, ExitId::QuanChuong, "Ô Quan Chưởng", at(places::QUAN_CHUONG), r),
            exit(&m, ExitId::LongBien, "Cầu Long Biên", bridge_end, r),
            exit(&m, ExitId::NhaHat, "Nhà hát Lớn", at(places::OPERA), r * 1.5),
            exit(&m, ExitId::Ga, "Ga Long Biên", at(places::STATION), r * 1.5),
        ];
        m.exits = exits.into_iter().flatten().collect();

        let zone = |p: Option<V2>, half_m: f32| -> [i32; 4] {
            let p = p.unwrap_or(v2(w as f32 / 2.0, h as f32 / 2.0));
            let k = half_m / tile_m;
            [(p.x - k) as i32, (p.y - k) as i32, (p.x + k) as i32, (p.y + k) as i32]
        };
        let lake = at(places::DINH_TIEN_HOANG).unwrap_or(v2(w as f32 / 2.0, h as f32 / 2.0));
        let cathedral = at(places::CATHEDRAL).map(|p| near(&m, p + v2(0.0, 25.0 / tile_m)));
        let start = |p: Option<V2>| near(&m, p.unwrap_or(lake));
        let warehouse = bridge_end.map(|_| ()).and(at(places::QUAN_CHUONG)).map(|p| p + v2(0.0, -150.0 / tile_m));
        // Each chapter starts a short walk outside its zone, on the side toward the lake: time to
        // take in the street and see where to go, not minutes of empty road before anything happens.
        let approach = |c: Option<V2>, half_m: f32| {
            c.map(|c| {
                let d = lake - c;
                let len = d.len();
                if len < 1e-3 { c } else { c + d * ((half_m + APPROACH_M) / tile_m / len).min(1.0) }
            })
        };
        let scenario = Scenario {
            zones: vec![
                zone(at(places::HANG_MA), 90.0),
                zone(at(places::TA_HIEN), 90.0),
                zone(at(places::DONG_XUAN), 100.0),
                zone(at(places::PHUNG_HUNG), 160.0),
                zone(warehouse, 70.0),
            ],
            spawns: vec![
                start(approach(at(places::HANG_MA), 90.0)),
                start(approach(at(places::TA_HIEN), 90.0)),
                start(approach(at(places::DONG_XUAN), 100.0)),
                start(approach(at(places::PHUNG_HUNG), 160.0)),
                start(approach(warehouse, 70.0)),
            ],
            free_spawns: vec![start(cathedral), start(Some(lake)), start(at(places::OPERA))],
            puppy_cage: start(warehouse),
            boss_home: start(warehouse.map(|p| p + v2(4.0, 3.0))),
            play: [0, 0, w, h],
        };
        // Everything the story needs must be reachable from the lake's ring road, the heart of the
        // district: move each place onto the nearest tile that is.
        let hub = near(&m, lake);
        let reach = m.distances(hub.tile());
        let onto = |p: V2| m.nearest_reachable(p, &reach);
        let new_scenario = Scenario {
            spawns: scenario.spawns.iter().map(|p| onto(*p)).collect(),
            free_spawns: scenario.free_spawns.iter().map(|p| onto(*p)).collect(),
            puppy_cage: onto(scenario.puppy_cage),
            boss_home: onto(scenario.boss_home),
            ..scenario
        };
        let exits: Vec<Exit> = m
            .exits
            .iter()
            .map(|e| {
                let (hw, hh) = ((e.x1 - e.x0) / 2.0, (e.y1 - e.y0) / 2.0);
                let c = onto(e.center());
                Exit { x0: c.x - hw, y0: c.y - hh, x1: c.x + hw, y1: c.y + hh, ..e.clone() }
            })
            .collect();
        m.scenario = new_scenario;
        m.exits = exits;
        m
    }

    /// The centre of the tile nearest the point that the reach map (from distances) gets to.
    pub fn nearest_reachable(&self, p: V2, reach: &[i32]) -> V2 {
        let (x, y) = p.tile();
        for r in 0..200i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let (tx, ty) = (x + dx, y + dy);
                    if tx < 0 || ty < 0 || tx >= self.w || ty >= self.h {
                        continue;
                    }
                    let ok = reach.get((ty * self.w + tx) as usize).is_some_and(|d| *d >= 0);
                    if ok && self.tile(tx, ty) != Tile::Rail {
                        return center(tx, ty);
                    }
                }
            }
        }
        p
    }

    /// The centre of the walkable tile nearest `p` (searching up to 60 tiles away).
    pub fn nearest_walkable(&self, p: V2) -> V2 {
        let (x, y) = p.tile();
        for r in 0..60i32 {
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dy.abs() != r {
                        continue;
                    }
                    let t = self.tile(x + dx, y + dy);
                    if t.walkable() && t != Tile::Rail {
                        return center(x + dx, y + dy);
                    }
                }
            }
        }
        p
    }

    /// Fills the area inside the rings (even-odd, so inner rings are holes), row by row.
    fn fill_rings(&mut self, rings: &[Vec<V2>], tile: impl Fn(Tile) -> Tile) {
        let (mut ymin, mut ymax) = (f32::INFINITY, f32::NEG_INFINITY);
        for r in rings {
            for p in r {
                ymin = ymin.min(p.y);
                ymax = ymax.max(p.y);
            }
        }
        if !ymin.is_finite() {
            return;
        }
        let y0 = (ymin.floor() as i32).max(0);
        let y1 = (ymax.ceil() as i32).min(self.h - 1);
        let mut xs: Vec<f32> = Vec::new();
        for y in y0..=y1 {
            let yc = y as f32 + 0.5;
            xs.clear();
            for r in rings {
                let n = r.len();
                for i in 0..n {
                    let (Some(a), Some(b)) = (r.get(i), r.get((i + 1) % n)) else { continue };
                    if (a.y > yc) != (b.y > yc) {
                        xs.push(a.x + (yc - a.y) / (b.y - a.y) * (b.x - a.x));
                    }
                }
            }
            xs.sort_by(f32::total_cmp);
            for pair in xs.chunks(2) {
                if let [l, r] = pair {
                    let xa = ((l - 0.5).ceil() as i32).max(0);
                    let xb = ((r - 0.5).floor() as i32).min(self.w - 1);
                    for x in xa..=xb {
                        let t = tile(self.tile(x, y));
                        self.set(x, y, t);
                    }
                }
            }
        }
    }

    /// Puts a railing round the water called `name`: the tiles are 2 m wide and count as water only
    /// when their centre is in it, so a shore tile could hold someone a metre and more out over
    /// the water. Every walkable tile whose centre is outside the water but any part of which is within
    /// `margin_m` of its shore becomes water too, except within `gap_m` of a bridge, where people
    /// step on and off. Returns whether the water was found.
    pub fn fence_shore(&mut self, c: &City, name: &str, margin_m: f32, gap_m: f32) -> bool {
        let tm = self.tile_m;
        let to = |p: &Dm| -> V2 { v2((p[0] - c.bounds[0]) as f32 / 10.0 / tm, (p[1] - c.bounds[1]) as f32 / 10.0 / tm) };
        let Some(lake) = c.water.iter().find(|w| w.name.as_deref() == Some(name)) else { return false };
        let ring: Vec<V2> = lake.outer.iter().map(to).collect();
        let bridges: Vec<Vec<V2>> = c.roads.iter().filter(|r| r.bridge).map(|r| r.pts.iter().map(to).collect()).collect();
        let reach = (margin_m.max(0.0) + tm * std::f32::consts::FRAC_1_SQRT_2) / tm;
        let gap = gap_m.max(0.0) / tm;
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for q in &ring {
            x0 = x0.min(q.x);
            y0 = y0.min(q.y);
            x1 = x1.max(q.x);
            y1 = y1.max(q.y);
        }
        if ring.len() < 3 {
            return true;
        }
        let near = |p: V2, line: &[V2], d: f32| line.windows(2).any(|w| matches!(w, [a, b] if seg_dist(p, *a, *b) <= d));
        let mut closed = ring.clone();
        closed.extend(ring.first().copied());
        for y in ((y0 - reach).floor() as i32).max(0)..=((y1 + reach).ceil() as i32).min(self.h - 1) {
            for x in ((x0 - reach).floor() as i32).max(0)..=((x1 + reach).ceil() as i32).min(self.w - 1) {
                let t = self.tile(x, y);
                let p = center(x, y);
                if t.walkable() && t != Tile::Bridge && !inside(&ring, p) && near(p, &closed, reach) && !bridges.iter().any(|b| near(p, b, gap)) {
                    self.set(x, y, Tile::Water);
                }
            }
        }
        true
    }

    /// Sets every tile whose centre is within `half` tiles of the polyline.
    fn stamp_line(&mut self, pts: &[V2], half: f32, tile: impl Fn(Tile) -> Tile) {
        for seg in pts.windows(2) {
            let [a, b] = seg else { continue };
            let (a, b) = (*a, *b);
            let x0 = (a.x.min(b.x) - half).floor() as i32;
            let x1 = (a.x.max(b.x) + half).ceil() as i32;
            let y0 = (a.y.min(b.y) - half).floor() as i32;
            let y1 = (a.y.max(b.y) + half).ceil() as i32;
            let ab = b - a;
            let len2 = ab.dot(ab);
            for y in y0.max(0)..=y1.min(self.h - 1) {
                for x in x0.max(0)..=x1.min(self.w - 1) {
                    let p = center(x, y);
                    let t = if len2 > 1e-6 { ((p - a).dot(ab) / len2).clamp(0.0, 1.0) } else { 0.0 };
                    if p.dist(a + ab * t) <= half {
                        let nt = tile(self.tile(x, y));
                        self.set(x, y, nt);
                    }
                }
            }
        }
    }
}

/// Whether `p` is inside the polygon `ring` (even-odd rule).
fn inside(ring: &[V2], p: V2) -> bool {
    let n = ring.len();
    let mut ins = false;
    for i in 0..n {
        let (Some(a), Some(b)) = (ring.get(i), ring.get((i + 1) % n)) else { continue };
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            ins = !ins;
        }
    }
    ins
}

/// Distance from `p` to the segment `a`–`b`.
fn seg_dist(p: V2, a: V2, b: V2) -> f32 {
    let ab = b - a;
    let len2 = ab.dot(ab);
    let t = if len2 > 1e-9 { ((p - a).dot(ab) / len2).clamp(0.0, 1.0) } else { 0.0 };
    p.dist(a + ab * t)
}

/// How far outside its zone a chapter starts (metres).
pub const APPROACH_M: f32 = 150.0;

/// Joins lines that share end points and returns the longest chain.
fn longest_chain(lines: &[Vec<Dm>]) -> Option<Vec<Dm>> {
    let mut pool: Vec<Vec<Dm>> = lines.iter().filter(|l| l.len() >= 2).cloned().collect();
    let mut best: Option<Vec<Dm>> = None;
    let length = |l: &Vec<Dm>| -> f64 {
        l.windows(2)
            .map(|w| match w {
                [a, b] => (((a[0] - b[0]) as f64).powi(2) + ((a[1] - b[1]) as f64).powi(2)).sqrt(),
                _ => 0.0,
            })
            .sum()
    };
    while let Some(mut cur) = pool.pop() {
        let mut grown = true;
        while grown {
            grown = false;
            for i in 0..pool.len() {
                let Some(o) = pool.get(i) else { continue };
                let (cf, cl, of, ol) = (cur.first().copied(), cur.last().copied(), o.first().copied(), o.last().copied());
                let joined = if cl == of {
                    Some((false, false))
                } else if cl == ol {
                    Some((false, true))
                } else if cf == ol {
                    Some((true, false))
                } else if cf == of {
                    Some((true, true))
                } else {
                    None
                };
                if let Some((front, reverse)) = joined {
                    let mut o = pool.swap_remove(i);
                    if reverse {
                        o.reverse();
                    }
                    if front {
                        o.extend(cur.into_iter().skip(1));
                        cur = o;
                    } else {
                        cur.extend(o.into_iter().skip(1));
                    }
                    grown = true;
                    break;
                }
            }
        }
        if best.as_ref().is_none_or(|b| length(&cur) > length(b)) {
            best = Some(cur);
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::{Area, Road};

    /// A 40 m square pond in a 100 m plaza, with a bridge from its south shore to the middle.
    fn pond() -> City {
        let sq = |x0: i32, y0: i32, x1: i32, y1: i32| vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]];
        City {
            version: crate::city::CITY_VERSION,
            bounds: [0, 0, 1000, 1000],
            water: vec![Area { outer: sq(300, 300, 700, 700), holes: vec![sq(450, 450, 550, 550)], name: Some("Ao".into()) }],
            roads: vec![Road { pts: vec![[500, 750], [500, 520]], w: 25, kind: "footway".into(), name: Some("Cầu".into()), bridge: true }],
            ..City::default()
        }
    }

    #[test]
    fn the_shore_is_fenced_but_the_bridge_stays_open() {
        let c = pond();
        let mut m = Map::from_city(&c, 2.0);
        // Before: a tile whose centre is 0.6 m outside the west shore (x = 30 m) can be stood on.
        let west = v2(29.0 / 2.0, 40.0 / 2.0);
        assert!(m.walkable_at(west));
        assert!(m.fence_shore(&c, "Ao", 0.6, 3.0));
        assert!(!m.walkable_at(west), "the shore is fenced");
        assert!(m.walkable_at(v2(26.0 / 2.0, 40.0 / 2.0)), "the path along the shore is still there");
        // The bridge still reaches the island from the land.
        let path = m.path(v2(50.0 / 2.0, 80.0 / 2.0), v2(50.0 / 2.0, 50.0 / 2.0), 20_000);
        assert!(path.is_some(), "the island can still be reached over the bridge");
        assert!(!m.fence_shore(&c, "Nowhere", 0.6, 3.0));
    }

    fn spawn() -> (i32, i32) {
        (5, 40)
    }

    #[test]
    fn landmarks_are_where_they_should_be() {
        let m = Map::hoan_kiem();
        assert_eq!(m.tile_at(LAKE), Tile::Water);
        assert_eq!(m.tile(49, 61), Tile::Landmark, "Tháp Rùa");
        assert_eq!(m.tile(52, 42), Tile::Bridge, "Thê Húc");
        assert!(m.tile(52, 45).walkable(), "the temple island is reachable");
        assert_eq!(m.tile(10, 30), Tile::Rail);
        assert_eq!(m.tile(97, 40), Tile::River);
        assert_eq!(m.tile(97, 5), Tile::Truss, "Long Biên crosses the river");
    }

    #[test]
    fn every_exit_and_the_temple_are_reachable_from_the_west() {
        let m = Map::hoan_kiem();
        let d = m.distances(spawn());
        let reach = |p: V2| {
            let (x, y) = p.tile();
            d.get((y * W + x) as usize).copied().unwrap_or(-1) >= 0
        };
        for e in &m.exits {
            assert!(reach(e.center()), "{}", e.name);
        }
        assert!(reach(v2(52.5, 44.5)), "Đền Ngọc Sơn");
        // Almost all walkable land is one connected area.
        let walkable = (0..W * H).filter(|i| m.tile(i % W, i / W).walkable()).count();
        let reached = d.iter().filter(|v| **v >= 0).count();
        assert!(reached * 100 >= walkable * 97, "{reached} of {walkable}");
    }

    #[test]
    fn paths_go_around_the_lake_and_respect_walls() {
        let m = Map::hoan_kiem();
        let from = v2(40.5, 39.0);
        let to = v2(60.5, 72.5);
        let p = m.path(from, to, 20_000);
        assert!(p.is_some(), "no path");
        let p = p.unwrap_or_default();
        assert!(p.len() > 20);
        for q in &p {
            assert!(m.walkable_at(*q));
            assert!(lake_ell(*q, 0.0) > 1.0 || m.tile_at(*q) == Tile::Bridge, "walked on water at {q:?}");
        }
        assert!(m.path(from, LAKE, 20_000).is_none());
    }

    #[test]
    fn sight_is_blocked_by_buildings() {
        let m = Map::hoan_kiem();
        assert!(m.line_of_sight(v2(15.0, 3.0), v2(30.0, 3.0)), "along a street");
        assert!(!m.line_of_sight(v2(15.0, 3.0), v2(15.0, 30.0)) || !m.line_of_sight(v2(20.0, 3.5), v2(20.0, 11.5)));
    }

    #[test]
    fn sliding_never_enters_walls() {
        let m = Map::hoan_kiem();
        let mut p = v2(5.0, 40.0);
        let mut rng = Rng::new(3);
        for _ in 0..5000 {
            let d = V2::from_angle(rng.range_f(0.0, 6.3)) * rng.range_f(0.0, 2.0);
            p = m.slide(p, d, 0.3);
            assert!(m.circle_free(p, 0.29), "{p:?}");
        }
    }

    #[test]
    fn routes_are_measured() {
        let m = Map::hoan_kiem();
        for r in &m.routes {
            assert!(r.len > 10.0);
            let (a, d) = r.at(r.len * 0.5);
            assert!(a.x.is_finite() && (d.len() - 1.0).abs() < 1e-3);
        }
        assert_eq!(m.minimap_rgba().len(), (W * H * 4) as usize);
    }
}

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
    pub text: &'static str,
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
    fn new(pts: Vec<V2>, closed: bool) -> Route {
        let mut r = Route { pts, closed, len: 0.0 };
        r.len = r.segments().map(|(a, b)| a.dist(b)).sum();
        r
    }
    fn segments(&self) -> impl Iterator<Item = (V2, V2)> + '_ {
        let n = self.pts.len();
        let extra = if self.closed { 1 } else { 0 };
        (0..(n.saturating_sub(1) + extra)).filter_map(move |i| Some((*self.pts.get(i)?, *self.pts.get((i + 1) % n)?)))
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

pub struct Map {
    tiles: Vec<Tile>,
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
        if x < 0 || y < 0 || x >= W || y >= H {
            return Tile::Building;
        }
        self.tiles.get((y * W + x) as usize).copied().unwrap_or(Tile::Building)
    }
    pub fn tile_at(&self, p: V2) -> Tile {
        let (x, y) = p.tile();
        self.tile(x, y)
    }
    fn set(&mut self, x: i32, y: i32, t: Tile) {
        if x >= 0
            && y >= 0
            && x < W
            && y < H
            && let Some(c) = self.tiles.get_mut((y * W + x) as usize)
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
        let mut dist = vec![-1i32; (W * H) as usize];
        let mut queue = std::collections::VecDeque::new();
        if self.tile(from.0, from.1).walkable() {
            if let Some(d) = dist.get_mut((from.1 * W + from.0) as usize) {
                *d = 0;
            }
            queue.push_back(from);
        }
        while let Some((x, y)) = queue.pop_front() {
            let here = dist.get((y * W + x) as usize).copied().unwrap_or(0);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, ny) = (x + dx, y + dy);
                if !self.tile(nx, ny).walkable() {
                    continue;
                }
                if let Some(d) = dist.get_mut((ny * W + nx) as usize)
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
        let idx = |p: (i32, i32)| (p.1 * W + p.0) as usize;
        let h = |p: (i32, i32)| {
            let dx = (p.0 - goal.0).abs();
            let dy = (p.1 - goal.1).abs();
            10 * (dx + dy) - 6 * dx.min(dy)
        };
        let mut g = vec![i32::MAX; (W * H) as usize];
        let mut came = vec![u32::MAX; (W * H) as usize];
        let mut open = BinaryHeap::new();
        if let Some(c) = g.get_mut(idx(start)) {
            *c = 0;
        }
        open.push(Reverse((h(start), start.0, start.1)));
        let mut expanded = 0usize;
        while let Some(Reverse((_, x, y))) = open.pop() {
            if (x, y) == goal {
                let mut out = Vec::new();
                let mut cur = goal;
                let mut guard = 0;
                while cur != start && guard < (W * H) {
                    out.push(center(cur.0, cur.1));
                    let prev = came.get(idx(cur)).copied().unwrap_or(u32::MAX);
                    if prev == u32::MAX {
                        break;
                    }
                    cur = ((prev % W as u32) as i32, (prev / W as u32) as i32);
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
            let here = g.get(idx((x, y))).copied().unwrap_or(i32::MAX);
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let n = (x + dx, y + dy);
                if !self.tile(n.0, n.1).walkable() {
                    continue;
                }
                if dx != 0 && dy != 0 && (!self.tile(x + dx, y).walkable() || !self.tile(x, y + dy).walkable()) {
                    continue;
                }
                let cost = here.saturating_add(if dx != 0 && dy != 0 { 14 } else { 10 });
                if let Some(slot) = g.get_mut(idx(n))
                    && cost < *slot
                {
                    *slot = cost;
                    if let Some(c) = came.get_mut(idx(n)) {
                        *c = (y * W + x) as u32;
                    }
                    open.push(Reverse((cost + h(n), n.0, n.1)));
                }
            }
        }
        None
    }

    /// A random walkable tile centre in the box whose tile passes `ok`.
    pub fn random_spot(&self, rng: &mut Rng, x0: i32, y0: i32, x1: i32, y1: i32, ok: impl Fn(Tile, V2) -> bool) -> Option<V2> {
        for _ in 0..400 {
            let x = rng.range(x0.max(0) as usize, x1.min(W) as usize) as i32;
            let y = rng.range(y0.max(0) as usize, y1.min(H) as usize) as i32;
            let t = self.tile(x, y);
            let p = center(x, y);
            if t.walkable() && ok(t, p) {
                return Some(p);
            }
        }
        None
    }

    /// Colour of each tile for the minimap, as RGBA bytes (W × H).
    pub fn minimap_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity((W * H * 4) as usize);
        for y in 0..H {
            for x in 0..W {
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
        let mut m = Map { tiles: vec![Tile::Building; (W * H) as usize], exits: Vec::new(), labels: Vec::new(), routes: Vec::new(), lanterns: Vec::new() };
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

        let big = |text, x, y| Label { text, at: v2(x, y), big: true };
        let small = |text, x, y| Label { text, at: v2(x, y), big: false };
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

#[cfg(test)]
mod tests {
    use super::*;

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

//! Drawing the district and everything in it, seen from above at night.

use std::f32::consts::{PI, TAU};

use crate::content::BREEDS;
use crate::draw::Canvas;
use crate::geom::{V2, hash2, v2};
use crate::map::{ExitId, Map, RAIL_X, Tile};
use crate::raid::{Context, DogState, PickupKind, Raid, TRAIN_LEN, TState, ThiefKind};

pub struct Cam {
    pub c: V2,
    /// Pixels per tile.
    pub ppt: f32,
    pub w: f32,
    pub h: f32,
}

impl Cam {
    pub fn new(c: V2, w: f32, h: f32) -> Cam {
        Cam::zoomed(c, w, h, 1.0)
    }
    /// Zoom below 1 shows more of the district (title and menus).
    pub fn zoomed(c: V2, w: f32, h: f32, zoom: f32) -> Cam {
        let ppt = (w.min(h) / 12.5 * zoom).clamp(12.0, 54.0);
        Cam { c, ppt, w, h }
    }
    pub fn s(&self, p: V2) -> (f32, f32) {
        ((p.x - self.c.x) * self.ppt + self.w / 2.0, (p.y - self.c.y) * self.ppt + self.h / 2.0)
    }
    pub fn visible(&self, p: V2, margin: f32) -> bool {
        let (x, y) = self.s(p);
        let m = margin * self.ppt;
        x > -m && y > -m && x < self.w + m && y < self.h + m
    }
    fn tiles(&self) -> (i32, i32, i32, i32) {
        let hw = self.w / 2.0 / self.ppt + 1.0;
        let hh = self.h / 2.0 / self.ppt + 1.0;
        ((self.c.x - hw).floor() as i32, (self.c.y - hh).floor() as i32, (self.c.x + hw).ceil() as i32, (self.c.y + hh).ceil() as i32)
    }
}

const ROOFS: [u32; 8] = [0x8e4a39, 0x7a3f31, 0x9c8f78, 0x6f777e, 0x5f6a5c, 0xa39467, 0x84604a, 0x6b5a7a];

fn landmark_color(x: i32, y: i32) -> u32 {
    match (x, y) {
        (49, 61) => 0xcfc2a0,           // Tháp Rùa
        (51..=52, 46) => 0x8a3b2a,      // Đền Ngọc Sơn
        (26..=31, 49..=54) => 0x7d7a74, // Nhà thờ Lớn
        (77..=83, 64..=69) => 0xd9c9a3, // Nhà hát Lớn
        (79..=84, 5..=7) => 0x5f6b75,   // the warehouse
        _ => 0x8a8478,
    }
}

/// The colour of a tile and a key: neighbouring tiles with the same key are drawn as one block.
fn ground(t: Tile, x: i32, y: i32) -> (u32, u32) {
    match t {
        Tile::Street => (0x3a3d44, 1),
        Tile::Alley => (0x33353a, 2),
        Tile::Crowd => (0x463c3a, 3),
        Tile::Plaza => (0x4e4f55, 4),
        Tile::Park | Tile::Tree => (0x2e5638, 5),
        Tile::Water => (0x2c7a72, 6),
        Tile::River => (0x7b5638, 7),
        Tile::Bridge => (0xa82424, 8),
        Tile::Truss => (0x4a3328, 9),
        Tile::Island => (0x4d7444, 10),
        Tile::Rail => (0x45423f, 11),
        Tile::Market => (0x9b8762, 12),
        Tile::Landmark => (landmark_color(x, y), 13),
        Tile::Building => {
            // Tube houses: one tile wide, three deep.
            let k = hash2(x, y.div_euclid(3) + 7);
            (ROOFS.get((k % ROOFS.len() as u32) as usize).copied().unwrap_or(0x777777), 100 + k)
        }
    }
}

/// The district itself: ground, roofs, landmarks, open exits and names.
pub fn draw_map(cv: &mut Canvas, map: &Map, cam: &Cam, t: f32, open_exits: &[ExitId], train_warning: bool) {
    let p = cam.ppt;
    cv.rect(0.0, 0.0, cam.w, cam.h, 0x1b1d22, 1.0);
    let (x0, y0, x1, y1) = cam.tiles();
    for y in y0..=y1 {
        let mut x = x0;
        while x <= x1 {
            let tile = map.tile(x, y);
            let (c, key) = ground(tile, x, y);
            let mut run = 1;
            while x + run <= x1 && ground(map.tile(x + run, y), x + run, y).1 == key {
                run += 1;
            }
            let (sx, sy) = cam.s(v2(x as f32, y as f32));
            let gap = if key >= 100 { 1.0 } else { 0.0 };
            cv.rect(sx + gap, sy, run as f32 * p - 2.0 * gap + 0.6, p + 0.6, c, 1.0);
            x += run;
        }
    }

    // Details on visible tiles.
    for y in y0..=y1 {
        for x in x0..=x1 {
            let tile = map.tile(x, y);
            let h = hash2(x, y);
            let (sx, sy) = cam.s(v2(x as f32, y as f32));
            match tile {
                Tile::Tree => {
                    cv.circle(sx + p * 0.5, sy + p * 0.55, p * 0.58, 0x1f3d27, 1.0);
                    cv.circle(sx + p * 0.45, sy + p * 0.45, p * 0.5, 0x2f6b3c, 1.0);
                    cv.circle(sx + p * 0.38, sy + p * 0.36, p * 0.22, 0x3f8650, 1.0);
                }
                Tile::Building => {
                    // The roof's ridge runs down the middle of each tube house.
                    cv.line(sx + p * 0.5, sy, sx + p * 0.5, sy + p, (p * 0.06).max(1.0), 0x000000, 0.22);
                    if y.rem_euclid(3) == 0 && !map.tile(x, y - 1).blocks_sight() {
                        cv.rect(sx + 1.0, sy, p - 2.0, p * 0.12, 0xf5e6c8, 0.35);
                    }
                    if h.is_multiple_of(9) {
                        cv.circle(sx + p * 0.5, sy + p * 0.5, p * 0.17, 0xc8ccd0, 1.0);
                        cv.circle(sx + p * 0.5, sy + p * 0.5, p * 0.08, 0x9aa1a8, 1.0);
                    } else if h % 13 == 1 {
                        cv.rect(sx + p * 0.3, sy + p * 0.35, p * 0.4, p * 0.28, 0xb9bec4, 1.0);
                    }
                }
                Tile::Street | Tile::Alley => {
                    let by_house = map.tile(x, y - 1) == Tile::Building || map.tile(x, y + 1) == Tile::Building;
                    if by_house && h.is_multiple_of(5) {
                        // A motorbike parked on the pavement.
                        let top = map.tile(x, y - 1) == Tile::Building;
                        let yy = if top { sy + p * 0.12 } else { sy + p * 0.62 };
                        let col = [0xb91c1c, 0x1d4ed8, 0xe5e7eb, 0x111827].get((h / 5 % 4) as usize).copied().unwrap_or(0xb91c1c);
                        cv.rrect(sx + p * 0.38, yy, p * 0.24, p * 0.3, 3.0, 0x0f0f10, 1.0);
                        cv.rrect(sx + p * 0.41, yy + p * 0.04, p * 0.18, p * 0.18, 2.0, col, 1.0);
                    }
                    if by_house && h % 47 == 7 {
                        // A street lamp's warm pool of light.
                        cv.circle(sx + p * 0.5, sy + p * 0.5, p * 1.6, 0xffc861, 0.07);
                        cv.circle(sx + p * 0.5, sy + p * 0.5, p * 0.8, 0xffc861, 0.08);
                        cv.circle(sx + p * 0.5, sy + p * 0.5, p * 0.1, 0xfff1c1, 0.9);
                    }
                }
                Tile::Crowd => {
                    let stool = if h.is_multiple_of(2) { 0x2563eb } else { 0xdc2626 };
                    cv.rect(sx + p * 0.15, sy + p * 0.2, p * 0.22, p * 0.22, stool, 1.0);
                    if h.is_multiple_of(3) {
                        cv.rect(sx + p * 0.6, sy + p * 0.55, p * 0.22, p * 0.22, 0x2563eb, 1.0);
                        cv.circle(sx + p * 0.71, sy + p * 0.5, p * 0.13, 0x2b2b2b, 1.0);
                    }
                    if h % 5 == 1 {
                        cv.circle(sx + p * 0.26, sy + p * 0.18, p * 0.13, 0x3a2a20, 1.0);
                    }
                }
                Tile::Water if h.is_multiple_of(6) => {
                    let o = ((t * 0.6 + (h % 10) as f32) % 1.0) * p * 0.3;
                    cv.line(sx + p * 0.2 + o, sy + p * 0.5, sx + p * 0.55 + o, sy + p * 0.5, 1.5, 0x7cc4bb, 0.5);
                }
                Tile::River if h.is_multiple_of(5) => {
                    let o = ((t * 0.3 + (h % 10) as f32 * 0.1) % 1.0) * p;
                    cv.line(sx, sy + o, sx + p * 0.4, sy + o, 1.5, 0xa07b58, 0.6);
                }
                _ => {}
            }
        }
    }

    // The railway, with the warning lights when a train is coming.
    if (RAIL_X - cam.c.x).abs() * p < cam.w {
        let (rx, _) = cam.s(v2(RAIL_X, 0.0));
        let (top, bottom) = (0.0, cam.h);
        let mut y = ((cam.c.y - cam.h / 2.0 / p).floor()).max(0.0);
        while cam.s(v2(0.0, y)).1 < bottom {
            let (_, sy) = cam.s(v2(0.0, y));
            cv.rect(rx - p * 0.85, sy, p * 1.7, p * 0.14, 0x6b4f3a, 1.0);
            y += 0.5;
        }
        cv.line(rx - p * 0.5, top, rx - p * 0.5, bottom, 3.0, 0xb8bcc2, 1.0);
        cv.line(rx + p * 0.5, top, rx + p * 0.5, bottom, 3.0, 0xb8bcc2, 1.0);
        if train_warning && (t * 4.0) as i32 % 2 == 0 {
            cv.rect(rx - p, 0.0, p * 2.0, cam.h, 0xff3b3b, 0.18);
        }
    }

    draw_landmarks(cv, map, cam, t);

    // Open exits.
    for e in &map.exits {
        if !open_exits.contains(&e.id) || !cam.visible(e.center(), 6.0) {
            continue;
        }
        let (sx, sy) = cam.s(v2(e.x0, e.y0));
        let (ex, ey) = cam.s(v2(e.x1, e.y1));
        let pulse = 0.18 + 0.1 * (t * 3.0).sin();
        cv.rect(sx, sy, ex - sx, ey - sy, 0x34d399, pulse);
        cv.line(sx, sy, ex, sy, 2.0, 0x34d399, 0.9);
        cv.line(sx, ey, ex, ey, 2.0, 0x34d399, 0.9);
        cv.line(sx, sy, sx, ey, 2.0, 0x34d399, 0.9);
        cv.line(ex, sy, ex, ey, 2.0, 0x34d399, 0.9);
        let (cx, cy) = cam.s(e.center());
        cv.label(cx, cy, format!("THOÁT · {}", e.name), (p * 0.4).clamp(12.0, 18.0), 0xa7f3d0, 1.0, true);
    }

    for l in &map.labels {
        if !cam.visible(l.at, 4.0) {
            continue;
        }
        let (sx, sy) = cam.s(l.at);
        if l.big {
            cv.label(sx, sy, l.text, (p * 0.62).clamp(15.0, 30.0), 0xfdf6e3, 0.85, true);
        } else {
            cv.label(sx, sy, l.text, (p * 0.36).clamp(11.0, 16.0), 0xe5e7eb, 0.75, false);
        }
    }
}

fn draw_landmarks(cv: &mut Canvas, map: &Map, cam: &Cam, t: f32) {
    let p = cam.ppt;
    let wr = |cv: &mut Canvas, x0: f32, y0: f32, x1: f32, y1: f32, c: u32, a: f32| {
        let (sx, sy) = cam.s(v2(x0, y0));
        cv.rect(sx, sy, (x1 - x0) * p, (y1 - y0) * p, c, a);
    };
    // Tháp Rùa: a small three-tiered tower on its islet.
    if cam.visible(v2(49.5, 61.5), 3.0) {
        let (sx, sy) = cam.s(v2(49.5, 61.5));
        cv.circle(sx, sy, p * 0.95, 0x4d7444, 1.0);
        cv.rect(sx - p * 0.38, sy - p * 0.38, p * 0.76, p * 0.76, 0xd8cba8, 1.0);
        cv.rect(sx - p * 0.26, sy - p * 0.26, p * 0.52, p * 0.52, 0x7a5a45, 1.0);
        cv.rect(sx - p * 0.12, sy - p * 0.12, p * 0.24, p * 0.24, 0x5b3f30, 1.0);
    }
    // Đền Ngọc Sơn.
    if cam.visible(v2(52.0, 46.5), 3.0) {
        wr(cv, 50.8, 45.8, 53.2, 47.2, 0x7a2e22, 1.0);
        wr(cv, 50.95, 45.95, 53.05, 47.05, 0x9c3b2b, 1.0);
        let (a, b) = cam.s(v2(50.95, 46.5));
        let (c, _) = cam.s(v2(53.05, 46.5));
        cv.line(a, b, c, b, 2.0, 0xe8c45a, 1.0);
    }
    // Cầu Thê Húc's railings.
    if cam.visible(v2(52.0, 42.0), 4.0) {
        let (ax, ay) = cam.s(v2(51.08, 39.0));
        let (_, by) = cam.s(v2(51.08, 45.0));
        let (cx, _) = cam.s(v2(52.92, 39.0));
        cv.line(ax, ay, ax, by, 2.5, 0xe25555, 1.0);
        cv.line(cx, ay, cx, by, 2.5, 0xe25555, 1.0);
    }
    // Nhà thờ Lớn: stone nave and two towers facing the square.
    if cam.visible(v2(29.0, 52.0), 6.0) {
        wr(cv, 27.0, 49.3, 31.0, 54.0, 0x6b6862, 1.0);
        let (a, b) = cam.s(v2(29.0, 49.3));
        let (_, d) = cam.s(v2(29.0, 54.0));
        cv.line(a, b, a, d, 2.0, 0x55524d, 1.0);
        wr(cv, 26.2, 53.2, 27.9, 55.0, 0x8e8a83, 1.0);
        wr(cv, 30.1, 53.2, 31.8, 55.0, 0x8e8a83, 1.0);
        for cx in [27.05, 30.95] {
            let (sx, sy) = cam.s(v2(cx, 54.1));
            cv.line(sx - p * 0.2, sy, sx + p * 0.2, sy, 2.0, 0x2a2a2a, 1.0);
            cv.line(sx, sy - p * 0.3, sx, sy + p * 0.3, 2.0, 0x2a2a2a, 1.0);
        }
    }
    // Nhà hát Lớn: a cream building with a dome and columns facing the square.
    if cam.visible(v2(80.5, 67.0), 6.0) {
        let (sx, sy) = cam.s(v2(80.5, 67.0));
        cv.circle(sx, sy, p * 1.3, 0xbfae84, 1.0);
        cv.circle(sx, sy, p * 0.9, 0x6f8f88, 1.0);
        for i in 0..6 {
            let (cx, cy) = cam.s(v2(77.25, 64.5 + i as f32));
            cv.circle(cx, cy, p * 0.14, 0xf3ead2, 1.0);
        }
    }
    // Chợ Đồng Xuân: long roof ridges and the sign.
    if cam.visible(v2(45.0, 7.0), 12.0) {
        for i in 0..4 {
            let x = 38.0 + i as f32 * 4.5;
            let (a, b) = cam.s(v2(x, 4.1));
            let (_, d) = cam.s(v2(x, 9.9));
            cv.line(a, b, a, d, 2.0, 0x7c6a49, 1.0);
        }
        wr(cv, 40.5, 9.25, 49.5, 9.95, 0x1e3a8a, 1.0);
        let (sx, sy) = cam.s(v2(45.0, 9.6));
        cv.label(sx, sy, "CHỢ ĐỒNG XUÂN", (p * 0.38).clamp(10.0, 18.0), 0xfde047, 1.0, true);
    }
    // The gang's warehouse.
    if cam.visible(v2(82.0, 6.5), 6.0) {
        for i in 0..6 {
            let (a, b) = cam.s(v2(79.0, 5.2 + i as f32 * 0.5));
            let (c, _) = cam.s(v2(85.0, 5.2));
            cv.line(a, b, c, b, 1.5, 0x4b5560, 1.0);
        }
    }
    // Ô Quan Chưởng: the gate over the street.
    if cam.visible(v2(87.0, 19.0), 4.0) {
        wr(cv, 86.2, 17.0, 87.8, 21.0, 0x5c4033, 0.95);
        wr(cv, 86.35, 17.2, 87.65, 20.8, 0x7a5642, 0.95);
        let (sx, sy) = cam.s(v2(87.0, 19.0));
        cv.rect(sx - p * 0.5, sy - p * 0.95, p, p * 1.9, 0x2b1d16, 0.8);
    }
    // Long Biên's steel trusses.
    if cam.visible(v2(cam.c.x.clamp(89.0, 100.0), 6.0), 3.0) {
        let (ax, ay) = cam.s(v2(89.0, 5.0));
        let (bx, by) = cam.s(v2(100.0, 7.0));
        cv.line(ax, ay, bx, ay, 3.0, 0x8e5a3c, 1.0);
        cv.line(ax, by, bx, by, 3.0, 0x8e5a3c, 1.0);
        let mut x = 89.0;
        while x < 100.0 {
            let (a, b) = cam.s(v2(x, 5.0));
            let (c, d) = cam.s(v2(x + 1.0, 7.0));
            cv.line(a, b, c, d, 2.0, 0x8e5a3c, 0.9);
            cv.line(a, d, c, b, 2.0, 0x8e5a3c, 0.9);
            x += 1.0;
        }
    }
    // Lanterns over Hàng Mã.
    for (pos, red) in &map.lanterns {
        if !cam.visible(*pos, 1.0) {
            continue;
        }
        let (sx, sy) = cam.s(*pos);
        let c = if *red { 0xef4444 } else { 0xfbbf24 };
        let glow = 0.22 + 0.06 * (t * 2.0 + pos.x).sin();
        cv.circle(sx, sy, p * 0.32, c, glow);
        cv.circle(sx, sy, p * 0.13, c, 1.0);
    }
}

fn ray_poly(map: &Map, cam: &Cam, from: V2, facing: f32, half: f32, range: f32) -> Vec<(f32, f32)> {
    let mut pts = vec![cam.s(from)];
    let rays = 14;
    for i in 0..=rays {
        let a = facing - half + 2.0 * half * i as f32 / rays as f32;
        let dir = V2::from_angle(a);
        let mut d = 0.3;
        while d < range {
            if map.tile_at(from + dir * d).blocks_sight() {
                break;
            }
            d += 0.25;
        }
        pts.push(cam.s(from + dir * d.min(range)));
    }
    pts
}

pub fn dog(cv: &mut Canvas, cam: &Cam, pos: V2, facing: f32, breed: usize, wag: f32, puppy: bool) {
    let p = cam.ppt;
    let b = BREEDS.get(breed).or(BREEDS.first());
    let (coat, ear) = b.map_or((0xd9a45b, 0xa8763a), |b| (b.coat, b.ear));
    let (sx, sy) = cam.s(pos);
    let f = V2::from_angle(facing);
    let tail = V2::from_angle(facing + PI + wag.sin() * 0.7);
    cv.circle(sx + 2.0, sy + 3.0, p * 0.22, 0x000000, 0.25);
    cv.line(sx - f.x * p * 0.15, sy - f.y * p * 0.15, sx + tail.x * p * 0.38, sy + tail.y * p * 0.38, (p * 0.07).max(2.0), coat, 1.0);
    cv.circle(sx - f.x * p * 0.06, sy - f.y * p * 0.06, p * 0.2, coat, 1.0);
    let (hx, hy) = (sx + f.x * p * 0.2, sy + f.y * p * 0.2);
    let side = V2::from_angle(facing + PI / 2.0);
    cv.circle(hx + side.x * p * 0.09, hy + side.y * p * 0.09, p * 0.07, ear, 1.0);
    cv.circle(hx - side.x * p * 0.09, hy - side.y * p * 0.09, p * 0.07, ear, 1.0);
    cv.circle(hx, hy, p * 0.13, coat, 1.0);
    cv.circle(hx + f.x * p * 0.11, hy + f.y * p * 0.11, p * 0.035, 0x1a1a1a, 1.0);
    if puppy {
        cv.ring(sx + f.x * p * 0.09, sy + f.y * p * 0.09, p * 0.13, 2.0, 0xe11d48, 1.0);
    }
}

/// Everything that moves, on top of the map.
pub fn draw_raid(cv: &mut Canvas, map: &Map, raid: &Raid, cam: &Cam, t: f32) {
    let p = cam.ppt;
    let fov = raid.setup.diff.fov_deg.to_radians() / 2.0;

    // Sounds the player makes, so stealth can be learned.
    for n in &raid.noises {
        if cam.visible(n.pos, n.r + 1.0) {
            let (sx, sy) = cam.s(n.pos);
            cv.ring(sx, sy, n.r * p * (1.0 - n.t * 4.0).max(0.3), 2.0, 0xffffff, 0.22);
        }
    }

    for k in &raid.pickups {
        if k.taken || !cam.visible(k.pos, 1.0) {
            continue;
        }
        let (sx, sy) = cam.s(k.pos);
        match k.kind {
            PickupKind::Loot(_) => {
                cv.rrect(sx - p * 0.24, sy - p * 0.2, p * 0.48, p * 0.4, 3.0, 0x9a6b3f, 1.0);
                cv.line(sx - p * 0.24, sy - p * 0.05, sx + p * 0.24, sy - p * 0.05, 1.5, 0x6b4526, 1.0);
                cv.circle(sx, sy - p * 0.05, p * 0.05, 0xf2d27a, 1.0);
            }
            PickupKind::Clue => {
                let glow = 0.25 + 0.2 * (t * 4.0).sin();
                cv.circle(sx, sy, p * 0.5, 0xffd166, glow);
                cv.rrect(sx - p * 0.2, sy - p * 0.26, p * 0.4, p * 0.52, 2.0, 0xc0392b, 1.0);
                cv.line(sx - p * 0.1, sy - p * 0.1, sx + p * 0.12, sy - p * 0.1, 1.5, 0xffffff, 0.8);
                cv.line(sx - p * 0.1, sy + p * 0.04, sx + p * 0.12, sy + p * 0.04, 1.5, 0xffffff, 0.8);
            }
            PickupKind::Slipper => {
                cv.rrect(sx - p * 0.09, sy - p * 0.17, p * 0.18, p * 0.34, p * 0.08, 0xff7eb6, 1.0);
            }
        }
    }

    // Cages and the dogs in them.
    for c in &raid.cages {
        if !cam.visible(c.pos, 1.5) {
            continue;
        }
        let (sx, sy) = cam.s(c.pos);
        if !c.open {
            if let Some(d) = raid.dogs.get(c.dog) {
                dog(cv, cam, c.pos, (t * 0.7 + c.dog as f32).sin() * 0.6 + PI / 2.0, d.breed, d.wag, d.puppy);
            }
            cv.rrect(sx - p * 0.45, sy - p * 0.45, p * 0.9, p * 0.9, 3.0, 0x0f172a, 0.25);
            for i in 0..5 {
                let x = sx - p * 0.4 + i as f32 * p * 0.2;
                cv.line(x, sy - p * 0.45, x, sy + p * 0.45, 2.0, 0xa3aab4, 1.0);
            }
            cv.line(sx - p * 0.45, sy - p * 0.45, sx + p * 0.45, sy - p * 0.45, 2.5, 0xa3aab4, 1.0);
            cv.line(sx - p * 0.45, sy + p * 0.45, sx + p * 0.45, sy + p * 0.45, 2.5, 0xa3aab4, 1.0);
            cv.rect(sx + p * 0.3, sy - p * 0.05, p * 0.14, p * 0.14, 0xeab308, 1.0);
            if c.progress > 0.0 {
                cv.arc(sx, sy, p * 0.62, -PI / 2.0, -PI / 2.0 + TAU * (c.progress / crate::raid::CAGE_S).min(1.0), 4.0, 0xffd166, 1.0);
            }
            if raid.dogs.get(c.dog).is_some_and(|d| d.puppy) {
                cv.label(sx, sy - p * 0.8, crate::content::PUPPY, (p * 0.36).clamp(12.0, 16.0), 0xfecdd3, 1.0, true);
            }
        } else {
            cv.rrect(sx - p * 0.45, sy - p * 0.45, p * 0.9, p * 0.9, 3.0, 0xa3aab4, 0.25);
        }
    }

    // Traffic.
    for v in &raid.vehicles {
        if !cam.visible(v.pos, 2.0) {
            continue;
        }
        let n = V2 { x: -v.dir.y, y: v.dir.x };
        let corner = |a: f32, b: f32| cam.s(v.pos + v.dir * a + n * b);
        let (fx, fy) = cam.s(v.pos + v.dir * 0.5);
        let a0 = v.dir.angle();
        cv.wedge(fx, fy, p * 2.2, a0 - 0.35, a0 + 0.35, 0xfff7c2, 0.09);
        cv.poly(vec![corner(0.45, 0.16), corner(0.45, -0.16), corner(-0.45, -0.16), corner(-0.45, 0.16)], 0x111111, 1.0);
        cv.poly(vec![corner(0.32, 0.12), corner(0.32, -0.12), corner(-0.3, -0.12), corner(-0.3, 0.12)], v.color, 1.0);
        let (rx, ry) = cam.s(v.pos - v.dir * 0.05);
        cv.circle(rx, ry, p * 0.15, 0xf5f5f5, 1.0);
        cv.circle(fx, fy, p * 0.06, 0xfff3a0, 1.0);
    }

    // The train.
    if raid.train.active {
        let (head, tail) = (raid.train.y, raid.train.y - TRAIN_LEN);
        if cam.visible(v2(RAIL_X, (head + tail) / 2.0), TRAIN_LEN) {
            let (sx, sy) = cam.s(v2(RAIL_X - 0.8, tail));
            cv.rrect(sx, sy, p * 1.6, TRAIN_LEN * p, 4.0, 0x2d5c9e, 1.0);
            cv.rect(sx + p * 0.65, sy, p * 0.3, TRAIN_LEN * p, 0xf1f5f9, 1.0);
            let mut y = tail + 2.6;
            while y < head - 0.2 {
                let (cx, cy) = cam.s(v2(RAIL_X - 0.8, y));
                cv.line(cx, cy, cx + p * 1.6, cy, 2.0, 0x1b2b45, 1.0);
                y += 2.6;
            }
            let (hx, hy) = cam.s(v2(RAIL_X, head));
            cv.circle(hx, hy - p * 0.1, p * 0.2, 0xfde68a, 1.0);
            cv.wedge(hx, hy, p * 4.0, PI / 2.0 - 0.3, PI / 2.0 + 0.3, 0xfff7c2, 0.12);
        }
    }

    // Thieves: what they see first, then them.
    for th in &raid.thieves {
        if !cam.visible(th.pos, 9.0) || th.out() {
            continue;
        }
        let range = raid.setup.diff.vision * if th.kind.is_boss() { 1.08 } else { 1.0 };
        let (c, a) = match th.state {
            TState::Chase | TState::Windup(_) | TState::Charge(_) => (0xff5e5e, 0.17),
            TState::Investigate | TState::Search(_) => (0xffb347, 0.15),
            _ => (0xfff3b0, 0.11),
        };
        cv.poly(ray_poly(map, cam, th.pos, th.facing, fov, range), c, a);
    }
    for th in &raid.thieves {
        if !cam.visible(th.pos, 2.0) {
            continue;
        }
        let (sx, sy) = cam.s(th.pos);
        let f = V2::from_angle(th.facing);
        let side = V2::from_angle(th.facing + PI / 2.0);
        let big = if th.kind.is_boss() { 1.25 } else { 1.0 };
        let jacket = match th.kind {
            ThiefKind::Lookout => 0x5b6472,
            ThiefKind::Grunt => 0x3b4250,
            ThiefKind::TuLong => 0x7c2d12,
            ThiefKind::Bay => 0x111827,
        };
        cv.circle(sx + 2.0, sy + 4.0, p * 0.34 * big, 0x000000, 0.3);
        match th.state {
            TState::Tied => {
                cv.circle(sx, sy, p * 0.3 * big, jacket, 1.0);
                cv.line(sx - p * 0.32, sy - p * 0.1, sx + p * 0.32, sy - p * 0.1, 3.0, 0xc8a165, 1.0);
                cv.line(sx - p * 0.32, sy + p * 0.1, sx + p * 0.32, sy + p * 0.1, 3.0, 0xc8a165, 1.0);
                cv.circle(sx, sy - p * 0.05, p * 0.15, 0xe0b48a, 1.0);
                continue;
            }
            TState::Down(_) => {
                cv.circle(sx - f.x * p * 0.15, sy - f.y * p * 0.15, p * 0.28 * big, jacket, 1.0);
                cv.circle(sx + f.x * p * 0.25, sy + f.y * p * 0.25, p * 0.15, 0xe0b48a, 1.0);
                for i in 0..3 {
                    let a = t * 4.0 + i as f32 * TAU / 3.0;
                    cv.circle(sx + a.cos() * p * 0.35, sy - p * 0.45 + a.sin() * p * 0.12, p * 0.06, 0xfde047, 1.0);
                }
                continue;
            }
            _ => {}
        }
        if let TState::Windup(w) = th.state {
            if th.charging {
                let (ex, ey) = cam.s(th.pos + f * 6.0);
                cv.line(sx, sy, ex, ey, p * 0.5, 0xff3b3b, 0.25);
            } else {
                cv.ring(sx, sy, p * (0.5 + (0.5 - w).max(0.0) * 1.6), 3.0, 0xff3b3b, 0.8);
            }
        }
        // The pole with a noose.
        let pole = V2::from_angle(th.facing + 0.5);
        let (px, py) = (sx + pole.x * p * 0.75 * big, sy + pole.y * p * 0.75 * big);
        cv.line(sx + side.x * p * 0.2, sy + side.y * p * 0.2, px, py, 2.5, 0x8b6b45, 1.0);
        cv.ring(px, py, p * 0.13, 2.0, 0xd6c39a, 1.0);
        cv.circle(sx, sy, p * 0.32 * big, jacket, 1.0);
        cv.circle(sx + f.x * p * 0.04, sy + f.y * p * 0.04, p * 0.17 * big, 0xe0b48a, 1.0);
        cv.wedge(sx + f.x * p * 0.04, sy + f.y * p * 0.04, p * 0.18 * big, th.facing + PI / 2.0, th.facing + PI * 1.5, 0x1f2937, 1.0);
        if th.kind == ThiefKind::Bay {
            cv.ring(sx, sy, p * 0.33 * big, 2.5, 0xdc2626, 1.0);
        }
        if th.flash > 0.0 {
            cv.circle(sx, sy, p * 0.36 * big, 0xffffff, 0.6);
        }
        if let TState::Stunned(_) = th.state {
            for i in 0..3 {
                let a = t * 6.0 + i as f32 * TAU / 3.0;
                cv.circle(sx + a.cos() * p * 0.3, sy - p * 0.5 + a.sin() * p * 0.1, p * 0.05, 0xfde047, 1.0);
            }
        }
        if th.unaware() && th.alert > 0.05 {
            let c = if th.alert > 0.6 { 0xff5e5e } else { 0xffd166 };
            cv.arc(sx, sy - p * 0.65, p * 0.16, -PI / 2.0, -PI / 2.0 + TAU * th.alert, 3.0, c, 1.0);
        }
        if th.kind.is_boss() {
            cv.label(sx, sy - p * 0.85, th.kind.name(), (p * 0.34).clamp(11.0, 16.0), 0xfca5a5, 1.0, true);
        }
    }

    // Leashes, then the dogs following.
    let pl = &raid.player;
    let (px, py) = cam.s(pl.pos);
    for d in &raid.dogs {
        if d.state == DogState::Following {
            let (dx, dy) = cam.s(d.pos);
            cv.line(px, py, dx, dy, 1.5, 0xe11d48, 0.9);
        }
    }
    for d in &raid.dogs {
        if d.state == DogState::Following && cam.visible(d.pos, 1.0) {
            dog(cv, cam, d.pos, d.facing, d.breed, d.wag, d.puppy);
        }
    }

    // The player.
    let f = V2::from_angle(pl.facing);
    let bob = if pl.moving { pl.walk_t.sin() * p * 0.03 } else { 0.0 };
    let a = if pl.sneaking { 0.82 } else { 1.0 };
    let r = if pl.roll_t > 0.0 {
        0.24
    } else if pl.sneaking {
        0.27
    } else {
        0.3
    };
    if pl.swing_t > 0.0 {
        cv.wedge(px, py, p * 1.5, pl.facing - 1.1, pl.facing + 1.1, 0xffffff, 0.35);
    }
    cv.circle(px + 2.0, py + 4.0, p * 0.33, 0x000000, 0.3);
    cv.circle(px - f.x * p * 0.2, py - f.y * p * 0.2 + bob, p * 0.2, 0xf59e0b, a);
    cv.circle(px, py + bob, p * r, 0x3b82f6, a);
    cv.circle(px + f.x * p * 0.05, py + f.y * p * 0.05 + bob, p * 0.17, 0xf1c27d, a);
    cv.wedge(px + f.x * p * 0.05, py + f.y * p * 0.05 + bob, p * 0.18, pl.facing + PI / 2.0, pl.facing + PI * 1.5, 0x111111, a);
    if pl.hurt_t > 0.0 && (t * 20.0) as i32 % 2 == 0 {
        cv.circle(px, py, p * 0.36, 0xff3b3b, 0.5);
    }

    for s in &raid.slippers {
        let (sx, sy) = cam.s(s.pos);
        cv.circle(sx, sy, p * 0.1, 0xff7eb6, 1.0);
    }
    for fx in &raid.fx {
        if cam.visible(fx.pos, 2.0) {
            let (sx, sy) = cam.s(fx.pos);
            cv.label(sx, sy, fx.text.clone(), (p * 0.42).clamp(13.0, 20.0), fx.color, fx.t.min(1.0), true);
        }
    }
    if let Context::Cage { index, progress } = raid.context
        && let Some(c) = raid.cages.get(index)
    {
        let (sx, sy) = cam.s(c.pos);
        if progress <= 0.0 {
            cv.ring(sx, sy, p * 0.62, 2.0, 0xffd166, 0.5 + 0.3 * (t * 6.0).sin());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raid::{PlayerStats, RaidSetup};

    #[test]
    fn a_frame_draws_and_stays_on_budget() {
        let map = Map::hoan_kiem();
        let stats = PlayerStats { max_hp: 100.0, speed: 4.0, bag: 6, leash: 2, slippers: 3 };
        let raid = Raid::new(&map, RaidSetup::free(6, stats, 3));
        for (w, h) in [(390.0, 844.0), (844.0, 390.0), (1440.0, 900.0)] {
            let mut cv = Canvas::default();
            let cam = Cam::new(raid.player.pos, w, h);
            draw_map(&mut cv, &map, &cam, 1.0, &raid.setup.exits, true);
            draw_raid(&mut cv, &map, &raid, &cam, 1.0);
            assert!(cv.list.len() > 50);
            assert!(cv.list.len() < 4000, "{} shapes at {w}x{h}", cv.list.len());
        }
    }
}

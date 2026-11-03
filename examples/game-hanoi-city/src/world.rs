//! The district in 3D, built from the OpenStreetMap city file: tube houses extruded to their real
//! height with facades and flat roofs (water tanks on top), asphalt roads, paved pavements, the
//! lake, parks, trees, the railway, bridges, lanterns over Hàng Mã and real shop signs.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use hanoi_rescue::city::{City, Dm};
use hanoi_rescue::geom::hash2;

use hanoi_rescue::geom::V2;
use hanoi_rescue::map::{Map, Tile};

use crate::facade;
use crate::hq::{Site, TREE_SITE};

/// Metres from the city's top-left corner: x east, z south.
pub fn to_world(c: &City, p: &Dm) -> Vec2 {
    Vec2::new((p[0] - c.bounds[0]) as f32 / 10.0, (p[1] - c.bounds[1]) as f32 / 10.0)
}

/// Builds meshes out of quads and triangles, one per material.
#[derive(Default)]
struct Builder {
    pos: Vec<[f32; 3]>,
    nrm: Vec<[f32; 3]>,
    tan: Vec<[f32; 4]>,
    uv: Vec<[f32; 2]>,
    idx: Vec<u32>,
}

/// The tangent (direction of growing u, and the sign that gives growing v) of a flat face, for
/// normal maps.
fn tangent(p: [Vec3; 4], uv: [Vec2; 4], n: Vec3) -> [f32; 4] {
    let du = uv[3] - uv[0];
    let dv = uv[1] - uv[0];
    let (e1, e2) = (p[3] - p[0], p[1] - p[0]);
    let det = du.x * dv.y - du.y * dv.x;
    if det.abs() < 1e-9 {
        return [1.0, 0.0, 0.0, 1.0];
    }
    let t = ((e1 * dv.y - e2 * du.y) / det).normalize_or_zero();
    let b = ((e2 * du.x - e1 * dv.x) / det).normalize_or_zero();
    let w = if n.cross(t).dot(b) < 0.0 { -1.0 } else { 1.0 };
    [t.x, t.y, t.z, w]
}

impl Builder {
    fn quad(&mut self, p: [Vec3; 4], uv: [Vec2; 4]) {
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let t = tangent(p, uv, n);
        let base = self.pos.len() as u32;
        for i in 0..4 {
            self.pos.push(p[i].to_array());
            self.nrm.push(n.to_array());
            self.tan.push(t);
            self.uv.push(uv[i].to_array());
        }
        self.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    fn tris(&mut self, pts: &[Vec3], tris: &[[usize; 3]], uv_scale: f32) {
        let base = self.pos.len() as u32;
        for p in pts {
            self.pos.push(p.to_array());
            self.nrm.push([0.0, 1.0, 0.0]);
            self.tan.push([1.0, 0.0, 0.0, -1.0]);
            self.uv.push([p.x * uv_scale, p.z * uv_scale]);
        }
        for t in tris {
            self.idx.extend_from_slice(&[base + t[0] as u32, base + t[2] as u32, base + t[1] as u32]);
        }
    }
    fn mesh(self) -> Option<Mesh> {
        if self.idx.is_empty() {
            return None;
        }
        Some(
            Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.pos)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.nrm)
                .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, self.tan)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uv)
                .with_inserted_indices(Indices::U32(self.idx)),
        )
    }
}

fn signed_area(r: &[Vec2]) -> f32 {
    let n = r.len();
    (0..n).map(|i| r[i].perp_dot(r[(i + 1) % n])).sum::<f32>() / 2.0
}

/// Ear clipping for a simple polygon; returns index triples into `r`.
pub fn earcut(r: &[Vec2]) -> Vec<[usize; 3]> {
    let mut idx: Vec<usize> = (0..r.len()).collect();
    if idx.len() >= 2 && r[idx[0]].distance(r[idx[idx.len() - 1]]) < 1e-4 {
        idx.pop();
    }
    if signed_area(&idx.iter().map(|i| r[*i]).collect::<Vec<_>>()) < 0.0 {
        idx.reverse();
    }
    let mut out = Vec::new();
    let mut guard = 0;
    while idx.len() > 3 && guard < 100_000 {
        guard += 1;
        let n = idx.len();
        let mut clipped = false;
        for i in 0..n {
            let (a, b, c) = (r[idx[(i + n - 1) % n]], r[idx[i]], r[idx[(i + 1) % n]]);
            if (b - a).perp_dot(c - b) <= 1e-7 {
                continue;
            }
            let inside = idx.iter().any(|&j| {
                let p = r[j];
                if p == a || p == b || p == c {
                    return false;
                }
                let d1 = (b - a).perp_dot(p - a);
                let d2 = (c - b).perp_dot(p - b);
                let d3 = (a - c).perp_dot(p - c);
                d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0
            });
            if !inside {
                out.push([idx[(i + n - 1) % n], idx[i], idx[(i + 1) % n]]);
                idx.remove(i);
                clipped = true;
                break;
            }
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    }
    out
}

#[derive(Resource, Clone)]
pub struct Mats {
    pub walls_up: Vec<Handle<StandardMaterial>>,
    pub walls_ground: Vec<Handle<StandardMaterial>>,
    pub roof: Handle<StandardMaterial>,
    pub roof_tiles: Handle<StandardMaterial>,
    pub asphalt: Handle<StandardMaterial>,
    pub paving: Handle<StandardMaterial>,
    /// Footways and the lakeside promenade.
    pub path_paving: Handle<StandardMaterial>,
    pub grass: Handle<StandardMaterial>,
    pub water: Handle<StandardMaterial>,
    pub red_bridge: Handle<StandardMaterial>,
    pub steel: Handle<StandardMaterial>,
    pub rail_bed: Handle<StandardMaterial>,
    pub signs: Handle<StandardMaterial>,
    pub tank: Handle<StandardMaterial>,
    pub bark: Handle<StandardMaterial>,
    pub leaves: Handle<StandardMaterial>,
    pub lantern_red: Handle<StandardMaterial>,
    pub lantern_yellow: Handle<StandardMaterial>,
}

impl Mats {
    /// A ground material by the name `hq` uses for it.
    pub fn by_name(&self, name: &str) -> Option<Handle<StandardMaterial>> {
        Some(
            match name {
                "paving" => &self.paving,
                "path_paving" => &self.path_paving,
                "asphalt" => &self.asphalt,
                "grass" => &self.grass,
                "roof" => &self.roof,
                "roof_tiles" => &self.roof_tiles,
                "water" => &self.water,
                _ => return None,
            }
            .clone(),
        )
    }
}

/// A stand-in for a piece of delivered art (see `hq`): hidden once the real one is placed.
#[derive(Component, Clone, Copy)]
pub struct StandIn(pub &'static str);

/// Landmarks the art list covers, by their name in OpenStreetMap.
const LANDMARK_BUILDINGS: [(&str, &str); 6] = [
    ("Tháp Rùa", "landmarks/thap_rua.glb"),
    ("Đền Ngọc Sơn", "landmarks/den_ngoc_son.glb"),
    ("Nhà thờ Lớn", "landmarks/nha_tho_lon.glb"),
    ("Nhà hát Lớn", "landmarks/nha_hat_lon.glb"),
    ("Ô Quan Chưởng", "landmarks/o_quan_chuong.glb"),
    ("Chợ Đồng Xuân", "landmarks/cho_dong_xuan.glb"),
];
const THE_HUC: (&str, &str) = ("Cầu Thê Húc", "landmarks/cau_the_huc.glb");
/// Landmarks whose delivered model covers more than their main building: everything standing within
/// this many metres of it (Đình Trấn Ba beside the temple, the tower's parts) is part of its stand-in.
const LANDMARK_GROUNDS: [(&str, f32); 2] = [("Đền Ngọc Sơn", 30.0), ("Tháp Rùa", 6.0)];
const GATE: &str = "landmarks/thap_but_dai_nghien.glb";

fn repeat(assets: &AssetServer, path: &'static str) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(|s: &mut ImageLoaderSettings| {
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..ImageSamplerDescriptor::linear()
            });
        })
        .load(path)
}

/// The lake's ripples: its normal map drifts slowly (seen once a `nuoc_ho` normal map is delivered).
pub fn ripple(time: Res<Time>, m: Option<Res<Mats>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let Some(m) = m else { return };
    let Some(mut water) = mats.get_mut(&m.water) else { return };
    if water.normal_map_texture.is_none() {
        return;
    }
    let scale = water.uv_transform.matrix2.x_axis.x;
    let t = time.elapsed_secs();
    water.uv_transform = bevy::math::Affine2::from_scale_angle_translation(Vec2::splat(scale), 0.0, Vec2::new(t * 0.012, t * 0.007));
}

/// Hidden when the camera is farther than `limit` from `at` (cheaper than drawing into fog).
#[derive(Component, Clone)]
pub struct Far {
    pub at: Vec3,
    pub limit: f32,
}

/// Shows and hides far things a few times a second.
pub fn cull_far(time: Res<Time>, mut next: Local<f32>, cam: Query<&GlobalTransform, With<Camera3d>>, mut q: Query<(&Far, &mut Visibility)>) {
    *next -= time.delta_secs();
    if *next > 0.0 {
        return;
    }
    *next = 0.4;
    let Ok(cam) = cam.single() else { return };
    let c = cam.translation();
    let flat = Vec3::new(c.x, 0.0, c.z);
    for (f, mut v) in &mut q {
        let show = f.at.distance(flat) < f.limit;
        let want = if show { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

pub fn materials(assets: &AssetServer, mats: &mut Assets<StandardMaterial>) -> Mats {
    let tex = |name: &'static str, rough: f32, mats: &mut Assets<StandardMaterial>| {
        mats.add(StandardMaterial { base_color_texture: Some(repeat(assets, name)), perceptual_roughness: rough, ..default() })
    };
    let mut walls_up = Vec::new();
    let mut walls_ground = Vec::new();
    const UP: [&str; 6] = ["tex/wall0_up.jpg", "tex/wall1_up.jpg", "tex/wall2_up.jpg", "tex/wall3_up.jpg", "tex/wall4_up.jpg", "tex/wall5_up.jpg"];
    const UP_GLOW: [&str; 6] =
        ["tex/wall0_up_glow.jpg", "tex/wall1_up_glow.jpg", "tex/wall2_up_glow.jpg", "tex/wall3_up_glow.jpg", "tex/wall4_up_glow.jpg", "tex/wall5_up_glow.jpg"];
    const GR: [&str; 6] =
        ["tex/wall0_ground.jpg", "tex/wall1_ground.jpg", "tex/wall2_ground.jpg", "tex/wall3_ground.jpg", "tex/wall4_ground.jpg", "tex/wall5_ground.jpg"];
    const GR_GLOW: [&str; 6] = [
        "tex/wall0_ground_glow.jpg",
        "tex/wall1_ground_glow.jpg",
        "tex/wall2_ground_glow.jpg",
        "tex/wall3_ground_glow.jpg",
        "tex/wall4_ground_glow.jpg",
        "tex/wall5_ground_glow.jpg",
    ];
    for i in 0..6 {
        walls_up.push(mats.add(StandardMaterial {
            base_color_texture: Some(repeat(assets, UP[i])),
            emissive_texture: Some(repeat(assets, UP_GLOW[i])),
            emissive: crate::render::glow(1.6, 1.3, 1.0),
            perceptual_roughness: 0.9,
            ..default()
        }));
        walls_ground.push(mats.add(StandardMaterial {
            base_color_texture: Some(repeat(assets, GR[i])),
            emissive_texture: Some(repeat(assets, GR_GLOW[i])),
            emissive: crate::render::glow(2.0, 1.6, 1.1),
            perceptual_roughness: 0.85,
            ..default()
        }));
    }
    let signs_tex = assets.load("tex/signs.jpg");
    Mats {
        walls_up,
        walls_ground,
        roof: tex("tex/concrete.jpg", 0.95, mats),
        roof_tiles: tex("tex/roof_tiles.jpg", 0.8, mats),
        asphalt: tex("tex/asphalt.jpg", 0.9, mats),
        paving: tex("tex/paving.jpg", 0.85, mats),
        path_paving: mats.add(StandardMaterial {
            base_color: Color::srgb(0.86, 0.8, 0.74),
            base_color_texture: Some(repeat(assets, "tex/paving.jpg")),
            perceptual_roughness: 0.8,
            ..default()
        }),
        grass: tex("tex/grass.jpg", 0.95, mats),
        water: mats.add(StandardMaterial { base_color: Color::srgb(0.16, 0.42, 0.38), perceptual_roughness: 0.08, reflectance: 0.6, ..default() }),
        red_bridge: mats.add(StandardMaterial { base_color: Color::srgb(0.72, 0.1, 0.1), perceptual_roughness: 0.6, ..default() }),
        steel: mats.add(StandardMaterial { base_color: Color::srgb(0.42, 0.28, 0.2), metallic: 0.5, perceptual_roughness: 0.6, ..default() }),
        rail_bed: mats.add(StandardMaterial { base_color: Color::srgb(0.3, 0.27, 0.24), perceptual_roughness: 1.0, ..default() }),
        signs: mats.add(StandardMaterial {
            base_color_texture: Some(signs_tex.clone()),
            emissive_texture: Some(signs_tex),
            emissive: crate::render::glow(0.9, 0.9, 0.9),
            unlit: false,
            ..default()
        }),
        tank: mats.add(StandardMaterial { base_color: Color::srgb(0.82, 0.84, 0.86), metallic: 0.8, perceptual_roughness: 0.3, ..default() }),
        bark: tex("tex/bark.jpg", 1.0, mats),
        leaves: mats.add(StandardMaterial { base_color: Color::srgb(0.18, 0.38, 0.17), perceptual_roughness: 0.9, ..default() }),
        lantern_red: mats.add(StandardMaterial { base_color: Color::srgb(0.9, 0.1, 0.1), emissive: crate::render::glow(6.0, 0.6, 0.3), ..default() }),
        lantern_yellow: mats.add(StandardMaterial { base_color: Color::srgb(0.95, 0.75, 0.2), emissive: crate::render::glow(6.0, 4.0, 0.8), ..default() }),
    }
}

/// How far from the lake's centre the town is drawn (metres): beyond it, fog.
pub const DRAW_RADIUS: f32 = 1000.0;
const CHUNK: f32 = 160.0;

/// What the city hands on to the art streamers: where delivered models go, and the street fronts.
pub struct Built {
    pub sites: Vec<Site>,
    pub facades: facade::Facades,
    /// A loop round Hồ Gươm a few metres out from the shore, where people stroll.
    pub promenade: Vec<Vec2>,
    /// What gives light at night: street lamps and the Hàng Mã lanterns.
    pub glows: Vec<crate::night::Glow>,
}

/// Mesh kinds at or above this are the street-facing part of a wall kind (kind + FRONT).
const FRONT: u8 = 20;

/// Whether a building face (ring order `a` to `b`, ring counter-clockwise) looks onto a street.
fn street_facing(map: &Map, a: Vec2, b: Vec2) -> bool {
    let d = (b - a).normalize_or_zero();
    let out = Vec2::new(d.y, -d.x);
    let p = (a + b) / 2.0 + out * 2.5;
    let t = map.tile_at(V2 { x: p.x / map.tile_m, y: p.y / map.tile_m });
    matches!(t, Tile::Street | Tile::Alley | Tile::Plaza | Tile::Crowd | Tile::Rail)
}

/// Spawns the whole district. `center` is where the lake is (metres).
pub fn spawn_city(commands: &mut Commands, meshes: &mut Assets<Mesh>, m: &Mats, c: &City, map: &Map, center: Vec2, signs: &[(String, Dm)]) -> Built {
    let mut sites = Vec::new();
    let mut glows = Vec::new();
    let mut blocks: HashMap<(i32, i32), facade::Block> = HashMap::new();
    // Stand-ins for delivered landmarks get meshes of their own, so they can be hidden.
    let mut own: Own = HashMap::new();
    let near = |p: Vec2| p.distance(center) < DRAW_RADIUS;
    let chunk_of = |p: Vec2| ((p.x / CHUNK).floor() as i32, (p.y / CHUNK).floor() as i32);
    let mut chunks: Chunks = HashMap::new();
    // Kinds: 0 wall up, 1 wall ground, 2 roof, 3 roof tiles, 4 asphalt, 5 grass, 6 water, 7 red bridge, 8 steel, 9 rail bed.
    let mut tanks: Vec<Vec3> = Vec::new();

    // Ground: paving everywhere, in chunks so it's culled.
    let (w, h) = ((c.bounds[2] - c.bounds[0]) as f32 / 10.0, (c.bounds[3] - c.bounds[1]) as f32 / 10.0);
    let mut x = 0.0;
    while x < w {
        let mut z = 0.0;
        while z < h {
            let p = Vec2::new(x + CHUNK / 2.0, z + CHUNK / 2.0);
            if near(p) {
                let (cx, cz) = chunk_of(p);
                let b = chunks.entry((cx, cz, 10, 0)).or_default();
                let s = 0.25;
                b.quad(
                    [Vec3::new(x, 0.0, z), Vec3::new(x, 0.0, z + CHUNK), Vec3::new(x + CHUNK, 0.0, z + CHUNK), Vec3::new(x + CHUNK, 0.0, z)],
                    [
                        Vec2::new(x * s, z * s),
                        Vec2::new(x * s, (z + CHUNK) * s),
                        Vec2::new((x + CHUNK) * s, (z + CHUNK) * s),
                        Vec2::new((x + CHUNK) * s, z * s),
                    ],
                );
            }
            z += CHUNK;
        }
        x += CHUNK;
    }

    // Parks and water, flat.
    for (areas, kind, y) in [(&c.parks, 5u8, 0.03f32), (&c.water, 6u8, 0.05f32)] {
        for a in areas.iter() {
            let ring: Vec<Vec2> = a.outer.iter().map(|p| to_world(c, p)).collect();
            if ring.len() < 3 || !ring.iter().any(|p| near(*p)) {
                continue;
            }
            let tris = earcut(&ring);
            let pts: Vec<Vec3> = ring.iter().map(|p| Vec3::new(p.x, y, p.y)).collect();
            let (cx, cz) = chunk_of(ring[0]);
            chunks.entry((cx, cz, kind, 0)).or_default().tris(&pts, &tris, 0.2);
            for hole in &a.holes {
                let ring: Vec<Vec2> = hole.iter().map(|p| to_world(c, p)).collect();
                let tris = earcut(&ring);
                let pts: Vec<Vec3> = ring.iter().map(|p| Vec3::new(p.x, y + 0.03, p.y)).collect();
                chunks.entry((cx, cz, 5, 0)).or_default().tris(&pts, &tris, 0.2);
            }
        }
    }

    // Roads: asphalt strips (bridges red, or steel for Long Biên), above the paving.
    for r in &c.roads {
        let footway = matches!(r.kind.as_str(), "footway" | "path" | "steps" | "cycleway" | "pedestrian") && !r.bridge;
        let pts: Vec<Vec2> = r.pts.iter().map(|p| to_world(c, p)).collect();
        if !pts.iter().any(|p| near(*p)) {
            continue;
        }
        let half = r.w as f32 / 20.0;
        let (kind, y) = if r.bridge {
            if r.name.as_deref() == Some("Cầu Long Biên") { (8u8, 4.0f32) } else { (7u8, 0.6f32) }
        } else if footway {
            (11u8, 0.035f32)
        } else {
            (4u8, 0.04f32)
        };
        let the_huc = r.name.as_deref() == Some(THE_HUC.0);
        if the_huc && let (Some(a), Some(b)) = (pts.first(), pts.last()) {
            // The model's north end (−Z) points along the bridge to its northern end.
            let (south, north) = if a.y > b.y { (*a, *b) } else { (*b, *a) };
            let d = north - south;
            let mid = (south + north) / 2.0;
            sites.push(Site { file: THE_HUC.1, kind: "landmark", at: Vec3::new(mid.x, 0.0, mid.y), yaw: (-d.x).atan2(-d.y) });
            // Tháp Bút, Đài Nghiên and the gate stand where the bridge meets the shore: the end
            // farther from the temple island. Their model's −Z points along the bridge to the island.
            let temple = c.buildings.iter().find(|b| b.name.as_deref() == Some("Đền Ngọc Sơn")).and_then(|b| b.outer.first()).map(|p| to_world(c, p));
            if let Some(t) = temple {
                let (shore, island) = if a.distance(t) > b.distance(t) { (*a, *b) } else { (*b, *a) };
                let to_island = island - shore;
                sites.push(Site { file: GATE, kind: "landmark", at: Vec3::new(shore.x, 0.0, shore.y), yaw: (-to_island.x).atan2(-to_island.y) });
            }
        }
        let mut along = 0.0;
        for seg in pts.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let d = (b - a).normalize_or_zero();
            let n = Vec2::new(-d.y, d.x) * half;
            let len = a.distance(b);
            let (cx, cz) = chunk_of(a);
            let s = 0.25;
            let w = 2.0 * half * s;
            let target = if the_huc { own.entry(THE_HUC.1).or_default().entry(kind).or_default() } else { chunks.entry((cx, cz, kind, 0)).or_default() };
            target.quad(
                [
                    Vec3::new(a.x + n.x, y, a.y + n.y),
                    Vec3::new(b.x + n.x, y, b.y + n.y),
                    Vec3::new(b.x - n.x, y, b.y - n.y),
                    Vec3::new(a.x - n.x, y, a.y - n.y),
                ],
                [Vec2::new(0.0, along * s), Vec2::new(0.0, (along + len) * s), Vec2::new(w, (along + len) * s), Vec2::new(w, along * s)],
            );
            along += len;
        }
    }
    for rail in &c.rails {
        let pts: Vec<Vec2> = rail.iter().map(|p| to_world(c, p)).collect();
        if !pts.iter().any(|p| near(*p)) {
            continue;
        }
        for seg in pts.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let d = (b - a).normalize_or_zero();
            let n = Vec2::new(-d.y, d.x) * 1.5;
            let (cx, cz) = chunk_of(a);
            let y = 0.06;
            chunks.entry((cx, cz, 9, 0)).or_default().quad(
                [
                    Vec3::new(a.x + n.x, y, a.y + n.y),
                    Vec3::new(b.x + n.x, y, b.y + n.y),
                    Vec3::new(b.x - n.x, y, b.y - n.y),
                    Vec3::new(a.x - n.x, y, a.y - n.y),
                ],
                [Vec2::ZERO, Vec2::Y, Vec2::ONE, Vec2::X],
            );
            for side in [-0.72f32, 0.72] {
                let o = Vec2::new(-d.y, d.x) * side;
                let t = Vec2::new(-d.y, d.x) * 0.06;
                chunks.entry((cx, cz, 8, 0)).or_default().quad(
                    [
                        Vec3::new(a.x + o.x + t.x, 0.2, a.y + o.y + t.y),
                        Vec3::new(b.x + o.x + t.x, 0.2, b.y + o.y + t.y),
                        Vec3::new(b.x + o.x - t.x, 0.2, b.y + o.y - t.y),
                        Vec3::new(a.x + o.x - t.x, 0.2, a.y + o.y - t.y),
                    ],
                    [Vec2::ZERO, Vec2::Y, Vec2::ONE, Vec2::X],
                );
            }
        }
    }

    // Bridge decks: OpenStreetMap also outlines some bridges as untagged areas, which would
    // otherwise stand up as a low building over the bridge (Thê Húc's did).
    let decks: Vec<Vec2> = c
        .roads
        .iter()
        .filter(|r| r.bridge)
        .flat_map(|r| {
            let pts: Vec<Vec2> = r.pts.iter().map(|p| to_world(c, p)).collect();
            pts.windows(2).map(|w| (w[0] + w[1]) / 2.0).collect::<Vec<_>>()
        })
        .collect();
    // The outlines themselves: untagged, unnamed areas lying over a bridge's deck. Everything
    // unnamed inside one (piers, railings, the deck's halves) is part of the bridge too.
    let outlines: Vec<Vec<Vec2>> = c
        .buildings
        .iter()
        .filter(|b| b.kind.is_empty() && b.name.is_none())
        .map(|b| b.outer.iter().map(|p| to_world(c, p)).collect::<Vec<Vec2>>())
        .filter(|ring| ring.len() >= 3 && decks.iter().any(|d| contains(ring, *d)))
        .collect();

    let grounds: Vec<(Vec2, f32, &'static str)> = LANDMARK_GROUNDS
        .iter()
        .filter_map(|(name, r)| {
            let b = c.buildings.iter().find(|b| b.name.as_deref() == Some(*name))?;
            let ring: Vec<Vec2> = b.outer.iter().map(|p| to_world(c, p)).collect();
            let file = LANDMARK_BUILDINGS.iter().find(|(n, _)| n == name)?.1;
            (!ring.is_empty()).then(|| (ring.iter().copied().sum::<Vec2>() / ring.len() as f32, *r, file))
        })
        .collect();

    // Buildings: two bands of wall (shopfront, then floors of windows) and a roof.
    for (bi, b) in c.buildings.iter().enumerate() {
        let mut ring: Vec<Vec2> = b.outer.iter().map(|p| to_world(c, p)).collect();
        if ring.len() >= 2 && ring[0].distance(ring[ring.len() - 1]) < 1e-3 {
            ring.pop();
        }
        if ring.len() < 3 || !near(ring[0]) {
            continue;
        }
        if signed_area(&ring) < 0.0 {
            ring.reverse();
        }
        let centre = ring.iter().copied().sum::<Vec2>() / ring.len() as f32;
        let on_bridge = b.name.is_none() && outlines.iter().any(|o| contains(o, centre));
        // Untagged parts of a bridge aren't buildings; a tagged structure at its end is a gate,
        // standing in for the delivered Tháp Bút and Đài Nghiên.
        if on_bridge && b.kind.is_empty() {
            continue;
        }
        let height = (b.h as f32 / 10.0).clamp(3.0, 120.0);
        let style = (hash2(bi as i32, 7) % 6) as u8;
        let (cx, cz) = chunk_of(ring[0]);
        let landmark = b.name.as_deref().and_then(|n| LANDMARK_BUILDINGS.iter().find(|(name, _)| *name == n)).map(|(_, f)| *f);
        let landmark = landmark.or_else(|| grounds.iter().find(|(at, r, _)| at.distance(centre) < *r).map(|(_, _, f)| *f));
        let landmark = if on_bridge { Some(GATE) } else { landmark };
        if let Some(f) = landmark {
            let (lo, hi) = ring.iter().fold((Vec2::splat(f32::MAX), Vec2::splat(f32::MIN)), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
            let mid = (lo + hi) / 2.0;
            if !sites.iter().any(|s: &Site| s.file == f) {
                sites.push(Site { file: f, kind: "landmark", at: Vec3::new(mid.x, 0.0, mid.y), yaw: 0.0 });
            }
        }

        let ground = 3.6f32.min(height);
        let n = ring.len();
        let french = ring[0].distance(center) < 380.0;
        for i in 0..n {
            let (a, bb) = (ring[i], ring[(i + 1) % n]);
            let len = a.distance(bb);
            if len < 0.05 {
                continue;
            }
            // A face onto a street can take delivered front modules: the part they cover goes into
            // the block's own "flat front" mesh, hidden while the modules are up.
            let (slots, covered) = if landmark.is_none() && street_facing(map, a, bb) {
                facade::slots_for(a, bb, height, french, hash2(bi as i32, i as i32))
            } else {
                (Vec::new(), 0.0)
            };
            let front = !slots.is_empty();
            if front {
                blocks
                    .entry((cx, cz))
                    .or_insert_with(|| facade::Block::new(Vec3::new((cx as f32 + 0.5) * CHUNK, 0.0, (cz as f32 + 0.5) * CHUNK)))
                    .slots
                    .extend(slots);
            }
            let u = (len / 3.0).max(1.0).round();
            let wall = |y0: f32, y1: f32| [Vec3::new(a.x, y0, a.y), Vec3::new(a.x, y1, a.y), Vec3::new(bb.x, y1, bb.y), Vec3::new(bb.x, y0, bb.y)];
            let to = |k: u8| if front { k + FRONT } else { k };
            pick(&mut chunks, &mut own, landmark, (cx, cz, to(1), style))
                .quad(wall(0.0, ground), [Vec2::new(0.0, 1.0), Vec2::new(0.0, 0.0), Vec2::new(u, 0.0), Vec2::new(u, 1.0)]);
            if height > ground + 0.5 {
                // v counts storeys down from the roof line, so a split face keeps its windows.
                let v = |y: f32| (height - y) / 3.3;
                let split = if front { covered.clamp(ground, height) } else { ground };
                if split > ground + 0.01 {
                    pick(&mut chunks, &mut own, landmark, (cx, cz, to(0), style))
                        .quad(wall(ground, split), [Vec2::new(0.0, v(ground)), Vec2::new(0.0, v(split)), Vec2::new(u, v(split)), Vec2::new(u, v(ground))]);
                }
                if height > split + 0.01 {
                    pick(&mut chunks, &mut own, landmark, (cx, cz, 0, style))
                        .quad(wall(split, height), [Vec2::new(0.0, v(split)), Vec2::new(0.0, 0.0), Vec2::new(u, 0.0), Vec2::new(u, v(split))]);
                }
            }
        }
        let tris = earcut(&ring);
        let pts: Vec<Vec3> = ring.iter().map(|p| Vec3::new(p.x, height, p.y)).collect();
        let tiled = b.kind == "temple" || b.kind == "church" || hash2(bi as i32, 3).is_multiple_of(9);
        pick(&mut chunks, &mut own, landmark, (cx, cz, if tiled { 3 } else { 2 }, 0)).tris(&pts, &tris, 0.25);
        // Inox water tanks on the roofs of homes near the middle.
        if !tiled && height < 25.0 && hash2(bi as i32, 11).is_multiple_of(4) && ring[0].distance(center) < 700.0 {
            let mid = ring.iter().copied().sum::<Vec2>() / n as f32;
            tanks.push(Vec3::new(mid.x, height, mid.y));
        }
    }

    let material = |kind: u8, style: u8| -> Handle<StandardMaterial> {
        match kind {
            0 => m.walls_up[style as usize % m.walls_up.len()].clone(),
            1 => m.walls_ground[style as usize % m.walls_ground.len()].clone(),
            2 => m.roof.clone(),
            3 => m.roof_tiles.clone(),
            4 => m.asphalt.clone(),
            5 => m.grass.clone(),
            6 => m.water.clone(),
            7 => m.red_bridge.clone(),
            8 => m.steel.clone(),
            9 => m.rail_bed.clone(),
            11 => m.path_paving.clone(),
            _ => m.paving.clone(),
        }
    };
    for (file, parts) in own {
        for (key, b) in parts {
            if let Some(mesh) = b.mesh() {
                commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material(key / 8, key % 8)), StandIn(file)));
            }
        }
    }
    for ((cx, cz, kind, style), b) in chunks {
        let b_center = Vec3::new((cx as f32 + 0.5) * CHUNK, 0.0, (cz as f32 + 0.5) * CHUNK);
        let front = kind >= FRONT;
        let mat = material(if front { kind - FRONT } else { kind }, style);
        if let Some(mesh) = b.mesh() {
            // Chunks beyond the fog aren't drawn at all (see cull_far).
            let far = Far { at: b_center, limit: if crate::render::HIGH { 1100.0 } else { 450.0 } + CHUNK };
            if front {
                // Flat fronts are shown and hidden by the facade streamer, not by distance.
                let e = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(mat))).id();
                if let Some(block) = blocks.get_mut(&(cx, cz)) {
                    block.flat.push(e);
                }
            } else {
                commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(mat), far));
            }
        }
    }
    street_furniture(c, map, &near, &chunk_of, &mut blocks, &facade::street_atlas().0, &decks, &mut glows);
    let facades = facade::Facades { blocks: blocks.into_values().collect() };

    // Water tanks.
    let tank_mesh = meshes.add(Cylinder::new(0.45, 1.3));
    for t in tanks {
        commands.spawn((
            Mesh3d(tank_mesh.clone()),
            MeshMaterial3d(m.tank.clone()),
            Transform::from_translation(t + Vec3::new(0.0, 0.65, 0.0)).with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
            Far { at: t, limit: 160.0 },
        ));
    }

    // Trees.
    let trunk = meshes.add(Cylinder::new(0.18, 3.0));
    let crown = meshes.add(Sphere::new(1.0).mesh().ico(1).unwrap_or_else(|_| Sphere::new(1.0).mesh().uv(8, 6)));
    for t in &c.trees {
        let p = to_world(c, t);
        if !near(p) {
            continue;
        }
        let k = (hash2(t[0], t[1]) % 100) as f32 / 100.0;
        let s = 1.6 + k * 1.4;
        let far = Far { at: Vec3::new(p.x, 0.0, p.y), limit: 260.0 };
        sites.push(Site { file: TREE_SITE, kind: "tree", at: Vec3::new(p.x, 0.0, p.y), yaw: k * std::f32::consts::TAU });
        let far = (far, StandIn(TREE_SITE));
        commands.spawn((Mesh3d(trunk.clone()), MeshMaterial3d(m.bark.clone()), Transform::from_xyz(p.x, 1.5, p.y), far.clone()));
        commands.spawn((
            Mesh3d(crown.clone()),
            MeshMaterial3d(m.leaves.clone()),
            Transform::from_xyz(p.x, 3.2 + s * 0.5, p.y).with_scale(Vec3::new(s, s * 0.8, s)),
            far.clone(),
        ));
        commands.spawn((
            Mesh3d(crown.clone()),
            MeshMaterial3d(m.leaves.clone()),
            Transform::from_xyz(p.x + s * 0.4, 3.6 + s * 0.4, p.y - s * 0.3).with_scale(Vec3::splat(s * 0.7)),
            far,
        ));
    }

    // Lanterns over Hàng Mã.
    let lantern = meshes.add(Sphere::new(0.22).mesh().uv(8, 6));
    for r in c.roads.iter().filter(|r| r.name.as_deref() == Some("Phố Hàng Mã")) {
        let pts: Vec<Vec2> = r.pts.iter().map(|p| to_world(c, p)).collect();
        for seg in pts.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let d = (b - a).normalize_or_zero();
            let n = Vec2::new(-d.y, d.x) * (r.w as f32 / 20.0 + 0.6);
            let steps = (a.distance(b) / 1.6).ceil() as i32;
            for k in 0..steps {
                let p = a.lerp(b, k as f32 / steps.max(1) as f32);
                for (side, red) in [(1.0f32, k % 2 == 0), (-1.0, k % 2 == 1)] {
                    let q = p + n * side;
                    let mat = if red { m.lantern_red.clone() } else { m.lantern_yellow.clone() };
                    if k % 4 == 0 {
                        glows.push(crate::night::Glow::lantern(Vec3::new(q.x, 4.4, q.y), red));
                    }
                    commands.spawn((
                        Mesh3d(lantern.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_xyz(q.x, 4.6 + (k % 3) as f32 * 0.25, q.y),
                        Far { at: Vec3::new(q.x, 0.0, q.y), limit: 200.0 },
                    ));
                }
            }
        }
    }

    // Shop signs: each board on the face of the nearest building, above the shopfront.
    let walls: Vec<(Vec2, Vec2)> = c
        .buildings
        .iter()
        .filter_map(|b| {
            let ring: Vec<Vec2> = b.outer.iter().map(|p| to_world(c, p)).collect();
            if ring.len() < 3 || !near(ring[0]) || ring[0].distance(center) > 900.0 {
                return None;
            }
            let ccw = signed_area(&ring) >= 0.0;
            Some(ring.windows(2).map(move |w| if ccw { (w[0], w[1]) } else { (w[1], w[0]) }).collect::<Vec<_>>())
        })
        .flatten()
        .filter(|(a, b)| a.distance(*b) > 2.5)
        .collect();
    let mut sign = Builder::default();
    for (i, (_, at)) in signs.iter().enumerate() {
        let p = to_world(c, at);
        let best = walls.iter().min_by(|x, y| seg_dist(p, x.0, x.1).total_cmp(&seg_dist(p, y.0, y.1)));
        let Some((a, b)) = best else { continue };
        if seg_dist(p, *a, *b) > 12.0 {
            continue;
        }
        let d = (*b - *a).normalize_or_zero();
        let out = Vec2::new(d.y, -d.x) * 0.12;
        let mid = (*a + *b) / 2.0 + out;
        let half = (a.distance(*b) / 2.0 - 0.3).clamp(0.8, 2.2);
        let (l, r) = (mid - d * half, mid + d * half);
        let (u0, v0) = ((i % 8) as f32 / 8.0, (i / 8) as f32 / 32.0);
        let (u1, v1) = (u0 + 1.0 / 8.0, v0 + 1.0 / 32.0);
        sign.quad(
            [Vec3::new(l.x, 3.75, l.y), Vec3::new(l.x, 4.7, l.y), Vec3::new(r.x, 4.7, r.y), Vec3::new(r.x, 3.75, r.y)],
            [Vec2::new(u0, v1), Vec2::new(u0, v0), Vec2::new(u1, v0), Vec2::new(u1, v1)],
        );
    }
    if let Some(mesh) = sign.mesh() {
        commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(m.signs.clone())));
    }
    let promenade = c
        .water
        .iter()
        .find(|w| w.name.as_deref() == Some("Hồ Hoàn Kiếm"))
        .map(|lake| {
            let mut ring: Vec<Vec2> = lake.outer.iter().map(|p| to_world(c, p)).collect();
            if signed_area(&ring) < 0.0 {
                ring.reverse();
            }
            let n = ring.len();
            (0..n)
                .map(|i| {
                    let (prev, here, next) = (ring[(i + n - 1) % n], ring[i], ring[(i + 1) % n]);
                    let d = (next - prev).normalize_or_zero();
                    here + Vec2::new(d.y, -d.x) * 3.5
                })
                .collect()
        })
        .unwrap_or_default();
    Built { sites, facades, promenade, glows }
}

type Chunks = HashMap<(i32, i32, u8, u8), Builder>;
type Own = HashMap<&'static str, HashMap<u8, Builder>>;

/// The mesh a building's faces go into: its chunk's, or its own if it stands in for a landmark.
fn pick<'a>(chunks: &'a mut Chunks, own: &'a mut Own, landmark: Option<&'static str>, key: (i32, i32, u8, u8)) -> &'a mut Builder {
    match landmark {
        Some(f) => own.entry(f).or_default().entry(key.2 * 8 + key.3).or_default(),
        None => chunks.entry(key).or_default(),
    }
}

/// Street furniture from the map, in the blocks it stands in (put up by `facade::stream` when the
/// art for it was delivered): granite kerbs along every street, lamps along the main roads, the low
/// railing round Hồ Gươm, and a name sign at both ends of every named street.
#[allow(clippy::too_many_arguments)]
fn street_furniture(
    c: &City,
    map: &Map,
    near: &dyn Fn(Vec2) -> bool,
    chunk_of: &dyn Fn(Vec2) -> (i32, i32),
    blocks: &mut HashMap<(i32, i32), facade::Block>,
    streets: &HashMap<String, u16>,
    decks: &[Vec2],
    glows: &mut Vec<crate::night::Glow>,
) {
    let mut put = |at: Vec2, slot: facade::PropSlot| {
        let (cx, cz) = chunk_of(at);
        blocks.entry((cx, cz)).or_insert_with(|| facade::Block::new(Vec3::new((cx as f32 + 0.5) * CHUNK, 0.0, (cz as f32 + 0.5) * CHUNK))).props.push(slot);
    };
    // A model's +X along `along`, its +Z toward `facing` (both flat).
    let turn = |along: Vec2, facing: Vec2| {
        let x = Vec3::new(along.x, 0.0, along.y).normalize_or(Vec3::X);
        let mut z = x.cross(Vec3::Y);
        if z.dot(Vec3::new(facing.x, 0.0, facing.y)) < 0.0 {
            z = -z;
        }
        Quat::from_mat3(&Mat3::from_cols(z.cross(Vec3::Y) * -1.0, Vec3::Y, z))
    };
    let road_at =
        |p: Vec2| matches!(map.tile_at(V2 { x: p.x / map.tile_m, y: p.y / map.tile_m }), Tile::Street | Tile::Alley | Tile::Crowd | Tile::Water | Tile::Bridge);

    for r in &c.roads {
        if r.bridge || r.w < 50 || matches!(r.kind.as_str(), "footway" | "path" | "steps" | "cycleway" | "pedestrian") {
            continue;
        }
        let pts: Vec<Vec2> = r.pts.iter().map(|p| to_world(c, p)).collect();
        if !pts.iter().any(|p| near(*p)) {
            continue;
        }
        let half = r.w as f32 / 20.0;
        let main = matches!(r.kind.as_str(), "primary" | "secondary" | "tertiary");
        let mut along = 0.0f32;
        let mut next_lamp = 12.0f32;
        let mut lamp_side = 1.0f32;
        for seg in pts.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let len = a.distance(b);
            if len < 0.5 {
                continue;
            }
            let d = (b - a) / len;
            let n = Vec2::new(-d.y, d.x);
            // Kerbs: 2 m granite pieces on both edges, left out where another street joins.
            let pieces = (len / 2.0).floor() as i32;
            for k in 0..pieces {
                let mid = a + d * (k as f32 * 2.0 + 1.0);
                for side in [-1.0f32, 1.0] {
                    let edge = mid + n * side * (half + 0.16);
                    if road_at(mid + n * side * (half + 1.6)) || !near(edge) {
                        continue;
                    }
                    let at = Transform::from_translation(Vec3::new(edge.x, 0.0, edge.y)).with_rotation(turn(d, -n * side)).with_scale(Vec3::new(2.0, 1.0, 1.0));
                    put(edge, facade::PropSlot { file: "street/bo_via.glb", at, upto: 70.0, street: None });
                }
            }
            // Lamps every 30 m along the main roads, side to side, their arms over the road.
            while main && next_lamp <= along + len {
                let p = a + d * (next_lamp - along);
                let foot = p + n * lamp_side * (half + 0.7);
                if !road_at(foot) && near(foot) {
                    let at = Transform::from_translation(Vec3::new(foot.x, 0.0, foot.y)).with_rotation(turn(d, -n * lamp_side));
                    glows.push(crate::night::Glow::lamp(&at));
                    put(foot, facade::PropSlot { file: crate::night::STREET_LAMP, at, upto: 260.0, street: None });
                }
                lamp_side = -lamp_side;
                next_lamp += 30.0;
            }
            along += len;
        }
        // Name signs at both ends of a named street, on the corner, facing along the street.
        if let Some(i) = r.name.as_ref().and_then(|n| streets.get(n)) {
            for (end, inward) in [(pts.first(), pts.get(1)), (pts.last(), pts.get(pts.len().saturating_sub(2)))] {
                let (Some(&e), Some(&next)) = (end, inward) else { continue };
                let d = (next - e).normalize_or_zero();
                let n = Vec2::new(-d.y, d.x);
                let foot = e + d * 4.0 + n * (half + 0.8);
                if road_at(foot) || !near(foot) {
                    continue;
                }
                let at = Transform::from_translation(Vec3::new(foot.x, 0.0, foot.y)).with_rotation(turn(n, -d));
                put(foot, facade::PropSlot { file: "street/cot_bien_ten_pho.glb", at, upto: 140.0, street: Some(*i) });
            }
        }
    }

    // The railing round Hồ Gươm: 2.1 m sections just outside the shore, open where the Thê Húc
    // bridge leaves it.
    if let Some(lake) = c.water.iter().find(|w| w.name.as_deref() == Some("Hồ Hoàn Kiếm")) {
        let mut ring: Vec<Vec2> = lake.outer.iter().map(|p| to_world(c, p)).collect();
        if signed_area(&ring) < 0.0 {
            ring.reverse();
        }
        let n = ring.len();
        // Lamps on the lake path every 25 m, 2 m in from the railing.
        let mut along = 0.0f32;
        let mut next_lamp = 10.0f32;
        for i in 0..n {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            let len = a.distance(b);
            if len < 0.5 {
                continue;
            }
            let d = (b - a) / len;
            let out = Vec2::new(d.y, -d.x);
            while next_lamp <= along + len {
                let p = a + d * (next_lamp - along) + out * 2.6;
                if !decks.iter().any(|q| q.distance(p) < 20.0) {
                    let at = Transform::from_translation(Vec3::new(p.x, 0.0, p.y)).with_rotation(turn(d, out));
                    glows.push(crate::night::Glow::lake_lamp(&at));
                    put(p, facade::PropSlot { file: crate::night::LAKE_LAMP, at, upto: 220.0, street: None });
                }
                next_lamp += 25.0;
            }
            along += len;
            let pieces = (len / 2.1).round().max(1.0) as i32;
            let step = len / pieces as f32;
            for k in 0..pieces {
                let p = a + d * (step * (k as f32 + 0.5)) + out * 0.6;
                if decks.iter().any(|q| q.distance(p) < 26.0) {
                    continue;
                }
                let at = Transform::from_translation(Vec3::new(p.x, 0.0, p.y)).with_rotation(turn(d, out)).with_scale(Vec3::new(step / 2.1, 1.0, 1.0));
                put(p, facade::PropSlot { file: "street/hang_rao_ho.glb", at, upto: 160.0, street: None });
            }
        }
    }
}

/// Whether `p` is inside the polygon `ring` (even-odd rule).
fn contains(ring: &[Vec2], p: Vec2) -> bool {
    let n = ring.len();
    let mut inside = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
    }
    inside
}

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 1e-6 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(a + ab * t)
}

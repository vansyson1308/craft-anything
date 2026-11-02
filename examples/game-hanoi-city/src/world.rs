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

/// Metres from the city's top-left corner: x east, z south.
pub fn to_world(c: &City, p: &Dm) -> Vec2 {
    Vec2::new((p[0] - c.bounds[0]) as f32 / 10.0, (p[1] - c.bounds[1]) as f32 / 10.0)
}

/// Builds meshes out of quads and triangles, one per material.
#[derive(Default)]
struct Builder {
    pos: Vec<[f32; 3]>,
    nrm: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    idx: Vec<u32>,
}

impl Builder {
    fn quad(&mut self, p: [Vec3; 4], uv: [Vec2; 4]) {
        let n = (p[1] - p[0]).cross(p[2] - p[0]).normalize_or_zero();
        let base = self.pos.len() as u32;
        for i in 0..4 {
            self.pos.push(p[i].to_array());
            self.nrm.push(n.to_array());
            self.uv.push(uv[i].to_array());
        }
        self.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    fn tris(&mut self, pts: &[Vec3], tris: &[[usize; 3]], uv_scale: f32) {
        let base = self.pos.len() as u32;
        for p in pts {
            self.pos.push(p.to_array());
            self.nrm.push([0.0, 1.0, 0.0]);
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

pub struct Mats {
    pub walls_up: Vec<Handle<StandardMaterial>>,
    pub walls_ground: Vec<Handle<StandardMaterial>>,
    pub roof: Handle<StandardMaterial>,
    pub roof_tiles: Handle<StandardMaterial>,
    pub asphalt: Handle<StandardMaterial>,
    pub paving: Handle<StandardMaterial>,
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
            emissive: LinearRgba::rgb(1.6, 1.3, 1.0),
            perceptual_roughness: 0.9,
            ..default()
        }));
        walls_ground.push(mats.add(StandardMaterial {
            base_color_texture: Some(repeat(assets, GR[i])),
            emissive_texture: Some(repeat(assets, GR_GLOW[i])),
            emissive: LinearRgba::rgb(2.0, 1.6, 1.1),
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
        grass: tex("tex/grass.jpg", 0.95, mats),
        water: mats.add(StandardMaterial { base_color: Color::srgb(0.16, 0.42, 0.38), perceptual_roughness: 0.08, reflectance: 0.6, ..default() }),
        red_bridge: mats.add(StandardMaterial { base_color: Color::srgb(0.72, 0.1, 0.1), perceptual_roughness: 0.6, ..default() }),
        steel: mats.add(StandardMaterial { base_color: Color::srgb(0.42, 0.28, 0.2), metallic: 0.5, perceptual_roughness: 0.6, ..default() }),
        rail_bed: mats.add(StandardMaterial { base_color: Color::srgb(0.3, 0.27, 0.24), perceptual_roughness: 1.0, ..default() }),
        signs: mats.add(StandardMaterial {
            base_color_texture: Some(signs_tex.clone()),
            emissive_texture: Some(signs_tex),
            emissive: LinearRgba::rgb(0.9, 0.9, 0.9),
            unlit: false,
            ..default()
        }),
        tank: mats.add(StandardMaterial { base_color: Color::srgb(0.82, 0.84, 0.86), metallic: 0.8, perceptual_roughness: 0.3, ..default() }),
        bark: tex("tex/bark.jpg", 1.0, mats),
        leaves: mats.add(StandardMaterial { base_color: Color::srgb(0.18, 0.38, 0.17), perceptual_roughness: 0.9, ..default() }),
        lantern_red: mats.add(StandardMaterial { base_color: Color::srgb(0.9, 0.1, 0.1), emissive: LinearRgba::rgb(6.0, 0.6, 0.3), ..default() }),
        lantern_yellow: mats.add(StandardMaterial { base_color: Color::srgb(0.95, 0.75, 0.2), emissive: LinearRgba::rgb(6.0, 4.0, 0.8), ..default() }),
    }
}

/// How far from the lake's centre the town is drawn (metres): beyond it, fog.
pub const DRAW_RADIUS: f32 = 1000.0;
const CHUNK: f32 = 160.0;

/// Spawns the whole district. `center` is where the lake is (metres).
pub fn spawn_city(commands: &mut Commands, meshes: &mut Assets<Mesh>, m: &Mats, c: &City, center: Vec2, signs: &[(String, Dm)]) {
    let near = |p: Vec2| p.distance(center) < DRAW_RADIUS;
    let chunk_of = |p: Vec2| ((p.x / CHUNK).floor() as i32, (p.y / CHUNK).floor() as i32);
    let mut chunks: HashMap<(i32, i32, u8, u8), Builder> = HashMap::new();
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
        if matches!(r.kind.as_str(), "footway" | "path" | "steps" | "cycleway") && !r.bridge {
            continue;
        }
        let pts: Vec<Vec2> = r.pts.iter().map(|p| to_world(c, p)).collect();
        if !pts.iter().any(|p| near(*p)) {
            continue;
        }
        let half = r.w as f32 / 20.0;
        let (kind, y) = if r.bridge { if r.name.as_deref() == Some("Cầu Long Biên") { (8u8, 4.0f32) } else { (7u8, 0.6f32) } } else { (4u8, 0.04f32) };
        let mut along = 0.0;
        for seg in pts.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let d = (b - a).normalize_or_zero();
            let n = Vec2::new(-d.y, d.x) * half;
            let len = a.distance(b);
            let (cx, cz) = chunk_of(a);
            let s = 1.0 / (2.0 * half).max(1.0);
            chunks.entry((cx, cz, kind, 0)).or_default().quad(
                [
                    Vec3::new(a.x + n.x, y, a.y + n.y),
                    Vec3::new(b.x + n.x, y, b.y + n.y),
                    Vec3::new(b.x - n.x, y, b.y - n.y),
                    Vec3::new(a.x - n.x, y, a.y - n.y),
                ],
                [Vec2::new(0.0, along * s), Vec2::new(0.0, (along + len) * s), Vec2::new(1.0, (along + len) * s), Vec2::new(1.0, along * s)],
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
        let height = (b.h as f32 / 10.0).clamp(3.0, 120.0);
        let style = (hash2(bi as i32, 7) % 6) as u8;
        let (cx, cz) = chunk_of(ring[0]);
        let ground = 3.6f32.min(height);
        let n = ring.len();
        for i in 0..n {
            let (a, bb) = (ring[i], ring[(i + 1) % n]);
            let len = a.distance(bb);
            if len < 0.05 {
                continue;
            }
            let u = (len / 3.0).max(1.0).round();
            let wall = |y0: f32, y1: f32| [Vec3::new(a.x, y0, a.y), Vec3::new(a.x, y1, a.y), Vec3::new(bb.x, y1, bb.y), Vec3::new(bb.x, y0, bb.y)];
            chunks
                .entry((cx, cz, 1, style))
                .or_default()
                .quad(wall(0.0, ground), [Vec2::new(0.0, 1.0), Vec2::new(0.0, 0.0), Vec2::new(u, 0.0), Vec2::new(u, 1.0)]);
            if height > ground + 0.5 {
                let floors = ((height - ground) / 3.3).max(1.0);
                chunks
                    .entry((cx, cz, 0, style))
                    .or_default()
                    .quad(wall(ground, height), [Vec2::new(0.0, floors), Vec2::new(0.0, 0.0), Vec2::new(u, 0.0), Vec2::new(u, floors)]);
            }
        }
        let tris = earcut(&ring);
        let pts: Vec<Vec3> = ring.iter().map(|p| Vec3::new(p.x, height, p.y)).collect();
        let tiled = b.kind == "temple" || b.kind == "church" || hash2(bi as i32, 3) % 9 == 0;
        chunks.entry((cx, cz, if tiled { 3 } else { 2 }, 0)).or_default().tris(&pts, &tris, 0.25);
        // Inox water tanks on the roofs of homes near the middle.
        if !tiled && height < 25.0 && hash2(bi as i32, 11) % 4 == 0 && ring[0].distance(center) < 700.0 {
            let mid = ring.iter().copied().sum::<Vec2>() / n as f32;
            tanks.push(Vec3::new(mid.x, height, mid.y));
        }
    }

    for ((cx, cz, kind, style), b) in chunks {
        let b_center = Vec3::new((cx as f32 + 0.5) * CHUNK, 0.0, (cz as f32 + 0.5) * CHUNK);
        let mat = match kind {
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
            _ => m.paving.clone(),
        };
        if let Some(mesh) = b.mesh() {
            // Chunks beyond the fog aren't drawn at all (see cull_far).
            let far = Far { at: b_center, limit: 450.0 + CHUNK };
            commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(mat), far));
        }
    }

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
}

fn seg_dist(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = if ab.length_squared() > 1e-6 { ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(a + ab * t)
}

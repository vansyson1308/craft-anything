//! Street fronts from delivered modules (ASSETS.md, group C): every building face that looks onto
//! a street is cut into 4 m bays and storeys (a 3.6 m shopfront, then 3.3 m floors), and each slot
//! takes one of the delivered modules. Only the blocks near the camera get them, the way large
//! open worlds stream their streets; farther away the flat stand-in front stays.

use bevy::prelude::*;

use crate::hq::{Hq, HqState, LodModel};

/// Size of the modules (metres): ground storey, upper storey, and bay width.
pub const GROUND_H: f32 = 3.6;
pub const UPPER_H: f32 = 3.3;
pub const BAY_W: f32 = 4.0;

/// Which kind of module a slot takes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Storey {
    Ground,
    Upper,
    FrenchGround,
    FrenchUpper,
}

/// One module's place: bottom-left corner, the facade's direction to the right (seen from the
/// street), how much to stretch the 4 m module to fill the bay, and a number for picking a variant.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub corner: Vec3,
    pub right: Vec2,
    pub stretch: f32,
    pub storey: Storey,
    pub seed: u32,
}

impl Slot {
    /// The module's transform: local +X along the front, +Y up, +Z out to the street.
    pub fn transform(&self) -> Transform {
        let x = Vec3::new(self.right.x, 0.0, self.right.y);
        let z = x.cross(Vec3::Y);
        Transform { translation: self.corner, rotation: Quat::from_mat3(&Mat3::from_cols(x, Vec3::Y, z)), scale: Vec3::new(self.stretch, 1.0, 1.0) }
    }
}

/// Splits a street-facing face (from `b` to `a` as seen from the street, so the module's right is
/// toward `a`) into slots; returns them and the height they cover.
pub fn slots_for(a: Vec2, b: Vec2, height: f32, french: bool, seed: u32) -> (Vec<Slot>, f32) {
    let len = a.distance(b);
    if len < 3.0 || height < GROUND_H {
        return (Vec::new(), 0.0);
    }
    let bays = ((len / BAY_W).round() as usize).max(1);
    let bay = len / bays as f32;
    let right = (a - b) / len;
    let upper_floors = ((height - GROUND_H + 0.4) / UPPER_H).floor().max(0.0) as usize;
    let covered = GROUND_H + upper_floors as f32 * UPPER_H;
    let mut out = Vec::with_capacity(bays * (1 + upper_floors));
    for i in 0..bays {
        let c = b + right * bay * i as f32;
        for f in 0..=upper_floors {
            let (y, storey) = if f == 0 {
                (0.0, if french { Storey::FrenchGround } else { Storey::Ground })
            } else {
                (GROUND_H + (f - 1) as f32 * UPPER_H, if french { Storey::FrenchUpper } else { Storey::Upper })
            };
            let s = seed.wrapping_mul(2_654_435_761).wrapping_add((i as u32) * 97 + f as u32 * 13);
            out.push(Slot { corner: Vec3::new(c.x, y, c.y), right, stretch: bay / BAY_W, storey, seed: s });
        }
    }
    (out, covered)
}

/// A piece of street furniture to stand in a block: which delivered file, where, how far away it
/// is still drawn, and for a street-name sign, which street (its cell in the name atlas).
#[derive(Clone, Debug)]
pub struct PropSlot {
    pub file: &'static str,
    pub at: Transform,
    pub upto: f32,
    pub street: Option<u16>,
    /// Other files that may stand here instead (variants of the same thing): one of those
    /// delivered is picked, the same one every time for this spot.
    pub alts: &'static [&'static str],
}

impl PropSlot {
    pub fn new(file: &'static str, at: Transform, upto: f32) -> PropSlot {
        PropSlot { file, at, upto, street: None, alts: &[] }
    }

    /// The file to stand here among those delivered: the slot's own if it was delivered, else a
    /// delivered variant picked by where it stands.
    pub fn pick(&self, delivered: impl Fn(&str) -> bool) -> Option<&'static str> {
        if delivered(self.file) {
            return Some(self.file);
        }
        let there: Vec<&'static str> = self.alts.iter().copied().filter(|f| delivered(f)).collect();
        let t = self.at.translation;
        let h = (t.x * 7.31 + t.z * 3.17).abs() as usize;
        there.get(h % there.len().max(1)).copied()
    }
}

/// A block's street: its front slots, the flat stand-in fronts they replace, and its furniture.
pub struct Block {
    pub center: Vec3,
    pub slots: Vec<Slot>,
    pub props: Vec<PropSlot>,
    pub flat: Vec<Entity>,
    shown: Option<Entity>,
}

impl Block {
    pub fn new(center: Vec3) -> Block {
        Block { center, slots: Vec::new(), props: Vec::new(), flat: Vec::new(), shown: None }
    }
}

#[derive(Resource, Default)]
pub struct Facades {
    pub blocks: Vec<Block>,
}

/// How far a shopfront's ground floor reaches out from its wall (counters, steps, awnings' posts).
pub const SHOPFRONT_DEPTH_M: f32 = 1.1;

impl Facades {
    /// Whether `p` is inside a shopfront's ground floor, or less than `depth` in front of its wall:
    /// where a camera would look out from inside a counter. Only blocks near `p` are looked at.
    pub fn in_shopfront(&self, p: Vec3, depth: f32) -> bool {
        if p.y >= GROUND_H {
            return false;
        }
        // A block holds what is within its 160 m cell, so nothing in it is farther than this from
        // its middle.
        const BLOCK_REACH: f32 = 120.0;
        self.blocks.iter().filter(|b| b.center.with_y(0.0).distance(p.with_y(0.0)) < BLOCK_REACH).flat_map(|b| b.slots.iter()).any(|s| {
            if !matches!(s.storey, Storey::Ground | Storey::FrenchGround) {
                return false;
            }
            let d = Vec2::new(p.x - s.corner.x, p.z - s.corner.z);
            let along = d.dot(s.right);
            // Out to the street: the module's +Z.
            let out = d.dot(Vec2::new(-s.right.y, s.right.x));
            (0.0..=BAY_W * s.stretch).contains(&along) && (-1.0..depth).contains(&out)
        })
    }
}

/// The street-name faces: the atlas drawn by `tools/make_street_names.py`, and a quad per street
/// mapped to its cell, made the first time a sign for that street goes up.
#[derive(Resource)]
pub struct SignFaces {
    pub material: Handle<StandardMaterial>,
    pub cols: u16,
    pub rows: u16,
    quads: std::collections::HashMap<u16, Handle<Mesh>>,
}

impl SignFaces {
    pub fn new(material: Handle<StandardMaterial>, cols: u16, rows: u16) -> SignFaces {
        SignFaces { material, cols: cols.max(1), rows: rows.max(1), quads: Default::default() }
    }
    /// The face for street `i`: a 0.94 × 0.27 m quad standing just in front of the plate.
    fn quad(&mut self, i: u16, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        let (cols, rows) = (self.cols, self.rows);
        self.quads
            .entry(i)
            .or_insert_with(|| {
                let (cu, cv) = (1.0 / cols as f32, 1.0 / rows as f32);
                let (u0, v0) = ((i % cols) as f32 * cu, (i / cols) as f32 * cv);
                let (u1, v1) = (u0 + cu, v0 + cv);
                let (w, y0, y1, z) = (0.47, 2.445, 2.715, 0.034);
                use bevy::asset::RenderAssetUsages;
                use bevy::mesh::{Indices, PrimitiveTopology};
                meshes.add(
                    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[-w, y0, z], [w, y0, z], [w, y1, z], [-w, y1, z]])
                        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4])
                        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[u0, v1], [u1, v1], [u1, v0], [u0, v0]])
                        .with_inserted_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3])),
                )
            })
            .clone()
    }
}

/// The streets that have a name sign, and the atlas's columns and rows
/// (`assets/tex/street_names.json`, written by `tools/make_street_names.py`).
pub fn street_atlas() -> (std::collections::HashMap<String, u16>, u16, u16) {
    let v: serde_json::Value = serde_json::from_str(include_str!("../assets/tex/street_names.json")).unwrap_or_default();
    let names = v.get("names").and_then(|n| n.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_default();
    let map = names.iter().enumerate().filter_map(|(i, n)| u16::try_from(i).ok().map(|i| (n.to_string(), i))).collect();
    let num = |k: &str| v.get(k).and_then(|x| x.as_u64()).and_then(|x| u16::try_from(x).ok()).unwrap_or(1);
    (map, num("cols"), num("rows"))
}

/// How near (metres from a block's centre) the modules go up, and how many blocks are dressed per
/// step so the frame rate never dips.
const NEAR: f32 = 230.0;
const PER_STEP: usize = 2;

fn modules(hq: &Hq, prefix: &str, n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("buildings/{prefix}_{i:02}.glb")).filter(|f| hq.get(f).is_some()).collect()
}

/// Dresses the blocks near the camera (front modules, street furniture) and undresses the ones
/// left behind.
#[allow(clippy::too_many_arguments)]
pub fn stream(
    mut commands: Commands,
    hq: Res<Hq>,
    time: Res<Time>,
    mut next: Local<f32>,
    mut lists: Local<Option<[Vec<String>; 4]>>,
    mut facades: ResMut<Facades>,
    mut signs: Option<ResMut<SignFaces>>,
    mut meshes: ResMut<Assets<Mesh>>,
    cam: Query<&GlobalTransform, With<Camera3d>>,
    mut vis: Query<&mut Visibility>,
) {
    if hq.state != HqState::Ready {
        return;
    }
    *next -= time.delta_secs();
    if *next > 0.0 {
        return;
    }
    *next = 0.3;
    let lists = lists.get_or_insert_with(|| {
        let ground = modules(&hq, "mat_tien_tret", 8);
        let upper = modules(&hq, "mat_tien_tang", 10);
        let fg = modules(&hq, "mat_tien_phap_tret", 3);
        let fu = modules(&hq, "mat_tien_phap_tang", 3);
        // French fronts fall back to the ordinary ones; without an ordinary pair, no fronts.
        let (fg, fu) = if fg.is_empty() || fu.is_empty() { (ground.clone(), upper.clone()) } else { (fg, fu) };
        [ground, upper, fg, fu]
    });
    let fronts = !lists[0].is_empty() && !lists[1].is_empty();
    let Ok(cam) = cam.single() else { return };
    let c = cam.translation();
    let flat = Vec3::new(c.x, 0.0, c.z);
    let mut budget = PER_STEP;
    for block in facades.blocks.iter_mut() {
        let near = block.center.distance(flat) < NEAR;
        match (near, block.shown) {
            (true, None) if budget > 0 => {
                budget -= 1;
                let holder = commands.spawn((Transform::default(), Visibility::default())).id();
                if fronts {
                    for s in &block.slots {
                        let list = match s.storey {
                            Storey::Ground => &lists[0],
                            Storey::Upper => &lists[1],
                            Storey::FrenchGround => &lists[2],
                            Storey::FrenchUpper => &lists[3],
                        };
                        let Some(file) = list.get(s.seed as usize % list.len().max(1)) else { continue };
                        let Some(entry) = hq.get(file) else { continue };
                        let levels = crate::hq::levels_of(entry);
                        let upto = if levels.len() > 1 { vec![45.0, 10_000.0] } else { vec![10_000.0] };
                        let m = commands.spawn((s.transform(), Visibility::default(), LodModel::new(levels, upto))).id();
                        commands.entity(holder).add_child(m);
                    }
                    for e in &block.flat {
                        if let Ok(mut v) = vis.get_mut(*e) {
                            *v = Visibility::Hidden;
                        }
                    }
                }
                for p in &block.props {
                    let Some(entry) = p.pick(|f| hq.get(f).is_some()).and_then(|f| hq.get(f)) else { continue };
                    let levels = crate::hq::levels_of(entry);
                    let n = levels.len();
                    let upto = (0..n).map(|i| p.upto * (i + 1) as f32 / n as f32).collect();
                    let m = commands.spawn((p.at, Visibility::default(), LodModel::new(levels, upto))).id();
                    if let (Some(i), Some(signs)) = (p.street, signs.as_deref_mut()) {
                        let far = crate::world::Far { at: p.at.translation, limit: p.upto };
                        let face = commands.spawn((Mesh3d(signs.quad(i, &mut meshes)), MeshMaterial3d(signs.material.clone()), Transform::default(), far)).id();
                        commands.entity(m).add_child(face);
                    }
                    commands.entity(holder).add_child(m);
                }
                block.shown = Some(holder);
            }
            (false, Some(holder)) => {
                commands.entity(holder).despawn();
                block.shown = None;
                for e in &block.flat {
                    if let Ok(mut v) = vis.get_mut(*e) {
                        *v = Visibility::Inherited;
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_camera_in_a_shop_counter_is_found() {
        // One 8 m face along x, seen from the street to the south (+z): two ground bays.
        let (slots, _) = slots_for(Vec2::new(8.0, 0.0), Vec2::new(0.0, 0.0), 10.0, false, 1);
        let mut b = Block::new(Vec3::new(4.0, 0.0, 0.0));
        b.slots = slots;
        let f = Facades { blocks: vec![b] };
        let out = Vec2::new(-f.blocks[0].slots[0].right.y, f.blocks[0].slots[0].right.x);
        let at = |along: f32, away: f32, y: f32| Vec3::new(along, y, 0.0) + Vec3::new(out.x, 0.0, out.y) * away;
        // Inside the counter, half a metre out from the wall at eye height.
        assert!(f.in_shopfront(at(3.0, 0.5, 1.5), SHOPFRONT_DEPTH_M));
        // Out on the pavement, past the counters; above the ground floor; beside the face.
        assert!(!f.in_shopfront(at(3.0, 1.6, 1.5), SHOPFRONT_DEPTH_M));
        assert!(!f.in_shopfront(at(3.0, 0.5, 5.0), SHOPFRONT_DEPTH_M));
        assert!(!f.in_shopfront(at(9.5, 0.5, 1.5), SHOPFRONT_DEPTH_M));
    }

    #[test]
    fn a_tube_house_front() {
        // 4.2 m wide, 10.9 m tall: one bay, a shopfront and two floors (the last 0.7 m stays flat).
        let (slots, covered) = slots_for(Vec2::new(4.2, 0.0), Vec2::ZERO, 10.9, false, 7);
        assert_eq!(slots.len(), 3);
        assert!((covered - 10.2).abs() < 1e-4);
        assert_eq!(slots[0].storey, Storey::Ground);
        assert_eq!(slots[1].storey, Storey::Upper);
        assert!((slots[2].corner.y - 6.9).abs() < 1e-4);
        assert!((slots[0].stretch - 1.05).abs() < 1e-4);
    }

    #[test]
    fn wide_and_tiny_fronts() {
        let (slots, _) = slots_for(Vec2::new(0.0, 17.0), Vec2::ZERO, 3.6, true, 1);
        assert_eq!(slots.len(), 4);
        assert!(slots.iter().all(|s| s.storey == Storey::FrenchGround));
        assert!(slots_for(Vec2::new(2.0, 0.0), Vec2::ZERO, 12.0, false, 1).0.is_empty());
        assert!(slots_for(Vec2::new(8.0, 0.0), Vec2::ZERO, 3.0, false, 1).0.is_empty());
    }

    #[test]
    fn modules_face_the_street() {
        // The north face of a building, its ring running west to east (a = west, b = east): seen
        // from the street to the north, right is west, the first bay starts at the east corner and
        // the module's +Z points north.
        let (slots, _) = slots_for(Vec2::new(0.0, 0.0), Vec2::new(10.0, 0.0), 4.0, false, 1);
        let t = slots[0].transform();
        let out = t.rotation * Vec3::Z;
        assert!((out - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-4, "{out:?}");
        let right = t.rotation * Vec3::X;
        assert!((right - Vec3::NEG_X).length() < 1e-4, "{right:?}");
        assert!((slots[0].corner - Vec3::new(10.0, 0.0, 0.0)).length() < 1e-4);
    }
}

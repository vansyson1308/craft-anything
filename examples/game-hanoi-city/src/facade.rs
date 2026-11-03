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

/// A block's fronts: its slots, and the flat stand-in fronts they replace.
pub struct Block {
    pub center: Vec3,
    pub slots: Vec<Slot>,
    pub flat: Vec<Entity>,
    shown: Option<Entity>,
}

impl Block {
    pub fn new(center: Vec3) -> Block {
        Block { center, slots: Vec::new(), flat: Vec::new(), shown: None }
    }
}

#[derive(Resource, Default)]
pub struct Facades {
    pub blocks: Vec<Block>,
}

/// How near (metres from a block's centre) the modules go up, and how many blocks are dressed per
/// step so the frame rate never dips.
const NEAR: f32 = 230.0;
const PER_STEP: usize = 2;

fn modules(hq: &Hq, prefix: &str, n: usize) -> Vec<String> {
    (1..=n).map(|i| format!("buildings/{prefix}_{i:02}.glb")).filter(|f| hq.get(f).is_some()).collect()
}

/// Puts modules on the blocks near the camera and takes them off the ones left behind.
#[allow(clippy::too_many_arguments)]
pub fn stream(
    mut commands: Commands,
    hq: Res<Hq>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut next: Local<f32>,
    mut lists: Local<Option<[Vec<String>; 4]>>,
    mut facades: ResMut<Facades>,
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
        // French fronts fall back to the ordinary ones; with no ordinary pair there is nothing to do.
        let (fg, fu) = if fg.is_empty() || fu.is_empty() { (ground.clone(), upper.clone()) } else { (fg, fu) };
        [ground, upper, fg, fu]
    });
    if lists[0].is_empty() || lists[1].is_empty() {
        return;
    }
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
                for s in &block.slots {
                    let list = match s.storey {
                        Storey::Ground => &lists[0],
                        Storey::Upper => &lists[1],
                        Storey::FrenchGround => &lists[2],
                        Storey::FrenchUpper => &lists[3],
                    };
                    let Some(file) = list.get(s.seed as usize % list.len().max(1)) else { continue };
                    let Some(entry) = hq.get(file) else { continue };
                    let levels: Vec<Handle<bevy::gltf::Gltf>> =
                        std::iter::once(&entry.file).chain(entry.lods.iter()).map(|f| assets.load(Hq::path(f))).collect();
                    let upto = if levels.len() > 1 { vec![45.0, 10_000.0] } else { vec![10_000.0] };
                    let m = commands.spawn((s.transform(), Visibility::default(), LodModel::new(levels, upto))).id();
                    commands.entity(holder).add_child(m);
                }
                block.shown = Some(holder);
                for e in &block.flat {
                    if let Ok(mut v) = vis.get_mut(*e) {
                        *v = Visibility::Hidden;
                    }
                }
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

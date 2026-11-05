//! Life on the pavement in front of the shops, the way Hanoi's streets look: motorbikes parked
//! nose to the wall, low plastic stools round a table outside a shop, a potted plant, a bin.
//! Each shopfront bay gets its own mix, the same every time (from the bay's number), and only on
//! pavement the map says is open. The pieces are delivered art (ASSETS.md §6, group D): a piece
//! shows up as soon as its file is delivered, and until then nothing stands there.

use bevy::prelude::*;

use crate::facade::{BAY_W, PropSlot, Slot, Storey};

/// The parked motorbikes (any delivered one of them is used).
pub const PARKED_BIKES: &[&str] = &["street/xe_may_01_dau.glb", "street/xe_may_02_dau.glb", "street/xe_may_03_dau.glb", "street/xe_may_04_dau.glb"];
pub const STOOLS: &[&str] = &["street/ghe_nhua_xanh.glb", "street/ghe_nhua_do.glb"];
pub const TABLE: &str = "street/ban_nhua.glb";
pub const PLANT: &str = "street/chau_cay.glb";
pub const BIN: &str = "street/thung_rac.glb";

/// How far away each kind is still drawn (metres).
const BIKE_UPTO: f32 = 80.0;
const SMALL_UPTO: f32 = 55.0;

/// A number in 0..1 from a seed and a salt.
fn roll(seed: u32, salt: u32) -> f32 {
    let mut h = seed ^ salt.wrapping_mul(0x9E37_79B9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h = h.wrapping_mul(0xC2B2_AE35);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// The pavement pieces in front of one ground-floor bay. `open` says whether a point on the
/// ground (x, z in metres) is pavement or street someone could stand on; pieces that would stand
/// anywhere else are left out.
pub fn for_bay(slot: &Slot, open: &dyn Fn(Vec2) -> bool) -> Vec<PropSlot> {
    if !matches!(slot.storey, Storey::Ground | Storey::FrenchGround) {
        return Vec::new();
    }
    let shop = slot.storey == Storey::Ground;
    let right = slot.right;
    // Out of the building, toward the street (the module's +Z; see `Slot::transform`).
    let out = Vec2::new(-right.y, right.x);
    let corner = Vec2::new(slot.corner.x, slot.corner.z);
    let bay = BAY_W * slot.stretch;
    let at = |along: f32, away: f32| corner + right * along + out * away;
    // A flat transform at a point, its +Z turned toward `facing`, then by `jitter` radians.
    let place = |p: Vec2, facing: Vec2, jitter: f32| {
        let yaw = facing.x.atan2(facing.y) + jitter;
        Transform::from_xyz(p.x, 0.0, p.y).with_rotation(Quat::from_rotation_y(yaw))
    };
    let mut out_props = Vec::new();
    let mut put = |file: &'static str, alts: &'static [&'static str], t: Transform, upto: f32| {
        if open(Vec2::new(t.translation.x, t.translation.z)) {
            out_props.push(PropSlot { file, at: t, upto, street: None, alts });
        }
    };
    let s = slot.seed;
    let what = roll(s, 1);
    let bikes_share = if shop { 0.55 } else { 0.3 };
    if what < bikes_share {
        // One to three motorbikes, nose to the wall, side by side across the bay.
        let n = 1 + (roll(s, 2) * 3.0) as usize % 3;
        for k in 0..n {
            let along = bay * (k as f32 + 0.5) / n as f32 + (roll(s, 10 + k as u32) - 0.5) * 0.3;
            let p = at(along, 1.15 + roll(s, 20 + k as u32) * 0.35);
            let jitter = (roll(s, 30 + k as u32) - 0.5) * 0.5;
            let t = place(p, -out, jitter);
            let first = PARKED_BIKES.get((roll(s, 40 + k as u32) * PARKED_BIKES.len() as f32) as usize).copied().unwrap_or("street/xe_may_01_dau.glb");
            put(first, PARKED_BIKES, t, BIKE_UPTO);
        }
    } else if shop && what < bikes_share + 0.2 {
        // A low table outside the shop with stools round it.
        let mid = at(bay / 2.0, 1.7);
        put(TABLE, &[], place(mid, out, roll(s, 3) * 0.6), SMALL_UPTO);
        let n = 3 + (roll(s, 4) * 2.0) as usize % 2;
        for k in 0..n {
            let a = std::f32::consts::TAU * (k as f32 + roll(s, 5) * 0.3) / n as f32;
            let p = mid + Vec2::new(a.cos(), a.sin()) * 0.6;
            let stool = STOOLS.get((roll(s, 50 + k as u32) * STOOLS.len() as f32) as usize).copied().unwrap_or("street/ghe_nhua_xanh.glb");
            put(stool, STOOLS, place(p, mid - p, roll(s, 60 + k as u32)), SMALL_UPTO);
        }
    }
    if roll(s, 6) < 0.14 {
        put(PLANT, &[], place(at(bay - 0.35, 0.35), out, roll(s, 7) * 6.0), SMALL_UPTO);
    }
    if roll(s, 8) < 0.06 {
        put(BIN, &[], place(at(0.4, 0.4), out, 0.0), SMALL_UPTO);
    }
    out_props
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bay(seed: u32, storey: Storey) -> Slot {
        // A front along the x axis, looking toward +z (the building is at −z).
        Slot { corner: Vec3::ZERO, right: Vec2::X, stretch: 1.0, storey, seed }
    }

    #[test]
    fn pieces_stand_on_the_pavement_in_front() {
        let mut bikes = 0;
        let mut tables = 0;
        for seed in 0..400 {
            for p in for_bay(&bay(seed * 7919, Storey::Ground), &|_| true) {
                let t = p.at.translation;
                assert!((-0.1..=4.1).contains(&t.x), "{t:?} is beside the bay");
                assert!(t.z > 0.2 && t.z < 2.6, "{t:?} is not on the pavement in front");
                if PARKED_BIKES.contains(&p.file) {
                    bikes += 1;
                    // Nose to the wall: the bike's +Z points back toward the building (−z).
                    assert!((p.at.rotation * Vec3::Z).z < -0.9, "a bike faces away from the wall");
                }
                if p.file == TABLE {
                    tables += 1;
                }
            }
        }
        assert!(bikes > 300, "{bikes} bikes in 400 bays");
        assert!(tables > 40, "{tables} tables in 400 bays");
    }

    #[test]
    fn nothing_where_the_map_is_closed_or_upstairs() {
        for seed in 0..100 {
            assert!(for_bay(&bay(seed, Storey::Ground), &|_| false).is_empty());
            assert!(for_bay(&bay(seed, Storey::Upper), &|_| true).is_empty());
        }
    }

    #[test]
    fn the_same_bay_gets_the_same_mix() {
        let a: Vec<_> = for_bay(&bay(1234, Storey::Ground), &|_| true).iter().map(|p| (p.file, p.at.translation)).collect();
        let b: Vec<_> = for_bay(&bay(1234, Storey::Ground), &|_| true).iter().map(|p| (p.file, p.at.translation)).collect();
        assert_eq!(a, b);
    }
}

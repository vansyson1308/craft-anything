//! Dog cages in the world. A cage stands clear of kerbs (the raid places it on the walking map's
//! 2 m tiles, which can straddle a kerb line), and a delivered cage's door swings open on the joint
//! its artist gave it (an `s3d_joint` in the model, kept as a [`Joint`] when the model is loaded)
//! when the dog is freed. The stand-in box has no door and just disappears.

use bevy::prelude::*;

use crate::hq::Joint;
use crate::world::Carriageways;

/// How long the door takes to swing fully open.
const SWING_S: f32 = 0.9;
/// Half a cage's footprint, rounded up (0.9 × 0.73 m): how far its middle must stand from a
/// carriageway's edge so no part of it is on the kerb.
const RADIUS: f32 = 0.5;
/// The farthest a cage is moved to get clear of a kerb; farther than that, it stays where it is.
const MAX_SHIFT: f32 = 1.2;

/// A cage in the world.
#[derive(Component, Default)]
pub struct CageDoor {
    /// Whether the delivered model (with a door) stands here, rather than the stand-in box.
    pub model: bool,
    /// Set when the dog is freed.
    pub open: bool,
    /// How long the door has been swinging.
    t: f32,
    /// How far the cage was moved to get clear of a kerb; the dog inside moves with it.
    pub shift: Vec3,
    settled: bool,
}

impl CageDoor {
    pub fn new(model: bool) -> CageDoor {
        CageDoor { model, ..default() }
    }
}

/// How far open a door is, `t` seconds after it was let go: smooth at both ends.
pub fn swing_amount(t: f32) -> f32 {
    let x = (t / SWING_S).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Moves each new cage off any kerb it stands across, once.
pub fn settle(ways: Option<Res<Carriageways>>, mut cages: Query<(&mut CageDoor, &mut Transform)>) {
    let Some(ways) = ways else { return };
    for (mut c, mut t) in &mut cages {
        if c.settled {
            continue;
        }
        c.settled = true;
        let p = Vec2::new(t.translation.x, t.translation.z);
        let to = ways.off_kerb(p, RADIUS, MAX_SHIFT);
        c.shift = Vec3::new(to.x - p.x, 0.0, to.y - p.y);
        t.translation += c.shift;
    }
}

/// Swings open the doors of cages whose dog was freed.
pub fn swing(time: Res<Time>, mut doors: Query<(Entity, &mut CageDoor)>, kids: Query<&Children>, mut joints: Query<(&Joint, &mut Transform)>) {
    for (e, mut d) in &mut doors {
        if !(d.open && d.model) {
            continue;
        }
        // Kept up after it's fully open: a level of detail can load later and needs its door set.
        d.t = (d.t + time.delta_secs()).min(SWING_S);
        let k = swing_amount(d.t);
        for node in kids.iter_descendants(e) {
            if let Ok((j, mut t)) = joints.get_mut(node) {
                t.rotation = j.at(k);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_door_swings_smoothly_and_stops_open() {
        assert_eq!(swing_amount(0.0), 0.0);
        assert!((swing_amount(SWING_S / 2.0) - 0.5).abs() < 1e-6);
        assert_eq!(swing_amount(SWING_S), 1.0);
        assert_eq!(swing_amount(10.0), 1.0);
        let mut last = 0.0;
        for i in 1..=20 {
            let k = swing_amount(SWING_S * i as f32 / 20.0);
            assert!(k >= last);
            last = k;
        }
    }
}

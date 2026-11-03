//! People strolling round Hồ Gươm, from the delivered passers-by (`people/nguoi_di_duong_0N.glb`),
//! walking the lakeside promenade both ways with the library's walk. They are scenery: the raid's
//! rules don't see them. Without delivered passers-by nobody walks.

use bevy::prelude::*;

use crate::actors::{self, AnimLib, Move, Person};
use crate::hq::{Hq, HqState};

/// The lakeside loop and the length along it of each of its points.
#[derive(Resource, Default)]
pub struct Promenade {
    pts: Vec<Vec2>,
    cum: Vec<f32>,
    len: f32,
}

impl Promenade {
    pub fn new(pts: Vec<Vec2>) -> Promenade {
        let mut cum = Vec::with_capacity(pts.len() + 1);
        let mut len = 0.0;
        for i in 0..pts.len() {
            cum.push(len);
            if let (Some(a), Some(b)) = (pts.get(i), pts.get((i + 1) % pts.len())) {
                len += a.distance(*b);
            }
        }
        cum.push(len);
        Promenade { pts, cum, len }
    }
    /// The point `s` metres along the loop, and the direction of travel there.
    pub fn at(&self, s: f32) -> Option<(Vec2, Vec2)> {
        if self.pts.len() < 2 || self.len <= 0.0 {
            return None;
        }
        let s = s.rem_euclid(self.len);
        let i = self.cum.partition_point(|c| *c <= s).saturating_sub(1).min(self.pts.len() - 1);
        let (a, b) = (*self.pts.get(i)?, *self.pts.get((i + 1) % self.pts.len())?);
        let seg = a.distance(b).max(1e-4);
        let t = ((s - self.cum.get(i).copied().unwrap_or(0.0)) / seg).clamp(0.0, 1.0);
        Some((a.lerp(b, t), (b - a) / seg))
    }
}

/// A passer-by: where on the loop, which way and how fast.
#[derive(Component)]
pub struct Walker {
    s: f32,
    speed: f32,
}

/// How many people stroll, and how far from the camera they are drawn.
const PEOPLE: usize = 40;
const SEEN: f32 = 140.0;

/// Sends the passers-by out once the delivered art is known.
pub fn spawn(mut commands: Commands, hq: Res<Hq>, lib: Res<AnimLib>, assets: Res<AssetServer>, path: Res<Promenade>, mut done: Local<bool>) {
    // The moves must be ready before anyone appears, or they would stand in their T-pose.
    if *done || hq.state == HqState::Waiting || lib.graph.is_none() {
        return;
    }
    *done = true;
    const WHO: [&str; 3] = ["nguoi_di_duong", "nguoi_di_duong_02", "nguoi_di_duong_03"];
    if hq.get("people/nguoi_di_duong_01.glb").is_none() || path.len <= 0.0 {
        return;
    }
    for i in 0..PEOPLE {
        let s = path.len * i as f32 / PEOPLE as f32 + (i % 3) as f32 * 2.0;
        let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        let Some((p, _)) = path.at(s) else { continue };
        let who = WHO[i % WHO.len()];
        let e = actors::spawn_person(&mut commands, &assets, &hq, who, "", Vec3::new(p.x, 0.0, p.y));
        commands.entity(e).insert(Walker { s, speed: dir * (1.05 + (i % 5) as f32 * 0.08) });
    }
}

/// Walks everyone along the loop, and only draws those near the camera.
pub fn walk(
    time: Res<Time>,
    path: Res<Promenade>,
    lib: Res<AnimLib>,
    cam: Query<&GlobalTransform, With<Camera3d>>,
    mut walkers: Query<(&mut Walker, &mut Transform, &mut Visibility, &mut Person)>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let dt = time.delta_secs().min(0.05);
    let c = cam.single().map(|c| c.translation()).unwrap_or_default();
    for (mut w, mut t, mut v, mut person) in &mut walkers {
        w.s += w.speed * dt;
        let Some((p, d)) = path.at(w.s) else { continue };
        let pos = Vec3::new(p.x, 0.0, p.y);
        t.translation = pos;
        let ahead = Vec3::new(d.x, 0.0, d.y) * w.speed.signum();
        t.rotation = Quat::from_rotation_arc(Vec3::Z, ahead.normalize_or(Vec3::Z));
        let near = pos.distance(c) < SEEN;
        let want = if near { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
        if near {
            actors::set_move(&mut person, Move::Walk, &lib, &mut players);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_round_the_loop() {
        let p = Promenade::new(vec![Vec2::ZERO, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0), Vec2::new(0.0, 10.0)]);
        assert!((p.len - 40.0).abs() < 1e-4);
        let (a, d) = p.at(15.0).unwrap();
        assert!((a - Vec2::new(10.0, 5.0)).length() < 1e-4 && (d - Vec2::Y).length() < 1e-4);
        // Past the end it comes round again; backwards works too.
        assert!((p.at(45.0).unwrap().0 - Vec2::new(5.0, 0.0)).length() < 1e-4);
        assert!((p.at(-5.0).unwrap().0 - Vec2::new(0.0, 5.0)).length() < 1e-4);
        assert!(Promenade::new(vec![Vec2::ZERO]).at(1.0).is_none());
    }
}

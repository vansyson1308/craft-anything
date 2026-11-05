//! Night. The first chapter, "Đêm mất Bơ", happens after dark, so it is played at night: a low
//! blue moon, the street lamps and the Hàng Mã lanterns really lighting the street around them,
//! wet roads that catch their light, windows glowing, and a torch in Minh's hand. Everything else
//! stays in the late afternoon. `?time=night` or `?time=day` on the page's address picks one for
//! the whole visit (for reviewing art at night, with `?look=…` too).
//!
//! The scene uses only the art that is already there: the light comes from where the delivered
//! lamps and the lanterns stand.

use std::collections::HashMap;

use bevy::camera::Exposure;
use bevy::prelude::*;

use crate::render::{self, Sun};

/// Whether it is night now (set by the game each frame from the mission and the address).
#[derive(Resource, Default)]
pub struct Sky {
    pub night: bool,
    /// Where the player is (or the camera looks): the lights nearest it are the ones switched on.
    pub focus: Vec3,
}

/// What the address asks for: `Some(true)` for `?time=night`, `Some(false)` for `?time=day`.
pub fn from_page() -> Option<bool> {
    match render::page_query("time").as_str() {
        "night" | "dem" => Some(true),
        "day" | "ngay" => Some(false),
        _ => None,
    }
}

/// Something that gives light at night.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glow {
    pub at: Vec3,
    pub color: Color,
    /// Luminous power in lumens.
    pub lumens: f32,
    /// How far the light reaches, in metres.
    pub range: f32,
    /// The delivered model this light belongs to: it is lit only once that model is there, so no
    /// light hangs in the air where a stand-in or nothing stands. Empty for lights that are always
    /// there.
    pub file: &'static str,
}

/// The street lamp along the main roads, and the cast-iron lamps round Hồ Gươm.
pub const STREET_LAMP: &str = "street/den_duong_01.glb";
pub const LAKE_LAMP: &str = "street/den_ven_ho.glb";
/// Shop light needs the delivered shopfronts to come from.
pub const SHOPFRONT: &str = "buildings/mat_tien_tret_01.glb";

impl Glow {
    /// A street lamp's head, from where its model stands (the arm reaches over the road along +Z).
    pub fn lamp(foot: &Transform) -> Glow {
        Glow { at: foot.transform_point(Vec3::new(0.0, 4.45, 1.15)), color: Color::srgb(1.0, 0.8, 0.55), lumens: 20_000.0, range: 22.0, file: STREET_LAMP }
    }

    /// A lamp on the path round the lake, its lantern on top of the post (ASSETS.md §12).
    pub fn lake_lamp(foot: &Transform) -> Glow {
        Glow { at: foot.transform_point(Vec3::new(0.0, 3.5, 0.0)), color: Color::srgb(1.0, 0.85, 0.62), lumens: 9_000.0, range: 16.0, file: LAKE_LAMP }
    }

    /// Light from an open shop spilling onto the pavement, for about one ground-floor shopfront in
    /// three (the same ones every time). It comes on once the shopfront modules are delivered.
    pub fn shop(slot: &crate::facade::Slot) -> Option<Glow> {
        if slot.storey != crate::facade::Storey::Ground || !slot.seed.is_multiple_of(3) {
            return None;
        }
        let t = slot.transform();
        let at = t.transform_point(Vec3::new(crate::facade::BAY_W / 2.0, 2.6, 0.7));
        Some(Glow { at, color: Color::srgb(1.0, 0.84, 0.6), lumens: 7_000.0, range: 11.0, file: SHOPFRONT })
    }

    /// A string of Hàng Mã lanterns.
    pub fn lantern(at: Vec3, red: bool) -> Glow {
        let color = if red { Color::srgb(1.0, 0.25, 0.15) } else { Color::srgb(1.0, 0.7, 0.3) };
        Glow { at, color, lumens: 5_000.0, range: 9.0, file: "" }
    }
}

/// Every place that gives light at night, from the map.
#[derive(Resource, Default)]
pub struct Glows(pub Vec<Glow>);

/// How many of them are lit at once: the nearest ones. Point lights cost on every pixel they
/// reach, and WebGL2 can only hold so many.
const POOL: usize = 20;

#[derive(Component)]
pub struct Pooled;

/// The torch in Minh's hand.
#[derive(Component)]
pub struct Torch;

/// The light settings of the night.
const MOON_LUX: f32 = 3.0;
/// Exposure at night: lamps light the street to a few dozen lux, the moon to a few.
const NIGHT_EV100: f32 = 3.5;
/// Wet roads: roughness is multiplied by this.
const WET: f32 = 0.4;

/// What day looked like, to put back in the morning.
#[derive(Default)]
pub struct Day {
    sun: Option<(DirectionalLight, Transform)>,
    ambient: Option<GlobalAmbientLight>,
    clear: Option<ClearColor>,
    fog: Option<DistanceFog>,
    exposure: Option<Option<Exposure>>,
}

fn moon() -> Transform {
    // High in the south-east, so streets get light from above and the lake a long reflection.
    Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.7, -0.95, 0.0))
}

/// The camera, its fog and its exposure.
type CameraParts<'a> = (Entity, Option<&'a mut DistanceFog>, Option<&'a Exposure>);

/// Switches the sun, sky and camera between day and night when the sky changes.
#[allow(clippy::too_many_arguments)]
pub fn apply(
    sky: Res<Sky>,
    mut shown: Local<Option<bool>>,
    mut day: Local<Day>,
    mut sun: Query<(&mut DirectionalLight, &mut Transform), With<Sun>>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut clear: Option<ResMut<ClearColor>>,
    mut cams: Query<CameraParts, With<Camera3d>>,
    mut commands: Commands,
) {
    if *shown == Some(sky.night) {
        return;
    }
    // Nothing to change on the first frame if it starts by day.
    if shown.is_none() && !sky.night {
        *shown = Some(false);
        return;
    }
    *shown = Some(sky.night);
    if sky.night {
        if let Ok((mut l, mut t)) = sun.single_mut() {
            day.sun = Some((*l, *t));
            l.illuminance = MOON_LUX;
            l.color = Color::srgb(0.62, 0.72, 1.0);
            *t = moon();
        }
        day.ambient = Some(ambient.clone());
        ambient.color = Color::srgb(0.45, 0.55, 0.9);
        ambient.brightness = if ambient.brightness > 0.0 { 1.6 } else { 0.0 };
        if let Some(c) = clear.as_mut() {
            day.clear = Some((**c).clone());
            c.0 = Color::srgb(0.012, 0.016, 0.035);
        }
        for (e, fog, exposure) in &mut cams {
            day.exposure = Some(exposure.copied());
            commands.entity(e).insert(Exposure { ev100: NIGHT_EV100 });
            if let Some(mut f) = fog {
                day.fog = Some(f.clone());
                f.color = Color::srgb(0.02, 0.025, 0.05);
                f.falloff = FogFalloff::Linear { start: 40.0, end: 320.0 };
            }
        }
    } else {
        if let (Ok((mut l, mut t)), Some((dl, dt))) = (sun.single_mut(), day.sun.take()) {
            *l = dl;
            *t = dt;
        }
        if let Some(a) = day.ambient.take() {
            *ambient = a;
        }
        if let (Some(c), Some(dc)) = (clear.as_mut(), day.clear.take()) {
            **c = dc;
        }
        for (e, fog, _) in &mut cams {
            match day.exposure.take() {
                Some(Some(x)) => {
                    commands.entity(e).insert(x);
                }
                Some(None) => {
                    commands.entity(e).remove::<Exposure>();
                }
                None => {}
            }
            if let (Some(mut f), Some(df)) = (fog, day.fog.take()) {
                *f = df;
            }
        }
    }
}

/// Lights the glows nearest the focus (a pool of point lights moved from lamp to lamp), and none
/// by day.
pub fn pool(
    sky: Res<Sky>,
    glows: Res<Glows>,
    hq: Res<crate::hq::Hq>,
    time: Res<Time>,
    mut since: Local<f32>,
    mut lights: Query<(Entity, &mut PointLight, &mut Transform, &mut Visibility), With<Pooled>>,
    mut commands: Commands,
) {
    if !sky.night {
        for (e, ..) in &lights {
            commands.entity(e).despawn();
        }
        return;
    }
    *since += time.delta_secs();
    let have = lights.iter().count();
    if have > 0 && *since < 0.25 {
        return;
    }
    *since = 0.0;
    let there: Vec<Glow> = glows.0.iter().filter(|g| g.file.is_empty() || hq.get(g.file).is_some()).copied().collect();
    let near = nearest(&there, sky.focus, POOL);
    if have == 0 {
        for _ in 0..POOL {
            commands.spawn((Pooled, PointLight { shadow_maps_enabled: false, ..default() }, Transform::default(), Visibility::Hidden));
        }
        return;
    }
    let mut it = near.iter();
    for (_, mut l, mut t, mut v) in &mut lights {
        match it.next().and_then(|&i| there.get(i)) {
            Some(g) => {
                l.intensity = g.lumens;
                l.range = g.range;
                l.color = g.color;
                t.translation = g.at;
                *v = Visibility::Visible;
            }
            None => *v = Visibility::Hidden,
        }
    }
}

/// The indices of the `n` glows nearest `to`, nearest first.
pub fn nearest(glows: &[Glow], to: Vec3, n: usize) -> Vec<usize> {
    let mut idx: Vec<(f32, usize)> = glows.iter().enumerate().map(|(i, g)| (g.at.distance_squared(to), i)).collect();
    if idx.len() > n && n > 0 {
        idx.select_nth_unstable_by(n - 1, |a, b| a.0.total_cmp(&b.0));
        idx.truncate(n);
    }
    idx.sort_by(|a, b| a.0.total_cmp(&b.0));
    idx.into_iter().map(|(_, i)| i).take(n).collect()
}

/// A torch for a character, held at the hand, pointing ahead and a little down.
pub fn torch(commands: &mut Commands, holder: Entity) {
    let light = commands
        .spawn((
            Torch,
            SpotLight {
                intensity: 30_000.0,
                range: 28.0,
                color: Color::srgb(1.0, 0.96, 0.88),
                inner_angle: 0.22,
                outer_angle: 0.5,
                shadow_maps_enabled: render::HIGH,
                ..default()
            },
            Transform::from_xyz(0.22, 1.15, 0.35).looking_to(Vec3::new(0.0, -0.22, 1.0), Vec3::Y),
        ))
        .id();
    commands.entity(holder).add_child(light);
}

/// The delivered torch model, held in the right hand.
const TORCH_FILE: &str = "people/den_pin.glb";
const HAND: &str = "hand_r";

/// The torch model in a hand, and the light it belongs to.
#[derive(Component)]
pub struct HandTorch(Entity);

/// Torch lights whose model is not in the hand yet.
type Unheld = (With<Torch>, Without<Held>);

/// The torch light has its model in the hand.
#[derive(Component)]
pub struct Held;

/// Puts the delivered torch model (when there is one) in the hand of whoever carries a torch light,
/// and takes it away with the light.
pub fn torch_prop(
    mut commands: Commands,
    hq: Res<crate::hq::Hq>,
    torches: Query<(Entity, &ChildOf), Unheld>,
    lights: Query<(), With<Torch>>,
    props: Query<(Entity, &HandTorch)>,
    kids: Query<&Children>,
    names: Query<&Name>,
) {
    for (e, t) in &props {
        if lights.get(t.0).is_err() {
            commands.entity(e).despawn();
        }
    }
    if hq.get(TORCH_FILE).is_none() {
        return;
    }
    for (light, holder) in &torches {
        let Some(hand) = kids.iter_descendants(holder.parent()).find(|k| names.get(*k).is_ok_and(|n| n.as_str() == HAND)) else { continue };
        if let Some(m) = crate::hq::prop(&mut commands, &hq, &[TORCH_FILE], Transform::IDENTITY) {
            commands.entity(m).insert(HandTorch(light));
            commands.entity(hand).add_child(m);
            commands.entity(light).insert(Held);
        }
    }
}

/// Road materials get wet at night (their roughness drops, so lamps shine in them) and dry by day.
/// It is checked every frame because a delivered ground material can arrive (and set its own
/// roughness) at any time.
pub fn wet(
    sky: Res<Sky>,
    m: Option<Res<crate::world::Mats>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut dry: Local<HashMap<AssetId<StandardMaterial>, (f32, f32)>>,
) {
    let Some(m) = m else { return };
    for h in [&m.asphalt, &m.paving, &m.path_paving] {
        let id = h.id();
        let Some(now) = mats.get(id).map(|x| x.perceptual_roughness) else { continue };
        if sky.night {
            let wet_now = dry.get(&id).map(|d| d.1);
            if wet_now != Some(now) {
                dry.insert(id, (now, now * WET));
                if let Some(mut x) = mats.get_mut(id) {
                    x.perceptual_roughness = now * WET;
                }
            }
        } else if let Some((d, w)) = dry.remove(&id)
            && w == now
            && let Some(mut x) = mats.get_mut(id)
        {
            x.perceptual_roughness = d;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32) -> Glow {
        Glow::lantern(Vec3::new(x, 0.0, 0.0), true)
    }

    #[test]
    fn picks_the_nearest_lights() {
        let glows: Vec<Glow> = [50.0, -3.0, 10.0, 1.0, -40.0].into_iter().map(at).collect();
        assert_eq!(nearest(&glows, Vec3::ZERO, 3), vec![3, 1, 2]);
        assert_eq!(nearest(&glows, Vec3::ZERO, 9).len(), 5);
        assert!(nearest(&glows, Vec3::ZERO, 0).is_empty());
        assert!(nearest(&[], Vec3::ZERO, 4).is_empty());
    }

    #[test]
    fn some_shops_light_the_pavement() {
        let slot = |seed, storey| crate::facade::Slot { corner: Vec3::ZERO, right: Vec2::X, stretch: 1.0, storey, seed };
        let lit = (0..300).filter_map(|s| Glow::shop(&slot(s, crate::facade::Storey::Ground))).collect::<Vec<_>>();
        assert_eq!(lit.len(), 100);
        // In front of the shop (out of the wall is +z here), above head height, within the bay.
        assert!(lit.iter().all(|g| g.at.z > 0.3 && g.at.y > 2.0 && (0.0..=4.0).contains(&g.at.x)));
        assert!(Glow::shop(&slot(3, crate::facade::Storey::Upper)).is_none());
    }

    #[test]
    fn lamp_light_hangs_over_the_road() {
        // A lamp on the north side of an east-west road, its arm (+Z) turned south over the road.
        let foot = Transform::from_xyz(10.0, 0.0, -5.0);
        let g = Glow::lamp(&foot);
        assert!(g.at.y > 4.0 && g.at.z > -5.0);
    }
}

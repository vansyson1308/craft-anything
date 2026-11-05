//! Showing the player the way and the danger, so nobody wanders lost and stealth can be read:
//!
//! - a beam of light rising where to go next (gold for a dog, a clue or a boss, green for the way
//!   out), lighting the ground round it, seen over the roofs; the way Nintendo games lead with
//!   light and landmarks rather than arrows;
//! - a line at the top of the screen naming it, with its distance and direction;
//! - each thief's field of view laid on the ground, pale while calm, orange when wary, red when
//!   hunting, with "?" or "!" over its head;
//! - short tips the first time something happens (the first thief near, the first cage, being
//!   seen…).
//!
//! The game fills `Guide` every frame from the raid; this module only draws it.

use bevy::prelude::*;
use bevy::text::FontSize;

/// What the beam marks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Something to get: a caged dog, a clue, a boss to tie up.
    Get,
    /// The way out.
    Out,
}

/// How a thief feels, for colouring its view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Calm,
    Wary,
    Hunting,
}

/// A thief's field of view on the ground.
#[derive(Clone, Copy, Debug)]
pub struct Cone {
    pub at: Vec3,
    /// Turns the cone's +Z to where the thief looks.
    pub turn: Quat,
    /// How far it sees (metres) and half the width of its view (radians).
    pub range: f32,
    pub half: f32,
    pub mood: Mood,
}

/// What to show this frame (filled by the game).
#[derive(Resource, Default)]
pub struct Guide {
    pub on: bool,
    pub goal: Option<(Vec3, Mark, String)>,
    pub cones: Vec<Cone>,
    pub tip: Option<String>,
}

#[derive(Component)]
pub struct Beam;
#[derive(Component)]
pub struct BeamLight;
#[derive(Component)]
pub struct ConeView(usize);
#[derive(Component)]
pub struct Mood3d(usize);
#[derive(Component)]
pub struct Compass;
#[derive(Component)]
pub struct Tip;

/// How many thieves' views are drawn at once.
const CONES: usize = 16;
/// Segments of a view's arc.
const ARC: usize = 20;

#[derive(Resource)]
pub struct Looks {
    get: Handle<StandardMaterial>,
    out: Handle<StandardMaterial>,
    calm: Handle<StandardMaterial>,
    wary: Handle<StandardMaterial>,
    hunting: Handle<StandardMaterial>,
    /// Fan meshes by width (half angle in hundredths of a radian).
    fans: Vec<(i32, Handle<Mesh>)>,
}

fn see_through(c: Color, alpha: f32, add: bool) -> StandardMaterial {
    StandardMaterial {
        base_color: c.with_alpha(alpha),
        emissive: c.to_linear() * 1.5,
        unlit: true,
        alpha_mode: if add { AlphaMode::Add } else { AlphaMode::Blend },
        // The beam must stand out over the roofs from far away, through the haze.
        fog_enabled: !add,
        double_sided: true,
        cull_mode: None,
        ..default()
    }
}

/// A flat fan on the ground: radius 1, from −`half` to +`half` round +Z, fading toward its rim.
pub fn fan(half: f32) -> Mesh {
    let mut pos = vec![[0.0, 0.0, 0.0]];
    let mut col = vec![[1.0, 1.0, 1.0, 0.9]];
    for i in 0..=ARC {
        let a = -half + 2.0 * half * i as f32 / ARC as f32;
        pos.push([a.sin(), 0.0, a.cos()]);
        col.push([1.0, 1.0, 1.0, 0.25]);
    }
    let mut idx = Vec::with_capacity(ARC * 3);
    for i in 1..=ARC as u32 {
        idx.extend_from_slice(&[0, i + 1, i]);
    }
    let n = pos.len();
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; n])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, col)
        .with_inserted_indices(bevy::mesh::Indices::U32(idx))
}

pub fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>, assets: Res<AssetServer>) {
    let looks = Looks {
        get: mats.add(see_through(Color::srgb(1.0, 0.78, 0.3), 0.22, true)),
        out: mats.add(see_through(Color::srgb(0.35, 1.0, 0.55), 0.22, true)),
        calm: mats.add(see_through(Color::srgb(1.0, 0.95, 0.6), 0.16, false)),
        wary: mats.add(see_through(Color::srgb(1.0, 0.6, 0.15), 0.28, false)),
        hunting: mats.add(see_through(Color::srgb(1.0, 0.15, 0.1), 0.34, false)),
        fans: Vec::new(),
    };
    // The beam: a tall soft column with a brighter core, and a light at its foot.
    let column = meshes.add(Cylinder::new(0.7, 60.0));
    let core = meshes.add(Cylinder::new(0.18, 60.0));
    commands.spawn((Beam, Transform::default(), Visibility::Hidden)).with_children(|b| {
        b.spawn((Mesh3d(column), MeshMaterial3d(looks.get.clone()), Transform::from_xyz(0.0, 30.0, 0.0)));
        b.spawn((Mesh3d(core), MeshMaterial3d(looks.get.clone()), Transform::from_xyz(0.0, 30.0, 0.0)));
        b.spawn((BeamLight, PointLight { intensity: 9_000.0, range: 10.0, shadow_maps_enabled: false, ..default() }, Transform::from_xyz(0.0, 2.0, 0.0)));
    });
    for i in 0..CONES {
        commands.spawn((ConeView(i), Transform::default(), Visibility::Hidden));
    }
    let font: Handle<Font> = assets.load("fonts/DejaVuSans-Bold.ttf");
    let text = |s: f32, c: Color| (Text::new(""), TextFont { font: font.clone().into(), font_size: FontSize::Px(s), ..default() }, TextColor(c));
    commands.spawn((
        Compass,
        text(17.0, Color::srgb(1.0, 0.92, 0.6)),
        TextLayout::justify(Justify::Center),
        Node { position_type: PositionType::Absolute, top: Val::Px(14.0), left: Val::Percent(30.0), width: Val::Percent(40.0), ..default() },
        Visibility::Hidden,
    ));
    commands.spawn((
        Tip,
        text(16.0, Color::WHITE),
        TextLayout::justify(Justify::Center),
        BackgroundColor(Color::srgba(0.05, 0.06, 0.09, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(46.0),
            left: Val::Percent(25.0),
            width: Val::Percent(50.0),
            padding: UiRect::axes(Val::Px(14.0), Val::Px(9.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        Visibility::Hidden,
    ));
    for i in 0..CONES {
        commands.spawn((Mood3d(i), text(26.0, Color::srgb(1.0, 0.8, 0.2)), Node { position_type: PositionType::Absolute, ..default() }, Visibility::Hidden));
    }
    commands.insert_resource(looks);
}

/// The half angle a fan mesh is kept under.
fn fan_key(half: f32) -> i32 {
    (half * 100.0).round() as i32
}

/// The beam's place, whether it shows, and its column and core.
type BeamParts<'a> = (&'a mut Transform, &'a mut Visibility, &'a Children);

#[allow(clippy::too_many_arguments)]
pub fn draw(
    guide: Res<Guide>,
    time: Res<Time>,
    mut looks: ResMut<Looks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut beam: Query<BeamParts, (With<Beam>, Without<ConeView>)>,
    mut beam_parts: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ConeView>>,
    mut beam_light: Query<&mut PointLight, With<BeamLight>>,
    mut cones: Query<(Entity, &ConeView, &mut Transform, &mut Visibility), Without<Beam>>,
    mut commands: Commands,
) {
    if let Ok((mut t, mut v, kids)) = beam.single_mut() {
        match (&guide.goal, guide.on) {
            (Some((at, mark, _)), true) => {
                *v = Visibility::Inherited;
                let pulse = 1.0 + (time.elapsed_secs() * 2.2).sin() * 0.08;
                *t = Transform::from_translation(*at).with_scale(Vec3::new(pulse, 1.0, pulse));
                let m = if *mark == Mark::Out { looks.out.clone() } else { looks.get.clone() };
                for k in kids.iter() {
                    if let Ok(mut mm) = beam_parts.get_mut(k) {
                        mm.0 = m.clone();
                    }
                }
                if let Ok(mut l) = beam_light.single_mut() {
                    l.color = if *mark == Mark::Out { Color::srgb(0.4, 1.0, 0.6) } else { Color::srgb(1.0, 0.8, 0.4) };
                }
            }
            _ => *v = Visibility::Hidden,
        }
    }
    for (e, ConeView(i), mut t, mut v) in &mut cones {
        let Some(c) = guide.cones.get(*i).filter(|_| guide.on) else {
            *v = Visibility::Hidden;
            continue;
        };
        let key = fan_key(c.half);
        let mesh = match looks.fans.iter().find(|f| f.0 == key) {
            Some(f) => f.1.clone(),
            None => {
                let h = meshes.add(fan(key as f32 / 100.0));
                looks.fans.push((key, h.clone()));
                h
            }
        };
        let mat = match c.mood {
            Mood::Calm => looks.calm.clone(),
            Mood::Wary => looks.wary.clone(),
            Mood::Hunting => looks.hunting.clone(),
        };
        commands.entity(e).insert((Mesh3d(mesh), MeshMaterial3d(mat)));
        *t = Transform::from_translation(c.at + Vec3::Y * 0.07).with_rotation(c.turn).with_scale(Vec3::new(c.range, 1.0, c.range));
        *v = Visibility::Inherited;
    }
}

/// The compass line, the tip card and the marks over thieves' heads.
#[allow(clippy::type_complexity)]
pub fn hud(
    guide: Res<Guide>,
    camera: Query<(&Camera, &GlobalTransform), With<crate::MainCamera>>,
    mut compass: Query<(&mut Text, &mut Visibility), (With<Compass>, Without<Tip>, Without<Mood3d>)>,
    mut tip: Query<(&mut Text, &mut Visibility), (With<Tip>, Without<Compass>, Without<Mood3d>)>,
    mut marks: Query<(&Mood3d, &mut Text, &mut TextColor, &mut Node, &mut Visibility), (Without<Compass>, Without<Tip>)>,
) {
    let Ok((cam, cam_t)) = camera.single() else { return };
    if let Ok((mut text, mut v)) = compass.single_mut() {
        match (&guide.goal, guide.on) {
            (Some((at, _, label)), true) => {
                let eye = cam_t.translation();
                let to = Vec2::new(at.x - eye.x, at.z - eye.z);
                let fwd = cam_t.forward();
                let ahead = Vec2::new(fwd.x, fwd.z);
                // Turn from straight ahead, positive to the left on screen (the camera's right is
                // a clockwise turn in this x–z plane).
                let a = -ahead.angle_to(to);
                text.0 = format!("{} {} · {} m", arrow(a), label, to.length().round() as i32);
                *v = Visibility::Inherited;
            }
            _ => *v = Visibility::Hidden,
        }
    }
    if let Ok((mut text, mut v)) = tip.single_mut() {
        match (&guide.tip, guide.on) {
            (Some(t), true) => {
                text.0.clone_from(t);
                *v = Visibility::Inherited;
            }
            _ => *v = Visibility::Hidden,
        }
    }
    for (Mood3d(i), mut text, mut colour, mut node, mut v) in &mut marks {
        let c = guide.cones.get(*i).filter(|c| guide.on && c.mood != Mood::Calm);
        let Some(c) = c else {
            *v = Visibility::Hidden;
            continue;
        };
        match cam.world_to_viewport(cam_t, c.at + Vec3::Y * 2.3) {
            Ok(p) => {
                let hunting = c.mood == Mood::Hunting;
                text.0 = if hunting { "!".into() } else { "?".into() };
                colour.0 = if hunting { Color::srgb(1.0, 0.25, 0.2) } else { Color::srgb(1.0, 0.75, 0.2) };
                node.left = Val::Px(p.x - 6.0);
                node.top = Val::Px(p.y - 16.0);
                *v = Visibility::Inherited;
            }
            Err(_) => *v = Visibility::Hidden,
        }
    }
}

/// An arrow for a turn of `a` radians from straight ahead (positive: to the left on screen).
pub fn arrow(a: f32) -> &'static str {
    let deg = a.to_degrees();
    let arrows = ["↑", "↖", "←", "↙", "↓", "↘", "→", "↗"];
    let i = (((deg + 22.5).rem_euclid(360.0)) / 45.0) as usize % 8;
    arrows.get(i).copied().unwrap_or("↑")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrows_point_the_right_way() {
        assert_eq!(arrow(0.0), "↑");
        assert_eq!(arrow(std::f32::consts::PI), "↓");
        assert_eq!(arrow(-std::f32::consts::PI), "↓");
        assert_eq!(arrow(std::f32::consts::FRAC_PI_2), "←");
        assert_eq!(arrow(-std::f32::consts::FRAC_PI_2), "→");
        assert_eq!(arrow(0.3), "↑");
        assert_eq!(arrow(0.6), "↖");
    }

    #[test]
    fn a_fan_spans_its_width() {
        let m = fan(0.6);
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!("no positions") };
        assert_eq!(p.len(), ARC + 2);
        let first = p[1];
        let last = p[ARC + 1];
        assert!((first[0] - (-0.6f32).sin()).abs() < 1e-5 && (last[0] - 0.6f32.sin()).abs() < 1e-5);
        assert!(p.iter().all(|v| (v[0] * v[0] + v[2] * v[2]).sqrt() <= 1.0 + 1e-5));
    }
}

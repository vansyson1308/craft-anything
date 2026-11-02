//! Hanoi City: the dog-rescue extraction game in 3D on the real streets of Hoàn Kiếm.
//!
//! The rules (thieves who see, hear and chase, cages, traffic, the train, getting out) are the
//! tested `hanoi-rescue` core, running on a map rasterised from OpenStreetMap; this crate draws
//! that world in 3D with Bevy and turns touches, the mouse and the keyboard into its controls.

mod actors;
mod world;

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::asset::AssetMetaCheck;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui_widgets::{Activate, Button};
use bevy::window::PrimaryWindow;
use hanoi_rescue::city::City;
use hanoi_rescue::content::{self, CHAPTERS, UPGRADES};
use hanoi_rescue::game::Hud;
use hanoi_rescue::geom::V2;
use hanoi_rescue::map::Map;
use hanoi_rescue::meta::{Earned, Profile, SAVE_KEY};
use hanoi_rescue::raid::{Context, Controls, DogState, Mission, Outcome, PickupKind, Raid, RaidSetup, Report, TState, ThiefKind};

use actors::{AnimLib, Dog, Move, Person, Props};

const CITY_JSON: &str = include_str!("../assets/hoankiem.json");
const SIGNS_JSON: &str = include_str!("../assets/signs.json");
const TILE_M: f32 = 2.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.32, 0.36, 0.52)))
        .insert_resource(GlobalAmbientLight { color: Color::srgb(0.62, 0.66, 0.9), brightness: 420.0, ..default() })
        .add_plugins(DefaultPlugins.set(AssetPlugin { meta_check: AssetMetaCheck::Never, ..default() }).set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hanoi City: Giải Cứu Cún".into(),
                canvas: Some("#game".into()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<Clicked>()
        .add_systems(Startup, setup)
        .add_systems(Update, ((actors::build_graphs, play, hud, menus).chain(), world::cull_far))
        .run();
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Loading,
    Title,
    Story(usize, usize),
    Brief(Mission),
    Shop,
    Playing,
    Paused,
    Result,
}

/// Story scenes: the intro, then each chapter's ending.
fn story_lines(which: usize) -> &'static [content::Line] {
    if which == 0 { content::INTRO } else { CHAPTERS.get(which - 1).map_or(&[], |c| c.after) }
}

#[derive(Resource)]
struct Game {
    map: Map,
    profile: Profile,
    raid: Option<Raid>,
    screen: Screen,
    shown: Option<Screen>,
    result: Option<(Report, Earned)>,
    // What the raid looks like in the world.
    root: Option<Entity>,
    player: Option<Entity>,
    thieves: Vec<Entity>,
    dogs: Vec<Entity>,
    cages: Vec<Entity>,
    pickups: Vec<Entity>,
    bikes: Vec<Entity>,
    train: Vec<Entity>,
    exits: Vec<Entity>,
    // The camera orbits behind the player.
    cam_yaw: f32,
    cam_target: Vec3,
    drag: Option<(u64, Vec2)>,
    joy: Option<(u64, Vec2, Vec2)>,
    t: f32,
    loaded_t: f32,
}

#[derive(Component)]
struct MainCamera;
#[derive(Component)]
struct Sun;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum Ui {
    Hp,
    Stamina,
    Timer,
    Info,
    Objective,
    Banner,
    Extract,
    Attack,
    Throw,
    Roll,
    Context,
    Pause,
    JoyBase,
    JoyKnob,
    Minimap,
    MiniDot,
    Hint,
    Loading,
}

#[derive(Component)]
struct Label3d(Vec3);
#[derive(Component)]
struct MenuRoot;
#[derive(Component, Clone, Copy)]
enum MenuButton {
    Story,
    Free,
    Shop,
    Go,
    Back,
    Buy(usize),
    Next,
    Skip,
    Resume,
    Abandon,
    Done,
}

fn world_pos(p: V2) -> Vec3 {
    Vec3::new(p.x * TILE_M, 0.0, p.y * TILE_M)
}

/// Yaw that turns a glTF model (facing +Z) toward a direction in the game's grid.
fn yaw_of(angle: f32) -> Quat {
    let (dx, dz) = (angle.cos(), angle.sin());
    Quat::from_rotation_y(dx.atan2(dz))
}

fn load_profile() -> Profile {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()).and_then(|s| s.get_item(SAVE_KEY).ok().flatten()) {
            return Profile::load(&s).unwrap_or_default();
        }
    }
    Profile::default()
}

/// The page's address ends in #peek (for screenshots and testing).
fn peek_requested() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window().and_then(|w| w.location().hash().ok()).is_some_and(|h| h == "#peek");
    }
    #[allow(unreachable_code)]
    false
}

fn save_profile(p: &Profile) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = s.set_item(SAVE_KEY, &p.to_json());
        }
    }
    let _ = p;
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let city = City::from_json(CITY_JSON).unwrap_or_default();
    let map = Map::from_city(&city, TILE_M);
    let lake = map.labels.iter().find(|l| l.text == "HỒ GƯƠM").map(|l| world_pos(l.at)).unwrap_or(Vec3::new(1000.0, 0.0, 1000.0));
    let signs: Vec<(String, [i32; 2])> = serde_json::from_str::<Vec<serde_json::Value>>(SIGNS_JSON)
        .unwrap_or_default()
        .iter()
        .filter_map(|v| {
            let name = v.get("name")?.as_str()?.to_string();
            let at = v.get("at")?.as_array()?;
            Some((name, [at.first()?.as_i64()? as i32, at.get(1)?.as_i64()? as i32]))
        })
        .collect();
    let m = world::materials(&assets, &mut mats);
    world::spawn_city(&mut commands, &mut meshes, &m, &city, Vec2::new(lake.x, lake.z), &signs);

    // Dusk: a low warm sun, blue sky light, haze in the distance.
    commands.spawn((
        Sun,
        DirectionalLight { illuminance: 7000.0, color: Color::srgb(1.0, 0.78, 0.55), shadow_maps_enabled: true, ..default() },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 0.9, -0.42, 0.0)),
        CascadeShadowConfigBuilder { num_cascades: 1, first_cascade_far_bound: 45.0, maximum_distance: 60.0, ..default() }.build(),
    ));
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { fov: 0.95, ..default() }),
        Transform::from_translation(lake + Vec3::new(0.0, 120.0, 220.0)).looking_at(lake, Vec3::Y),
        DistanceFog { color: Color::srgb(0.36, 0.38, 0.52), falloff: FogFalloff::Linear { start: 90.0, end: 420.0 }, ..default() },
    ));

    commands.insert_resource(AnimLib { gltf: assets.load("anim.glb"), dog_gltf: assets.load("dog.glb"), graph: None, dog_graph: None });
    commands.insert_resource(actors::make_props(&mut meshes, &mut mats));

    // The minimap, from the map's tiles, scaled down.
    let rgba = map.minimap_rgba();
    let (mw, mh) = ((map.w / 4).max(1) as u32, (map.h / 4).max(1) as u32);
    let mut small = vec![0u8; (mw * mh * 4) as usize];
    for y in 0..mh {
        for x in 0..mw {
            let src = (((y * 4) * map.w as u32 + x * 4) * 4) as usize;
            let dst = ((y * mw + x) * 4) as usize;
            if let (Some(s), Some(d)) = (rgba.get(src..src + 4), small.get_mut(dst..dst + 4)) {
                d.copy_from_slice(s);
            }
        }
    }
    let mini = images.add(Image::new(
        bevy::render::render_resource::Extent3d { width: mw, height: mh, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        small,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    ));

    let font = assets.load("fonts/DejaVuSans-Bold.ttf");
    spawn_hud(&mut commands, font.clone(), mini);
    for l in map.labels.iter().filter(|l| l.big) {
        commands.spawn((
            Label3d(world_pos(l.at) + Vec3::Y * 8.0),
            Text::new(l.text.clone()),
            TextFont { font: font.clone().into(), font_size: FontSize::Px(17.0), ..default() },
            TextColor(Color::srgb(1.0, 0.95, 0.8)),
            Node { position_type: PositionType::Absolute, ..default() },
            Visibility::Hidden,
        ));
    }
    let profile = load_profile();
    commands.insert_resource(UiFont(font));
    commands.insert_resource(Game {
        map,
        profile,
        raid: None,
        screen: Screen::Loading,
        shown: None,
        result: None,
        root: None,
        player: None,
        thieves: Vec::new(),
        dogs: Vec::new(),
        cages: Vec::new(),
        pickups: Vec::new(),
        bikes: Vec::new(),
        train: Vec::new(),
        exits: Vec::new(),
        cam_yaw: PI,
        cam_target: lake,
        drag: None,
        joy: None,
        t: 0.0,
        loaded_t: 0.0,
    });
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

/// The menu button pressed this frame.
#[derive(Resource, Default)]
struct Clicked(Option<MenuButton>);

fn text(font: &Handle<Font>, s: &str, size: f32, c: Color) -> (Text, TextFont, TextColor) {
    (Text::new(s), TextFont { font: font.clone().into(), font_size: FontSize::Px(size), ..default() }, TextColor(c))
}

fn abs(left: f32, top: f32, w: f32, h: f32) -> Node {
    Node { position_type: PositionType::Absolute, left: Val::Px(left), top: Val::Px(top), width: Val::Px(w), height: Val::Px(h), ..default() }
}

fn spawn_hud(commands: &mut Commands, font: Handle<Font>, mini: Handle<Image>) {
    let dark = Color::srgba(0.04, 0.05, 0.08, 0.72);
    commands.spawn((Ui::Hp, abs(12.0, 12.0, 200.0, 12.0), BackgroundColor(Color::srgb(0.13, 0.77, 0.37)), Visibility::Hidden));
    commands.spawn((Ui::Stamina, abs(12.0, 28.0, 200.0, 4.0), BackgroundColor(Color::srgb(0.58, 0.77, 0.99)), Visibility::Hidden));
    commands.spawn((Ui::Timer, text(&font, "", 20.0, Color::WHITE), abs(12.0, 38.0, 90.0, 26.0), Visibility::Hidden));
    commands.spawn((Ui::Info, text(&font, "", 12.0, Color::srgb(0.85, 0.87, 0.9)), abs(84.0, 44.0, 260.0, 18.0), Visibility::Hidden));
    commands.spawn((Ui::Objective, text(&font, "", 14.0, Color::srgb(1.0, 0.9, 0.55)), abs(12.0, 70.0, 300.0, 60.0), Visibility::Hidden));
    commands.spawn((Ui::Banner, text(&font, "", 18.0, Color::srgb(1.0, 0.9, 0.55)), abs(0.0, 0.0, 10.0, 30.0), Visibility::Hidden));
    commands.spawn((Ui::Extract, text(&font, "", 18.0, Color::srgb(0.65, 0.95, 0.8)), abs(0.0, 0.0, 10.0, 30.0), Visibility::Hidden));
    commands.spawn((Ui::Hint, text(&font, content::ui::KEYS, 12.0, Color::srgb(0.85, 0.87, 0.9)), abs(0.0, 0.0, 10.0, 20.0), Visibility::Hidden));
    commands.spawn((Ui::Loading, text(&font, "Đang dựng phố cổ Hà Nội…", 22.0, Color::srgb(1.0, 0.75, 0.2)), abs(0.0, 0.0, 10.0, 30.0)));
    commands.spawn((Ui::Minimap, ImageNode::new(mini), abs(0.0, 0.0, 120.0, 120.0), Visibility::Hidden));
    commands.spawn((Ui::MiniDot, BackgroundColor(Color::WHITE), Node { border_radius: BorderRadius::MAX, ..abs(0.0, 0.0, 7.0, 7.0) }, Visibility::Hidden));
    let round = |c: Color| (BackgroundColor(c), Visibility::Hidden);
    for (kind, label, c) in [
        (Ui::Attack, content::ui::ATTACK, Color::srgba(0.86, 0.15, 0.15, 0.78)),
        (Ui::Throw, content::ui::THROW, Color::srgba(0.86, 0.15, 0.47, 0.78)),
        (Ui::Roll, content::ui::ROLL, Color::srgba(0.15, 0.39, 0.92, 0.78)),
        (Ui::Context, "", Color::srgba(0.85, 0.47, 0.02, 0.85)),
        (Ui::Pause, "II", dark),
    ] {
        commands
            .spawn((
                kind,
                Node {
                    border_radius: BorderRadius::MAX,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..abs(0.0, 0.0, 60.0, 60.0)
                },
                round(c),
            ))
            .with_children(|p| {
                p.spawn(text(&font, label, 15.0, Color::WHITE));
            });
    }
    commands.spawn((
        Ui::JoyBase,
        Node { border_radius: BorderRadius::MAX, ..abs(0.0, 0.0, 100.0, 100.0) },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        Visibility::Hidden,
    ));
    commands.spawn((
        Ui::JoyKnob,
        Node { border_radius: BorderRadius::MAX, ..abs(0.0, 0.0, 44.0, 44.0) },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.35)),
        Visibility::Hidden,
    ));
}

/// Spawns everything a raid shows, under one root that is despawned when it ends.
fn spawn_raid(commands: &mut Commands, assets: &AssetServer, props: &Props, g: &mut Game) {
    let Some(raid) = &g.raid else { return };
    let root = commands.spawn((Transform::default(), Visibility::default())).id();
    let mut kids = Vec::new();
    let player = actors::spawn_person(commands, assets, "minh", "Hair_SimpleParted", world_pos(raid.player.pos));
    kids.push(player);
    g.player = Some(player);
    g.thieves = raid
        .thieves
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let (who, hair) = match t.kind {
                ThiefKind::Bay | ThiefKind::TuLong => ("dai_ca", "Hair_Beard"),
                _ if i % 2 == 0 => ("trom", "Hair_Buzzed"),
                _ => ("trom2", "Hair_Buzzed"),
            };
            actors::spawn_person(commands, assets, who, hair, world_pos(t.pos))
        })
        .collect();
    kids.extend(&g.thieves);
    g.dogs = raid.dogs.iter().map(|d| actors::spawn_dog(commands, assets, world_pos(d.pos))).collect();
    kids.extend(&g.dogs);
    g.cages = raid
        .cages
        .iter()
        .map(|c| {
            commands
                .spawn((Mesh3d(props.cage.clone()), MeshMaterial3d(props.cage_mat.clone()), Transform::from_translation(world_pos(c.pos) + Vec3::Y * 0.35)))
                .id()
        })
        .collect();
    kids.extend(&g.cages);
    g.pickups = raid
        .pickups
        .iter()
        .map(|k| {
            let (mesh, mat) = match k.kind {
                PickupKind::Clue => (props.crate_mesh.clone(), props.book_mat.clone()),
                PickupKind::Slipper => (props.small.clone(), props.slipper_mat.clone()),
                PickupKind::Loot(_) => (props.crate_mesh.clone(), props.crate_mat.clone()),
            };
            commands.spawn((Mesh3d(mesh), MeshMaterial3d(mat), Transform::from_translation(world_pos(k.pos) + Vec3::Y * 0.23))).id()
        })
        .collect();
    kids.extend(&g.pickups);
    g.bikes = raid
        .vehicles
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let mat = props.bike_mats[i % props.bike_mats.len()].clone();
            commands
                .spawn((Transform::default(), Visibility::default()))
                .with_children(|b| {
                    b.spawn((Mesh3d(props.bike_body.clone()), MeshMaterial3d(mat), Transform::from_xyz(0.0, 0.55, 0.0)));
                    for z in [-0.65f32, 0.65] {
                        b.spawn((
                            Mesh3d(props.wheel.clone()),
                            MeshMaterial3d(props.tyre.clone()),
                            Transform::from_xyz(0.0, 0.3, z).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
                        ));
                    }
                    b.spawn((Mesh3d(props.rider.clone()), MeshMaterial3d(props.skin.clone()), Transform::from_xyz(0.0, 1.25, -0.15)));
                    b.spawn((
                        Mesh3d(props.helmet.clone()),
                        MeshMaterial3d(props.bike_mats[(i + 2) % props.bike_mats.len()].clone()),
                        Transform::from_xyz(0.0, 1.75, -0.12),
                    ));
                    b.spawn((
                        Mesh3d(props.helmet.clone()),
                        MeshMaterial3d(props.light.clone()),
                        Transform::from_xyz(0.0, 0.85, 0.86).with_scale(Vec3::splat(0.4)),
                    ));
                })
                .id()
        })
        .collect();
    kids.extend(&g.bikes);
    g.train = (0..7)
        .map(|_| commands.spawn((Mesh3d(props.car.clone()), MeshMaterial3d(props.car_mat.clone()), Transform::default(), Visibility::Hidden)).id())
        .collect();
    kids.extend(&g.train);
    g.exits = g
        .map
        .exits
        .iter()
        .filter(|e| raid.exit_open(e.id))
        .map(|e| {
            let c = world_pos(e.center());
            let r = (e.x1 - e.x0) * TILE_M / 2.0;
            commands
                .spawn((
                    Mesh3d(props.exit_mesh.clone()),
                    MeshMaterial3d(props.exit_mat.clone()),
                    Transform::from_translation(c + Vec3::Y * 0.1).with_scale(Vec3::new(r, 1.0, r)),
                ))
                .id()
        })
        .collect();
    kids.extend(&g.exits);
    commands.entity(root).add_children(&kids);
    g.root = Some(root);
}

fn end_raid(commands: &mut Commands, g: &mut Game) {
    if let Some(r) = g.root.take() {
        commands.entity(r).despawn();
    }
    g.player = None;
    g.thieves.clear();
    g.dogs.clear();
    g.cages.clear();
    g.pickups.clear();
    g.bikes.clear();
    g.train.clear();
    g.exits.clear();
}

fn start_raid(commands: &mut Commands, assets: &AssetServer, props: &Props, g: &mut Game, mission: Mission) {
    let _ = g.profile.next_seed();
    save_profile(&g.profile);
    let stats = g.profile.stats();
    let setup = match mission {
        Mission::Story(ch) => RaidSetup::story(&g.map, ch, stats, g.profile.seed),
        Mission::Free(l) => RaidSetup::free(&g.map, l, stats, g.profile.seed),
    };
    let mut raid = Raid::new(&g.map, setup);
    if peek_requested() {
        // A look at the action for screenshots: start a few steps from the first cage.
        if let Some(c) = raid.cages.first().map(|c| c.pos) {
            raid.player.pos = g.map.nearest_walkable(c + V2 { x: 4.0, y: 0.0 });
        }
    }
    g.cam_target = world_pos(raid.player.pos);
    g.cam_yaw = PI;
    g.raid = Some(raid);
    end_raid(commands, g);
    spawn_raid(commands, assets, props, g);
    g.screen = Screen::Playing;
}

/// Gameplay: input to controls, the raid's step, and keeping the 3D world in step with it.
#[allow(clippy::too_many_arguments)]
fn play(
    mut commands: Commands,
    mut g: ResMut<Game>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    lib: Res<AnimLib>,
    mut tf: Query<&mut Transform>,
    mut vis: Query<&mut Visibility>,
    mut people: Query<&mut Person>,
    mut dogs: Query<&mut Dog>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    camera: Single<Entity, With<MainCamera>>,
) {
    let dt = time.delta_secs().min(0.05);
    g.t += dt;
    if g.screen == Screen::Loading {
        g.loaded_t += dt;
        if lib.graph.is_some() && g.loaded_t > 0.5 {
            g.screen = if g.profile.seen_intro { Screen::Title } else { Screen::Story(0, 0) };
        }
    }
    let (w, h) = (window.width(), window.height());
    let hudl = Hud::new(w, h, [0.0; 4]);
    let cam_e = *camera;

    if g.screen != Screen::Playing {
        // Menus: the camera drifts over the lake.
        if g.raid.is_none() || matches!(g.screen, Screen::Title | Screen::Story(..) | Screen::Brief(_) | Screen::Shop) {
            let a = g.t * 0.05;
            let target = g.cam_target;
            if let Ok(mut c) = tf.get_mut(cam_e) {
                *c = Transform::from_translation(target + Vec3::new(a.cos() * 160.0, 90.0, a.sin() * 160.0)).looking_at(target, Vec3::Y);
            }
        }
        if g.screen == Screen::Paused && (keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Enter)) {
            g.screen = Screen::Playing;
        }
        return;
    }

    // Input: keyboard, then touches (joystick on the left, camera drag elsewhere, buttons).
    let mut mv = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        mv.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        mv.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        mv.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        mv.x += 1.0;
    }
    mv = mv.normalize_or_zero();
    let mut c = Controls {
        sneak: keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        attack: keys.pressed(KeyCode::KeyJ),
        throw: keys.just_pressed(KeyCode::KeyK),
        roll: keys.just_pressed(KeyCode::Space),
        interact: keys.pressed(KeyCode::KeyE),
        ..default()
    };
    let hit = |b: (V2, f32), p: Vec2| Vec2::new(b.0.x, b.0.y).distance(p) < b.1 * 1.25;
    let mut pause = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP);
    for t in touches.iter_just_pressed() {
        let p = t.position();
        if hit(hudl.pause, p) {
            pause = true;
        } else if hit(hudl.throw, p) {
            c.throw = true;
        } else if hit(hudl.roll, p) {
            c.roll = true;
        } else if g.joy.is_none() && hudl.in_joystick_zone(V2 { x: p.x, y: p.y }, w, h) {
            g.joy = Some((t.id(), p, p));
        } else if !hit(hudl.attack, p) && !hit(hudl.context, p) && g.drag.is_none() {
            g.drag = Some((t.id(), p));
        }
    }
    for t in touches.iter() {
        let start = t.start_position();
        if hit(hudl.attack, start) {
            c.attack = true;
        }
        if hit(hudl.context, start) {
            c.interact = true;
        }
        if let Some((id, s, _)) = g.joy {
            if id == t.id() {
                g.joy = Some((id, s, t.position()));
            }
        }
        if let Some((id, last)) = g.drag {
            if id == t.id() {
                let d = t.position() - last;
                g.cam_yaw -= d.x * 0.008;
                g.drag = Some((id, t.position()));
            }
        }
    }
    for t in touches.iter_just_released().chain(touches.iter_just_canceled()) {
        if g.joy.is_some_and(|j| j.0 == t.id()) {
            g.joy = None;
        }
        if g.drag.is_some_and(|d| d.0 == t.id()) {
            g.drag = None;
        }
    }
    if let Some((_, s, p)) = g.joy {
        mv = ((p - s) / hudl.joy_r).clamp_length_max(1.0);
    }
    // Mouse: drag with any button to turn the camera.
    if mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right) {
        if let Some(p) = window.cursor_position() {
            if let Some((id, last)) = g.drag.filter(|d| d.0 == u64::MAX) {
                g.cam_yaw -= (p.x - last.x) * 0.006;
                g.drag = Some((id, p));
            } else if g.drag.is_none() && !hudl.in_joystick_zone(V2 { x: p.x, y: p.y }, w, h) {
                g.drag = Some((u64::MAX, p));
            }
        }
    } else if g.drag.is_some_and(|d| d.0 == u64::MAX) {
        g.drag = None;
    }

    // The stick is relative to the camera: up walks away from it.
    let fwd = Vec2::new(g.cam_yaw.sin(), g.cam_yaw.cos());
    let right = Vec2::new(-fwd.y, fwd.x);
    let dir = fwd * (-mv.y) + right * mv.x;
    c.mv = V2 { x: dir.x, y: dir.y };
    if pause {
        g.screen = Screen::Paused;
    }

    let Some(mut raid) = g.raid.take() else { return };
    raid.update(&g.map, dt, &c);
    let finished = raid.outcome.map(|_| (raid.report(), raid.setup.mission));

    // Sync: the player.
    let p = &raid.player;
    let ppos = world_pos(p.pos);
    if let Some(e) = g.player {
        if let Ok(mut t) = tf.get_mut(e) {
            t.translation = t.translation.lerp(ppos, (dt * 20.0).min(1.0));
            t.rotation = t.rotation.slerp(yaw_of(p.facing), (dt * 14.0).min(1.0));
        }
        if let Ok(mut person) = people.get_mut(e) {
            let busy = matches!(raid.context, Context::Cage { progress, .. } | Context::Tie { progress, .. } if progress > 0.0);
            let m = if p.hp <= 0.0 {
                Move::Down
            } else if p.roll_t > 0.0 {
                Move::Roll
            } else if p.swing_t > 0.0 {
                Move::Punch
            } else if p.hurt_t > 0.5 {
                Move::Hit
            } else if busy && c.interact {
                Move::Interact
            } else if p.moving && p.sneaking {
                Move::Sneak
            } else if p.moving {
                if raid.following() > 0 { Move::Jog } else { Move::Sprint }
            } else if p.sneaking {
                Move::SneakIdle
            } else {
                Move::Idle
            };
            actors::set_move(&mut person, m, &lib, &mut players);
        }
    }
    // Thieves.
    for (i, th) in raid.thieves.iter().enumerate() {
        let Some(&e) = g.thieves.get(i) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            t.translation = t.translation.lerp(world_pos(th.pos), (dt * 15.0).min(1.0));
            t.rotation = t.rotation.slerp(yaw_of(th.facing), (dt * 10.0).min(1.0));
            if th.kind.is_boss() {
                t.scale = Vec3::splat(1.12);
            }
        }
        if let Ok(mut person) = people.get_mut(e) {
            let m = match th.state {
                TState::Down(_) | TState::Tied => Move::Down,
                TState::Stunned(_) => Move::Hit,
                TState::Windup(_) => Move::Punch,
                TState::Charge(_) | TState::Chase => Move::Sprint,
                TState::Investigate => Move::Jog,
                TState::Patrol => Move::Walk,
                _ => Move::Idle,
            };
            actors::set_move(&mut person, m, &lib, &mut players);
        }
    }
    // Dogs: in their cages, or trotting after Minh.
    for (i, d) in raid.dogs.iter().enumerate() {
        let Some(&e) = g.dogs.get(i) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            let target = world_pos(d.pos);
            let moved = t.translation.distance(target) > 0.02;
            t.translation = t.translation.lerp(target, (dt * 12.0).min(1.0));
            if d.state == DogState::Following {
                t.rotation = t.rotation.slerp(yaw_of(d.facing), (dt * 8.0).min(1.0));
            }
            if let Ok(mut dog) = dogs.get_mut(e) {
                let which = if d.state == DogState::Following && moved { if p.moving && !p.sneaking { 2 } else { 1 } } else { 0 };
                actors::set_dog_move(&mut dog, which, &lib, &mut players);
            }
        }
    }
    for (i, cg) in raid.cages.iter().enumerate() {
        if let (Some(&e), true) = (g.cages.get(i), cg.open) {
            if let Ok(mut v) = vis.get_mut(e) {
                *v = Visibility::Hidden;
            }
        }
    }
    for (i, k) in raid.pickups.iter().enumerate() {
        let Some(&e) = g.pickups.get(i) else { continue };
        if let Ok(mut v) = vis.get_mut(e) {
            *v = if k.taken { Visibility::Hidden } else { Visibility::Inherited };
        }
        if let Ok(mut t) = tf.get_mut(e) {
            t.rotation = Quat::from_rotation_y(g.t * 1.5 + i as f32);
        }
    }
    // Traffic.
    for (i, v) in raid.vehicles.iter().enumerate() {
        let Some(&e) = g.bikes.get(i) else { continue };
        if let Ok(mut t) = tf.get_mut(e) {
            t.translation = world_pos(v.pos);
            t.rotation = Quat::from_rotation_y(v.dir.x.atan2(v.dir.y));
        }
    }
    // The train.
    if let Some(rail) = &g.map.rail {
        for (k, &e) in g.train.iter().enumerate() {
            let d = raid.train.s - k as f32 * 2.0 - 1.0;
            let show = raid.train.active && d >= 0.0 && d <= rail.len;
            if let Ok(mut v) = vis.get_mut(e) {
                *v = if show { Visibility::Inherited } else { Visibility::Hidden };
            }
            if show {
                if let Ok(mut t) = tf.get_mut(e) {
                    let (q, dir) = rail.at(d);
                    *t = Transform::from_translation(world_pos(q) + Vec3::Y * 1.9)
                        .with_rotation(Quat::from_rotation_y(dir.x.atan2(dir.y)))
                        .with_scale(Vec3::new(1.0, 1.0, TILE_M * 2.0 - 0.3));
                }
            }
        }
    }
    // Camera: behind and above the player, easing round when he runs.
    if p.moving && g.drag.is_none() {
        let want = p.facing.cos().atan2(p.facing.sin());
        let cur = g.cam_yaw;
        let mut d = (want - cur) % TAU;
        if d > PI {
            d -= TAU;
        } else if d < -PI {
            d += TAU;
        }
        g.cam_yaw = cur + d * (dt * 0.9).min(1.0);
    }
    let target = g.cam_target.lerp(ppos + Vec3::Y * 1.3, (dt * 8.0).min(1.0));
    g.cam_target = target;
    let yaw = g.cam_yaw;
    let shake = if raid.shake > 0.0 { Vec3::new((g.t * 90.0).sin(), (g.t * 77.0).cos(), 0.0) * 0.08 } else { Vec3::ZERO };
    if let Ok(mut t) = tf.get_mut(cam_e) {
        let back = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
        *t = Transform::from_translation(target + back * 6.5 + Vec3::Y * 3.4 + shake).looking_at(target, Vec3::Y);
    }
    g.raid = Some(raid);

    if let Some((report, mission)) = finished {
        let earned = g.profile.apply(mission, &report);
        save_profile(&g.profile);
        g.result = Some((report, earned));
        g.screen = Screen::Result;
        g.joy = None;
        g.drag = None;
    }
    let _ = &mut commands;
}

/// The heads-up display during a raid, and the loading text.
#[allow(clippy::too_many_arguments)]
fn hud(
    g: Res<Game>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut nodes: Query<(&Ui, &mut Node, &mut Visibility, Option<&mut Text>, Option<&mut BackgroundColor>, Option<&Children>), Without<Label3d>>,
    mut child_text: Query<&mut Text, (Without<Ui>, Without<Label3d>)>,
    mut labels: Query<(&Label3d, &mut Node, &mut Visibility), Without<Ui>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (w, h) = (window.width(), window.height());
    let hudl = Hud::new(w, h, [0.0; 4]);
    let playing = matches!(g.screen, Screen::Playing | Screen::Paused) && g.raid.is_some();
    let raid = g.raid.as_ref();
    let (cam, cam_t) = *camera;
    for (kind, mut node, mut v, text, bg, kids) in &mut nodes {
        let mut show = playing;
        let place = |node: &mut Node, x: f32, y: f32, ww: f32, hh: f32| {
            node.left = Val::Px(x);
            node.top = Val::Px(y);
            node.width = Val::Px(ww);
            node.height = Val::Px(hh);
        };
        let round = |node: &mut Node, (c, r): (V2, f32)| {
            node.left = Val::Px(c.x - r);
            node.top = Val::Px(c.y - r);
            node.width = Val::Px(r * 2.0);
            node.height = Val::Px(r * 2.0);
        };
        match kind {
            Ui::Loading => {
                show = g.screen == Screen::Loading;
                place(&mut node, w / 2.0 - 160.0, h / 2.0 - 15.0, 320.0, 30.0);
            }
            Ui::Hp => {
                if let Some(r) = raid {
                    let frac = (r.player.hp / r.setup.stats.max_hp).clamp(0.0, 1.0);
                    place(&mut node, 12.0, 12.0, 200.0 * frac, 12.0);
                    if let Some(mut bg) = bg {
                        bg.0 = if frac > 0.5 {
                            Color::srgb(0.13, 0.77, 0.37)
                        } else if frac > 0.25 {
                            Color::srgb(0.96, 0.62, 0.04)
                        } else {
                            Color::srgb(0.94, 0.27, 0.27)
                        };
                    }
                }
            }
            Ui::Stamina => {
                if let Some(r) = raid {
                    place(&mut node, 12.0, 28.0, 2.0 * r.player.stamina.clamp(0.0, 100.0), 4.0);
                }
            }
            Ui::Timer => {
                if let (Some(r), Some(mut t)) = (raid, text) {
                    let s = r.time_left.max(0.0) as u32;
                    t.0 = format!("{:02}:{:02}", s / 60, s % 60);
                }
            }
            Ui::Info => {
                if let (Some(r), Some(mut t)) = (raid, text) {
                    t.0 = format!("Chó {}/{} · Dép {} · Balo {}/{}", r.following(), r.setup.stats.leash, r.player.slippers, r.bag.len(), r.setup.stats.bag);
                }
            }
            Ui::Objective => {
                if let (Some(r), Some(mut t)) = (raid, text) {
                    t.0 = r.objective_text().replace(" · ", "\n");
                }
                place(&mut node, 12.0, 70.0, (w * 0.55).min(340.0), 90.0);
            }
            Ui::Banner => {
                let msg = raid.and_then(|r| r.banner.as_ref().map(|b| b.0.clone()));
                show = playing && msg.is_some();
                if let (Some(m), Some(mut t)) = (msg, text) {
                    t.0 = m;
                }
                place(&mut node, w * 0.1, h * 0.28, w * 0.8, 30.0);
            }
            Ui::Extract => {
                let e = raid.map_or(0.0, |r| r.player.extract_t);
                show = playing && e > 0.0;
                if let Some(mut t) = text {
                    let left = (hanoi_rescue::raid::EXTRACT_S - e).max(0.0);
                    t.0 = format!("{} {:.1}s", content::ui::EXTRACTING, left);
                }
                place(&mut node, w / 2.0 - 90.0, h * 0.4, 180.0, 30.0);
            }
            Ui::Hint => {
                show = playing && w > 900.0;
                place(&mut node, w / 2.0 - 330.0, h - 26.0, 660.0, 20.0);
            }
            Ui::Attack => round(&mut node, hudl.attack),
            Ui::Throw => {
                round(&mut node, hudl.throw);
                if let (Some(r), Some(kids)) = (raid, kids) {
                    for k in kids.iter() {
                        if let Ok(mut t) = child_text.get_mut(k) {
                            t.0 = format!("{} {}", content::ui::THROW, r.player.slippers);
                        }
                    }
                }
            }
            Ui::Roll => round(&mut node, hudl.roll),
            Ui::Pause => round(&mut node, hudl.pause),
            Ui::Context => {
                round(&mut node, hudl.context);
                let label = raid.and_then(|r| match r.context {
                    Context::Cage { .. } => Some(content::ui::OPEN_CAGE),
                    Context::Tie { .. } => Some(content::ui::TIE),
                    Context::LeashFull => Some(content::ui::LEASH_FULL),
                    Context::None => None,
                });
                show = playing && label.is_some();
                if let (Some(l), Some(kids)) = (label, kids) {
                    for k in kids.iter() {
                        if let Ok(mut t) = child_text.get_mut(k) {
                            t.0 = l.to_string();
                        }
                    }
                }
            }
            Ui::JoyBase => {
                let (s, _) = g.joy.map_or((Vec2::new(hudl.joy_home.x, hudl.joy_home.y), Vec2::ZERO), |j| (j.1, j.2));
                place(&mut node, s.x - hudl.joy_r, s.y - hudl.joy_r, hudl.joy_r * 2.0, hudl.joy_r * 2.0);
            }
            Ui::JoyKnob => {
                let (s, p) = g.joy.map_or((Vec2::new(hudl.joy_home.x, hudl.joy_home.y), Vec2::new(hudl.joy_home.x, hudl.joy_home.y)), |j| (j.1, j.2));
                let k = s + (p - s).clamp_length_max(hudl.joy_r);
                place(&mut node, k.x - 22.0, k.y - 22.0, 44.0, 44.0);
            }
            Ui::Minimap => {
                let m = hudl.minimap;
                place(&mut node, m.x, m.y, m.w, m.h);
            }
            Ui::MiniDot => {
                if let Some(r) = raid {
                    let m = hudl.minimap;
                    let x = m.x + r.player.pos.x / g.map.w as f32 * m.w;
                    let y = m.y + r.player.pos.y / g.map.h as f32 * m.h;
                    place(&mut node, x - 3.5, y - 3.5, 7.0, 7.0);
                }
            }
        }
        *v = if show { Visibility::Inherited } else { Visibility::Hidden };
    }
    // Landmark names over the world.
    for (l, mut node, mut v) in &mut labels {
        let on = playing && cam_t.translation().distance(l.0) < 260.0;
        match cam.world_to_viewport(cam_t, l.0) {
            Ok(p) if on && p.x > 0.0 && p.y > 0.0 && p.x < w && p.y < h => {
                node.left = Val::Px(p.x - 60.0);
                node.top = Val::Px(p.y);
                *v = Visibility::Inherited;
            }
            _ => *v = Visibility::Hidden,
        }
    }
}

/// Title, story, briefing, shop, pause and results: panels of text and buttons, rebuilt when the
/// screen changes; their buttons work with touch and the mouse.
#[allow(clippy::too_many_arguments)]
fn menus(
    mut commands: Commands,
    mut g: ResMut<Game>,
    font: Res<UiFont>,
    assets: Res<AssetServer>,
    props: Res<Props>,
    roots: Query<Entity, With<MenuRoot>>,
    mut clicked_res: ResMut<Clicked>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    // Clicks first.
    let mut clicked = clicked_res.0.take();
    if keys.just_pressed(KeyCode::Enter) {
        clicked = clicked.or(match g.screen {
            Screen::Title => Some(MenuButton::Story),
            Screen::Story(..) => Some(MenuButton::Next),
            Screen::Brief(_) => Some(MenuButton::Go),
            Screen::Result => Some(MenuButton::Done),
            _ => None,
        });
    }
    if let Some(b) = clicked {
        match (b, g.screen) {
            (MenuButton::Story, _) => {
                if !g.profile.story_done() {
                    g.screen = Screen::Brief(Mission::Story(g.profile.chapter as usize));
                }
            }
            (MenuButton::Free, _) => g.screen = Screen::Brief(Mission::Free(g.profile.free_level)),
            (MenuButton::Shop, _) => g.screen = Screen::Shop,
            (MenuButton::Back, _) => g.screen = Screen::Title,
            (MenuButton::Go, Screen::Brief(m)) => {
                let need = matches!(m, Mission::Story(2)) && g.profile.stats().leash < 2;
                if !need {
                    start_raid(&mut commands, &assets, &props, &mut g, m);
                }
            }
            (MenuButton::Buy(i), _) => {
                if let Some(u) = UPGRADES.get(i) {
                    if g.profile.buy(u.kind) {
                        save_profile(&g.profile);
                        g.shown = None;
                    }
                }
            }
            (MenuButton::Next, Screen::Story(s, i)) => {
                g.screen = if i + 1 < story_lines(s).len() { Screen::Story(s, i + 1) } else { Screen::Title };
                if s == 0 && !g.profile.seen_intro {
                    g.profile.seen_intro = true;
                    save_profile(&g.profile);
                }
            }
            (MenuButton::Skip, Screen::Story(s, _)) => {
                if s == 0 {
                    g.profile.seen_intro = true;
                    save_profile(&g.profile);
                }
                g.screen = Screen::Title;
            }
            (MenuButton::Resume, _) => g.screen = Screen::Playing,
            (MenuButton::Abandon, _) => {
                if let Some(r) = g.raid.as_mut() {
                    r.outcome = Some(Outcome::Abandoned);
                }
                let report = g.raid.as_ref().map(|r| (r.report(), r.setup.mission));
                if let Some((rep, m)) = report {
                    let e = g.profile.apply(m, &rep);
                    save_profile(&g.profile);
                    g.result = Some((rep, e));
                }
                g.screen = Screen::Result;
            }
            (MenuButton::Done, _) => {
                let chapter = g.result.as_ref().and_then(|(_, e)| e.chapter_done);
                g.raid = None;
                end_raid(&mut commands, &mut g);
                g.screen = match chapter {
                    Some(ch) => Screen::Story(ch + 1, 0),
                    None => Screen::Title,
                };
            }
            _ => {}
        }
    }

    if g.shown == Some(g.screen) {
        return;
    }
    g.shown = Some(g.screen);
    for r in &roots {
        commands.entity(r).despawn();
    }
    let f = &font.0;
    let panel = |commands: &mut Commands| {
        commands
            .spawn((
                MenuRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    width: Val::Px(360.0),
                    max_width: Val::Vw(92.0),
                    max_height: Val::Vh(92.0),
                    margin: UiRect { left: Val::Px(-180.0), top: Val::Px(-200.0), ..default() },
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.07, 0.1, 0.9)),
            ))
            .id()
    };
    let button = |commands: &mut Commands, parent: Entity, label: &str, kind: MenuButton, primary: bool| {
        let b = commands
            .spawn((
                Button,
                kind,
                Node {
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(if primary { Color::srgb(0.96, 0.62, 0.04) } else { Color::srgb(0.18, 0.2, 0.26) }),
            ))
            .with_children(|p| {
                p.spawn(text(f, label, 17.0, if primary { Color::srgb(0.1, 0.07, 0.02) } else { Color::WHITE }));
            })
            .observe(move |_: On<Activate>, mut c: ResMut<Clicked>| c.0 = Some(kind))
            .id();
        commands.entity(parent).add_child(b);
    };
    let line = |commands: &mut Commands, parent: Entity, s: &str, size: f32, c: Color| {
        let t = commands.spawn(text(f, s, size, c)).id();
        commands.entity(parent).add_child(t);
    };
    let gold = Color::srgb(0.98, 0.75, 0.15);
    let soft = Color::srgb(0.82, 0.84, 0.88);
    match g.screen {
        Screen::Loading | Screen::Playing => {}
        Screen::Title => {
            let p = panel(&mut commands);
            line(&mut commands, p, content::TITLE, 34.0, gold);
            line(&mut commands, p, "Hà Nội · Hoàn Kiếm · phố thật", 15.0, Color::WHITE);
            line(&mut commands, p, &format!("Tiền {}k · Đã cứu {} chú chó", g.profile.money, g.profile.dogs.len()), 14.0, soft);
            if let Some(ch) = CHAPTERS.get(g.profile.chapter as usize) {
                button(&mut commands, p, &format!("Chương {}: {}", g.profile.chapter + 1, ch.title), MenuButton::Story, true);
            }
            if g.profile.free_unlocked() {
                button(&mut commands, p, &format!("{} · cấp {}", content::FREE_RAID_TITLE, g.profile.free_level), MenuButton::Free, g.profile.story_done());
            }
            button(&mut commands, p, content::ui::SHOP, MenuButton::Shop, false);
            line(&mut commands, p, "Bản đồ © OpenStreetMap · nhân vật CC0 Quaternius", 11.0, Color::srgb(0.6, 0.62, 0.66));
        }
        Screen::Story(s, i) => {
            let p = panel(&mut commands);
            if let Some((who, said)) = story_lines(s).get(i) {
                if !who.is_empty() {
                    line(&mut commands, p, who, 16.0, gold);
                }
                line(&mut commands, p, said, 17.0, Color::WHITE);
            }
            button(&mut commands, p, "Tiếp", MenuButton::Next, true);
            button(&mut commands, p, content::ui::SKIP, MenuButton::Skip, false);
        }
        Screen::Brief(m) => {
            let p = panel(&mut commands);
            let (title, brief, tip) = match m {
                Mission::Story(ch) => {
                    let c = CHAPTERS.get(ch);
                    (format!("Chương {}: {}", ch + 1, c.map_or("", |c| c.title)), c.map_or("", |c| c.brief), c.map_or("", |c| c.tip))
                }
                Mission::Free(l) => (format!("{} · cấp {l}", content::FREE_RAID_TITLE), content::FREE_RAID_BRIEF, "Bị tóm hoặc hết giờ là mất balo."),
            };
            line(&mut commands, p, &title, 20.0, gold);
            line(&mut commands, p, brief, 15.0, Color::WHITE);
            line(&mut commands, p, &format!("Mẹo: {tip}"), 13.0, Color::srgb(0.65, 0.95, 0.8));
            if matches!(m, Mission::Story(2)) && g.profile.stats().leash < 2 {
                line(&mut commands, p, "Cần mua Dây xích ở cửa hàng để dắt 2 chú chó.", 14.0, Color::srgb(1.0, 0.55, 0.55));
                button(&mut commands, p, content::ui::SHOP, MenuButton::Shop, true);
            } else {
                button(&mut commands, p, content::ui::GO, MenuButton::Go, true);
            }
            button(&mut commands, p, content::ui::BACK, MenuButton::Back, false);
        }
        Screen::Shop => {
            let p = panel(&mut commands);
            line(&mut commands, p, &format!("{} · {}k", content::ui::SHOP, g.profile.money), 20.0, gold);
            for (i, u) in UPGRADES.iter().enumerate() {
                let lvl = g.profile.upgrades.level(u.kind);
                let label = match g.profile.next_price(u.kind) {
                    Some(price) => format!("{} {}/{} · {price}k", u.name, lvl, u.prices.len()),
                    None => format!("{} · {}", u.name, content::ui::MAXED),
                };
                button(&mut commands, p, &label, MenuButton::Buy(i), false);
            }
            button(&mut commands, p, content::ui::BACK, MenuButton::Back, true);
        }
        Screen::Paused => {
            let p = panel(&mut commands);
            line(&mut commands, p, content::ui::PAUSED, 24.0, Color::WHITE);
            button(&mut commands, p, content::ui::RESUME, MenuButton::Resume, true);
            button(&mut commands, p, content::ui::ABANDON, MenuButton::Abandon, false);
        }
        Screen::Result => {
            let p = panel(&mut commands);
            if let Some((rep, e)) = &g.result {
                let (title, c) = match rep.outcome {
                    Outcome::Escaped(_) => (content::ui::SUCCESS, Color::srgb(0.2, 0.83, 0.6)),
                    Outcome::TimeUp => (content::ui::TIMEOUT, Color::srgb(0.96, 0.62, 0.04)),
                    _ => (content::ui::CAUGHT, Color::srgb(0.94, 0.27, 0.27)),
                };
                line(&mut commands, p, title, 26.0, c);
                if let Some(ch) = e.chapter_done {
                    line(&mut commands, p, &format!("Hoàn thành chương {}!", ch + 1), 16.0, gold);
                }
                if !e.new_dogs.is_empty() {
                    line(&mut commands, p, &format!("Đã cứu: {}", e.new_dogs.join(", ")), 15.0, Color::WHITE);
                }
                line(&mut commands, p, &format!("Trói {} tên trộm · +{}k", rep.tied, e.money), 15.0, soft);
            }
            button(&mut commands, p, content::ui::TO_HIDEOUT, MenuButton::Done, true);
        }
    }
}

//! Hanoi City: the dog-rescue extraction game in 3D on the real streets of Hoàn Kiếm.
//!
//! The rules (thieves who see, hear and chase, cages, traffic, the train, getting out) are the
//! tested `hanoi-rescue` core, running on a map rasterised from OpenStreetMap; this crate draws
//! that world in 3D with Bevy and turns touches, the mouse and the keyboard into its controls.

mod actors;
mod cage;
mod crowd;
mod dressing;
mod facade;
mod guide;
mod hq;
mod mips;
mod night;
mod render;
mod world;

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::asset::AssetMetaCheck;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::text::FontSize;
use bevy::ui_widgets::{Activate, Button};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use hanoi_rescue::city::City;
use hanoi_rescue::content::{self, CHAPTERS, UPGRADES};
use hanoi_rescue::game::Hud;
use hanoi_rescue::geom::V2;
use hanoi_rescue::map::{Map, Tile};
#[cfg(target_arch = "wasm32")]
use hanoi_rescue::meta::SAVE_KEY;
use hanoi_rescue::meta::{Earned, Profile};
use hanoi_rescue::raid::{Context, Controls, DogState, Mission, Outcome, PickupKind, Raid, RaidSetup, Report, TState, ThiefKind};

use actors::{AnimLib, Dog, Move, Person, Props};

const CITY_JSON: &str = include_str!("../assets/hoankiem.json");
const SIGNS_JSON: &str = include_str!("../assets/signs.json");
const TILE_M: f32 = 2.0;
/// The chapters the story sets after dark ("Đêm mất Bơ"): played at night.
const NIGHT_CHAPTERS: &[usize] = &[0];
/// The controls, shown along the bottom during a raid.
const PC_KEYS: &str = "WASD đi · Shift lén · bấm để khoá chuột, di chuột nhìn · chuột trái đánh · chuột phải ném dép · Space lăn · giữ E mở/trói · Esc dừng";

fn main() {
    App::new()
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
        .add_plugins(hq::HqPlugin)
        .init_resource::<Clicked>()
        .add_systems(Startup, (setup, guide::setup))
        .add_systems(
            Update,
            (
                (actors::adopt_hq, actors::build_graphs, loading, pointer_lock, play, hud, menus).chain(),
                world::cull_far,
                mips::make_mips,
                facade::stream,
                crowd::spawn,
                crowd::walk,
                world::ripple,
                (choose_sky, night::apply, night::pool).chain().after(menus),
                (fill_guide, guide::draw, guide::hud).chain().after(hud),
                night::wet,
                night::torch_prop,
                (cage::settle, cage::swing).chain().after(play),
            ),
        )
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

#[cfg(test)]
mod pacing_tests {
    use super::*;
    use hanoi_rescue::meta::Profile;

    /// On the real map every chapter starts within a short walk of its first goal (no minutes of
    /// empty road), and never on top of a thief.
    #[test]
    fn every_chapter_starts_near_its_first_goal() {
        let city = City::from_json(CITY_JSON).unwrap_or_default();
        let mut map = Map::from_city(&city, TILE_M);
        map.fence_shore(&city, world::LAKE, world::LAKE_RAILING_M, world::BRIDGE_GAP_M);
        let stats = Profile::default().stats();
        for ch in 0..CHAPTERS.len() {
            for seed in 0..6u64 {
                let r = Raid::new(&map, RaidSetup::story(&map, ch, stats, seed));
                let Some((at, kind)) = r.goal(&map) else { panic!("ch{ch} seed{seed}: no goal") };
                let d = r.player.pos.dist(at) * TILE_M;
                assert!(d < 450.0, "ch{ch} seed{seed}: the first goal ({kind:?}) is {d:.0} m away");
                for t in &r.thieves {
                    assert!(t.pos.dist(r.player.pos) * TILE_M > 12.0 || t.kind.is_boss(), "ch{ch} seed{seed}: a thief at the start");
                }
            }
        }
    }
}

#[cfg(test)]
mod speed_tests {
    #[test]
    fn reads_the_game_speed() {
        assert_eq!(super::game_speed("0.1"), Some(0.1));
        assert_eq!(super::game_speed("100"), Some(4.0));
        assert_eq!(super::game_speed("0.004"), Some(0.004));
        assert_eq!(super::game_speed("0.00001"), Some(0.001));
        for bad in ["", "abc", "0", "-1", "NaN", "inf"] {
            assert_eq!(super::game_speed(bad), None, "{bad}");
        }
    }
}

#[derive(Resource)]
struct Game {
    map: Map,
    profile: Profile,
    raid: Option<Raid>,
    /// What the current (or last) raid is.
    mission: Option<Mission>,
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
    /// How far the camera looks down (radians) and how far behind it sits (metres).
    cam_pitch: f32,
    cam_dist: f32,
    cam_target: Vec3,
    /// The mouse is captured for looking around (the page's pointer lock).
    locked: bool,
    /// Someone touched the screen: show the on-screen stick and buttons.
    touch_seen: bool,
    drag: Option<(u64, Vec2)>,
    joy: Option<(u64, Vec2, Vec2)>,
    t: f32,
    loaded_t: f32,
    /// Art review (`?look=…`): a fixed camera, no menus.
    look: Option<Look>,
}

/// A fixed view for reviewing delivered art against the reference photo, from the page's address:
/// `?look=thehuc` (from the lakeside by its shore end along the Thê Húc bridge), `?look=bridge` (the
/// bridge from above the water), `?look=lineup` (Minh, a thief and the dog standing at the bridge's
/// shore end), `?look=tree` (the nearest street tree), `?look=streetlife` (the nearest table and
/// stools on a pavement, or else the nearest pavement piece) or `?look=cage` (the dog cage close up,
/// 1.5 m away at eye height, on the lakeside path; `&door=open` swings its door open) or
/// `?look=hangma` (along Hàng Mã under its lanterns, at eye height).
#[derive(Clone, Copy)]
struct Look {
    eye: Vec3,
    target: Vec3,
    /// Where to stand the characters, facing the camera, for `lineup`.
    lineup: Option<(Vec3, Vec3)>,
    /// Where to stand the cage, facing the camera, and whether its door opens, for `cage`.
    cage: Option<(Vec3, bool)>,
    spawned: bool,
}

fn look_from_page(sites: &[hq::Site], facades: &facade::Facades, map: &Map) -> Option<Look> {
    let which = render::page_query("look");
    if which.is_empty() {
        return None;
    }
    let bridge = sites.iter().find(|s| s.file.ends_with("cau_the_huc.glb"))?;
    // The bridge's north end is its model's −Z turned by the site's yaw; the shore end is south.
    let north = Quat::from_rotation_y(bridge.yaw) * Vec3::NEG_Z;
    let right = north.cross(Vec3::Y);
    let shore = bridge.at - north * 23.0 - north * 16.0;
    match which.as_str() {
        "lineup" => {
            Some(Look { eye: shore - north * 4.2 + Vec3::Y * 1.45, target: shore + Vec3::Y * 0.95, lineup: Some((shore, right)), cage: None, spawned: false })
        }
        "tree" => {
            let tree = sites.iter().filter(|s| s.kind == "tree").min_by(|a, b| a.at.distance(shore).total_cmp(&b.at.distance(shore)))?;
            Some(Look { eye: tree.at - north * 13.0 + right * 4.0 + Vec3::Y * 3.0, target: tree.at + Vec3::Y * 6.0, lineup: None, cage: None, spawned: true })
        }
        // The pavement life (tables and stools, else any piece) nearest the bridge's shore end, from
        // the street at eye level, up to 4 m out. In a narrow street 4 m out can be inside the
        // building across it (the view was black), so the eye is moved in until the map is open
        // all the way to it, or the next piece is tried.
        "streetlife" => {
            let open = |v: Vec3| map.walkable_at(V2 { x: v.x / TILE_M, y: v.z / TILE_M });
            let ground: Vec<&facade::Slot> = facades
                .blocks
                .iter()
                .flat_map(|b| b.slots.iter())
                .filter(|s| matches!(s.storey, facade::Storey::Ground | facade::Storey::FrenchGround))
                .collect();
            let view = |p: &facade::PropSlot| {
                let at = p.at.translation;
                let slot = ground.iter().min_by(|a, b| a.corner.distance(at).total_cmp(&b.corner.distance(at)))?;
                let out = slot.transform().rotation * Vec3::Z;
                let away = eye_distance(at, out, &open)?;
                Some(Look { eye: at + out * away + Vec3::Y * 1.6, target: at + Vec3::Y * 0.35, lineup: None, cage: None, spawned: true })
            };
            let nearest = |files: &[&str]| {
                let mut props: Vec<&facade::PropSlot> = facades
                    .blocks
                    .iter()
                    .flat_map(|b| b.props.iter())
                    .filter(|p| files.contains(&p.file) || p.alts.iter().any(|a| files.contains(a)))
                    .collect();
                props.sort_by(|a, b| a.at.translation.distance(shore).total_cmp(&b.at.translation.distance(shore)));
                props.into_iter().take(40).find_map(view)
            };
            let all: Vec<&str> = [dressing::PARKED_BIKES, dressing::STOOLS, &[dressing::TABLE, dressing::PLANT, dressing::BIN]].concat();
            nearest(&[dressing::TABLE]).or_else(|| nearest(&all))
        }
        "bridge" => Some(Look {
            eye: bridge.at + right * 38.0 - north * 6.0 + Vec3::Y * 11.0,
            target: bridge.at + Vec3::Y * 1.0,
            lineup: None,
            cage: None,
            spawned: true,
        }),
        // From the lakeside path just north of the bridge's shore end, looking along it to the island.
        "hangma" => {
            let ls: Vec<&hq::Site> = sites.iter().filter(|s| s.kind == "lantern").collect();
            let n = ls.len().max(1) as f32;
            let mid = ls.iter().fold(Vec3::ZERO, |a, s| a + s.at) / n;
            let a = ls.iter().min_by(|x, y| x.at.distance(mid).total_cmp(&y.at.distance(mid)))?;
            // The next lantern on the same string (1.6 m on), and the one across the street.
            let along = ls
                .iter()
                .map(|s| s.at - a.at)
                .filter(|d| (1.0..2.5).contains(&d.length()))
                .min_by(|x, y| x.length().total_cmp(&y.length()))?
                .with_y(0.0)
                .normalize_or_zero();
            let across = ls
                .iter()
                .map(|s| s.at - a.at)
                .filter(|d| (3.0..16.0).contains(&d.length()) && d.normalize_or_zero().dot(along).abs() < 0.3)
                .min_by(|x, y| x.length().total_cmp(&y.length()))?;
            let eye = (a.at + across * 0.5).with_y(1.6) - along * 6.0;
            Some(Look { eye, target: eye + along * 14.0 + Vec3::Y * 2.6, lineup: None, cage: None, spawned: true })
        }
        // The cage on the lakeside path by the bridge's shore end, seen from in front and a little to
        // the side, 1.5 m away at eye height (how a player sees it when they reach it).
        "cage" => {
            let open = render::page_query("door") == "open";
            let eye = shore - north * 1.3 + right * 0.75 + Vec3::Y * 1.5;
            Some(Look { eye, target: shore + Vec3::Y * 0.3, lineup: None, cage: Some((shore, open)), spawned: false })
        }
        _ => Some(Look {
            eye: bridge.at - north * 21.0 + right * 15.0 + Vec3::Y * 1.7,
            target: bridge.at + north * 6.0 + Vec3::Y * 1.0,
            lineup: None,
            cage: None,
            spawned: true,
        }),
    }
}

/// How far out from a pavement piece (along `out`) a review camera can stand: the farthest of
/// 4.2, 3.6, 3.0 and 2.5 m with open ground every 0.5 m from the piece to 0.8 m past the eye, so the
/// eye is not inside, or against, the building across the street.
fn eye_distance(at: Vec3, out: Vec3, open: &dyn Fn(Vec3) -> bool) -> Option<f32> {
    [4.2f32, 3.6, 3.0, 2.5].into_iter().find(|&d| {
        let steps = ((d + 0.8) / 0.5).ceil() as i32;
        (1..=steps).all(|i| open(at + out * (i as f32 * 0.5).min(d + 0.8)))
    })
}

#[cfg(test)]
mod look_tests {
    use super::*;

    #[test]
    fn the_eye_stays_out_of_the_building_across_the_street() {
        // Open for 4 m along +X, then a wall.
        let open = |v: Vec3| v.x < 4.0;
        assert_eq!(eye_distance(Vec3::ZERO, Vec3::X, &open), Some(3.0));
        assert_eq!(eye_distance(Vec3::ZERO, Vec3::X, &|_| true), Some(4.2));
        assert_eq!(eye_distance(Vec3::ZERO, Vec3::X, &|v: Vec3| v.x < 2.0), None);
    }
}

#[derive(Component)]
struct MainCamera;

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
    mut media: ResMut<Assets<bevy::light::atmosphere::ScatteringMedium>>,
    mut clock: ResMut<Time<Virtual>>,
) {
    if let Some(speed) = game_speed(&render::page_query("speed")) {
        // Recording on a slow computer: the game runs slower than real time and every frame counts,
        // however long it took to draw, so the video can be sped up afterwards to real speed.
        clock.set_relative_speed(speed);
        clock.set_max_delta(std::time::Duration::from_secs(10));
    }
    let city = City::from_json(CITY_JSON).unwrap_or_default();
    let mut map = Map::from_city(&city, TILE_M);
    map.fence_shore(&city, world::LAKE, world::LAKE_RAILING_M, world::BRIDGE_GAP_M);
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
    let built = world::spawn_city(&mut commands, &mut meshes, &m, &city, &map, Vec2::new(lake.x, lake.z), &signs);
    let look = look_from_page(&built.sites, &built.facades, &map);
    commands.insert_resource(hq::Sites(built.sites));
    commands.insert_resource(built.facades);
    commands.insert_resource(crowd::Promenade::new(built.promenade));
    commands.insert_resource(night::Glows(built.glows));
    commands.insert_resource(built.carriageways);
    commands.insert_resource(night::Sky::default());
    commands.insert_resource(guide::Guide::default());
    let (_, cols, rows) = facade::street_atlas();
    let sign_mat = mats.add(StandardMaterial { base_color_texture: Some(assets.load("tex/street_names.png")), perceptual_roughness: 0.45, ..default() });
    commands.insert_resource(facade::SignFaces::new(sign_mat, cols, rows));
    commands.insert_resource(m);

    let fx = render::Fx::from_page();
    info!("renderer: {fx:?}");
    commands.insert_resource(fx);
    render::spawn_sun(&mut commands, fx);
    let mut cam = commands.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection { fov: 0.95, ..default() }),
        Transform::from_translation(lake + Vec3::new(0.0, 120.0, 220.0)).looking_at(lake, Vec3::Y),
    ));
    render::setup_camera(&mut cam, &mut media, fx);

    commands.insert_resource(AnimLib {
        gltf: assets.load("anim.glb"),
        dog_gltf: assets.load("dog.glb"),
        dog_path: "dog.glb".into(),
        dog_scale: 0.0075,
        graph: None,
        dog_graph: None,
    });
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
        mission: None,
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
        // Low and close behind Minh, at street level like a person following him; the wheel moves
        // it back and up for an overview.
        cam_pitch: 0.2,
        cam_dist: 4.2,
        cam_target: lake,
        locked: false,
        touch_seen: false,
        drag: None,
        joy: None,
        t: 0.0,
        loaded_t: 0.0,
        look,
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
    commands.spawn((Ui::Hint, text(&font, PC_KEYS, 12.0, Color::srgb(0.85, 0.87, 0.9)), abs(0.0, 0.0, 10.0, 20.0), Visibility::Hidden));
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
                p.spawn(text(&font, label, 13.0, Color::WHITE));
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
fn spawn_raid(commands: &mut Commands, assets: &AssetServer, hq: &hq::Hq, lib: &AnimLib, props: &Props, g: &mut Game) {
    let Some(raid) = &g.raid else { return };
    let root = commands.spawn((Transform::default(), Visibility::default())).id();
    let mut kids = Vec::new();
    let player = actors::spawn_person(commands, assets, hq, "minh", "Hair_SimpleParted", world_pos(raid.player.pos));
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
            actors::spawn_person(commands, assets, hq, who, hair, world_pos(t.pos))
        })
        .collect();
    kids.extend(&g.thieves);
    g.dogs = raid.dogs.iter().map(|d| actors::spawn_dog(commands, assets, lib, world_pos(d.pos))).collect();
    kids.extend(&g.dogs);
    g.cages = raid
        .cages
        .iter()
        .map(|c| {
            const CAGE: [&str; 3] = ["people/long_cho.glb", "animals/long_cho.glb", "street/long_cho.glb"];
            match hq::prop(commands, hq, &CAGE, Transform::from_translation(world_pos(c.pos))) {
                Some(e) => commands.entity(e).insert(cage::CageDoor::new(true)).id(),
                None => commands
                    .spawn((
                        Mesh3d(props.cage.clone()),
                        MeshMaterial3d(props.cage_mat.clone()),
                        Transform::from_translation(world_pos(c.pos) + Vec3::Y * 0.35),
                        cage::CageDoor::new(false),
                    ))
                    .id(),
            }
        })
        .collect();
    kids.extend(&g.cages);
    g.pickups = raid
        .pickups
        .iter()
        .map(|k| {
            let file = match k.kind {
                PickupKind::Clue => "street/so_tay.glb",
                PickupKind::Slipper => "street/dep_to_ong.glb",
                PickupKind::Loot(_) => "street/thung_hang.glb",
            };
            if let Some(e) = hq::prop(commands, hq, &[file], Transform::from_translation(world_pos(k.pos))) {
                return e;
            }
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
            const BIKES: [&str; 4] = ["street/xe_may_01_chay.glb", "street/xe_may_02_chay.glb", "street/xe_may_03_chay.glb", "street/xe_may_04_chay.glb"];
            let delivered: Vec<&str> = BIKES.iter().copied().filter(|f| hq.get(f).is_some()).collect();
            if let Some(f) = delivered.get(i % delivered.len().max(1))
                && let Some(e) = hq::prop(commands, hq, &[f], Transform::default())
            {
                return e;
            }
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

#[allow(clippy::too_many_arguments)]
fn start_raid(
    commands: &mut Commands,
    assets: &AssetServer,
    hq: &hq::Hq,
    lib: &AnimLib,
    props: &Props,
    facades: &facade::Facades,
    g: &mut Game,
    mission: Mission,
) {
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
    raid.player.pos = open_start(&g.map, facades, &raid);
    g.cam_target = world_pos(raid.player.pos);
    // Looking the way to go: towards the first goal (the guide's beam is in view from the start).
    g.cam_yaw = raid.goal(&g.map).map(|(at, _)| at - raid.player.pos).filter(|d| d.x != 0.0 || d.y != 0.0).map_or(PI, |d| d.x.atan2(d.y));
    g.raid = Some(raid);
    g.mission = Some(mission);
    end_raid(commands, g);
    spawn_raid(commands, assets, hq, lib, props, g);
    g.screen = Screen::Playing;
}

/// Where the player starts: where the raid put them, unless that is against a shopfront (the
/// camera then has no room behind them); then the nearest open spot within 8 m with 2 m of
/// pavement in front of any shop, and still more than 12 m from every thief.
fn open_start(map: &Map, facades: &facade::Facades, raid: &Raid) -> V2 {
    let p0 = raid.player.pos;
    let open = |q: V2| !facades.in_shopfront(world_pos(q) + Vec3::Y, 2.0);
    if open(p0) {
        return p0;
    }
    let mut best: Option<(i32, V2)> = None;
    for dy in -4..=4 {
        for dx in -4..=4 {
            let q = V2 { x: p0.x + dx as f32, y: p0.y + dy as f32 };
            let d = dx * dx + dy * dy;
            if best.is_some_and(|(b, _)| b <= d) || !map.circle_free(q, 0.4) || !open(q) {
                continue;
            }
            if raid.thieves.iter().all(|t| t.pos.dist(q) * TILE_M > 12.0) {
                best = Some((d, q));
            }
        }
    }
    best.map_or(p0, |(_, q)| q)
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
    (motion, scroll, mut doors, facades): (Res<AccumulatedMouseMotion>, Res<AccumulatedMouseScroll>, Query<&mut cage::CageDoor>, Res<facade::Facades>),
) {
    let dt = time.delta_secs().min(0.05);
    g.t += dt;
    let (w, h) = (window.width(), window.height());
    let hudl = Hud::new(w, h, [0.0; 4]);
    let cam_e = *camera;

    if let Some(l) = g.look {
        if let Ok(mut c) = tf.get_mut(cam_e) {
            *c = Transform::from_translation(l.eye).looking_at(l.target, Vec3::Y);
        }
        return;
    }
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
        attack: keys.pressed(KeyCode::KeyJ) || (g.locked && mouse.pressed(MouseButton::Left)),
        throw: keys.just_pressed(KeyCode::KeyK) || keys.just_pressed(KeyCode::KeyQ) || (g.locked && mouse.just_pressed(MouseButton::Right)),
        roll: keys.just_pressed(KeyCode::Space),
        interact: keys.pressed(KeyCode::KeyE),
        ..default()
    };
    let hit = |b: (V2, f32), p: Vec2| Vec2::new(b.0.x, b.0.y).distance(p) < b.1 * 1.25;
    let mut pause = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP);
    if touches.iter_just_pressed().next().is_some() {
        g.touch_seen = true;
    }
    // Mouse look while the pointer is captured; the wheel moves the camera in and out.
    if g.locked {
        let d = motion.delta;
        g.cam_yaw -= d.x * 0.0028;
        g.cam_pitch = (g.cam_pitch + d.y * 0.0022).clamp(-0.05, 1.25);
    }
    if scroll.delta.y != 0.0 {
        let k = if matches!(scroll.unit, bevy::input::mouse::MouseScrollUnit::Line) { 0.6 } else { 0.01 };
        g.cam_dist = (g.cam_dist - scroll.delta.y * k).clamp(2.5, 14.0);
    }
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
        if let Some((id, s, _)) = g.joy
            && id == t.id()
        {
            g.joy = Some((id, s, t.position()));
        }
        if let Some((id, last)) = g.drag
            && id == t.id()
        {
            let d = t.position() - last;
            g.cam_yaw -= d.x * 0.008;
            g.drag = Some((id, t.position()));
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
    // Mouse without capture: drag to turn the camera.
    if !g.locked && (mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Right)) {
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
            // A caged dog stays inside its cage, which may have been moved off a kerb.
            let shift = if d.state == DogState::Caged {
                raid.cages.iter().position(|c| c.dog == i).and_then(|c| g.cages.get(c)).and_then(|&c| doors.get(c).ok()).map_or(Vec3::ZERO, |c| c.shift)
            } else {
                Vec3::ZERO
            };
            let target = world_pos(d.pos) + shift;
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
    // A freed dog's cage: a delivered one swings its door open (see `cage::swing`), the stand-in
    // box goes.
    for (i, cg) in raid.cages.iter().enumerate() {
        let Some(&e) = g.cages.get(i) else { continue };
        if !cg.open {
            continue;
        }
        match doors.get_mut(e) {
            Ok(mut d) if d.model => d.open = true,
            _ => {
                if let Ok(mut v) = vis.get_mut(e) {
                    *v = Visibility::Hidden;
                }
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
            if show && let Ok(mut t) = tf.get_mut(e) {
                let (q, dir) = rail.at(d);
                *t = Transform::from_translation(world_pos(q) + Vec3::Y * 1.9).with_rotation(Quat::from_rotation_y(dir.x.atan2(dir.y))).with_scale(Vec3::new(
                    1.0,
                    1.0,
                    TILE_M * 2.0 - 0.3,
                ));
            }
        }
    }
    // Camera: behind and above the player, easing round when he runs (unless the mouse steers it).
    if p.moving && g.drag.is_none() && !g.locked {
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
    let target = g.cam_target.lerp(ppos + Vec3::Y * 1.45, (dt * 8.0).min(1.0));
    g.cam_target = target;
    let yaw = g.cam_yaw;
    let shake = if raid.shake > 0.0 { Vec3::new((g.t * 90.0).sin(), (g.t * 77.0).cos(), 0.0) * 0.08 } else { Vec3::ZERO };
    if let Ok(mut t) = tf.get_mut(cam_e) {
        let pitch = g.cam_pitch;
        let back = Vec3::new(-yaw.sin() * pitch.cos(), pitch.sin(), -yaw.cos() * pitch.cos());
        let dist = camera_room(&g.map, &facades, target, back, g.cam_dist);
        let eye = target + back * dist;
        *t = Transform::from_translation(Vec3::new(eye.x, eye.y.max(0.4), eye.z) + shake).looking_at(target, Vec3::Y);
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

/// The loading screen ends once the list of delivered art has been read and the moves are ready.
fn loading(mut commands: Commands, assets: Res<AssetServer>, mut g: ResMut<Game>, time: Res<Time>, lib: Res<AnimLib>, hq: Res<hq::Hq>) {
    if let Some(mut l) = g.look
        && !l.spawned
        && hq.resolved()
        && let Some((at, open)) = l.cage
    {
        const CAGE: [&str; 1] = ["people/long_cho.glb"];
        let facing = Quat::from_rotation_arc(Vec3::Z, (l.eye - at).with_y(0.0).normalize_or(Vec3::Z));
        if let Some(e) = hq::prop(&mut commands, &hq, &CAGE, Transform::from_translation(at).with_rotation(facing)) {
            let mut door = cage::CageDoor::new(true);
            door.open = open;
            commands.entity(e).insert(door);
        }
        // At night a player sees the cage in their torch's light: the same light, held where a
        // player standing at the camera would hold it.
        if night::from_page() == Some(true) {
            commands.spawn((
                SpotLight {
                    intensity: 30_000.0,
                    range: 28.0,
                    color: Color::srgb(1.0, 0.96, 0.88),
                    inner_angle: 0.22,
                    outer_angle: 0.5,
                    shadow_maps_enabled: render::HIGH,
                    ..default()
                },
                Transform::from_translation(l.eye.with_y(1.15)).looking_at(at + Vec3::Y * 0.3, Vec3::Y),
            ));
        }
        l.spawned = true;
        g.look = Some(l);
    }
    if let Some(mut l) = g.look
        && !l.spawned
        && hq.resolved()
        && lib.graph.is_some()
        && lib.dog_graph.is_some()
        && let Some((at, right)) = l.lineup
    {
        let facing = Quat::from_rotation_arc(Vec3::Z, (l.eye - at).with_y(0.0).normalize_or(Vec3::Z));
        let spots = [at - right * 0.9, at + right * 0.9, at + (l.eye - at).with_y(0.0).normalize_or(Vec3::Z) * 1.4];
        let minh = actors::spawn_person(&mut commands, &assets, &hq, "minh", "Hair_SimpleParted", spots[0]);
        let thief = actors::spawn_person(&mut commands, &assets, &hq, "trom", "Hair_Buzzed", spots[1]);
        let dog = actors::spawn_dog(&mut commands, &assets, &lib, spots[2]);
        for (e, p) in [(minh, spots[0]), (thief, spots[1])] {
            commands.entity(e).insert(Transform::from_translation(p).with_rotation(facing));
        }
        commands.entity(dog).insert(Transform::from_translation(spots[2]).with_rotation(facing).with_scale(Vec3::splat(lib.dog_scale)));
        l.spawned = true;
        g.look = Some(l);
    }
    if g.screen != Screen::Loading {
        return;
    }
    g.loaded_t += time.delta_secs().min(0.05);
    let ready = hq.resolved() && lib.graph.is_some() && lib.dog_graph.is_some();
    if (ready && g.loaded_t > 0.5) || g.loaded_t > 30.0 {
        g.screen = if g.profile.seen_intro { Screen::Title } else { Screen::Story(0, 0) };
    }
}

/// How far the camera can sit behind `target` along `back` before a building gets in the way,
/// so it never looks out from inside a wall.
/// Tips shown once each, the first time their moment comes (see `fill_guide`).
#[derive(Default)]
struct Tips {
    shown: std::collections::HashSet<&'static str>,
    queue: std::collections::VecDeque<String>,
    now: Option<(String, f32)>,
}

impl Tips {
    fn offer(&mut self, id: &'static str, text: impl Into<String>) {
        if self.shown.insert(id) {
            self.queue.push_back(text.into());
        }
    }
}

/// How long a tip stays up (seconds).
const TIP_S: f32 = 7.0;

/// What the guide shows (see `guide`): where to go next and what is there, each thief's view, and
/// a tip the first time each thing happens.
fn fill_guide(g: Res<Game>, mut guide: ResMut<guide::Guide>, time: Res<Time>, mut tips: Local<Tips>) {
    let raid = g.raid.as_ref().filter(|_| matches!(g.screen, Screen::Playing | Screen::Paused) && g.look.is_none());
    guide.on = raid.is_some();
    let Some(r) = raid else {
        guide.goal = None;
        guide.cones.clear();
        guide.tip = None;
        tips.now = None;
        tips.queue.clear();
        return;
    };
    let exit_name = |id| g.map.exits.iter().find(|e| e.id == id).map_or("điểm thoát", |e| e.name);
    let goal = r.goal(&g.map);
    guide.goal = goal.map(|(at, kind)| {
        use hanoi_rescue::raid::Goal;
        let (mark, label) = match kind {
            Goal::Cage => (guide::Mark::Get, "Lồng chó".to_string()),
            Goal::Dog => (guide::Mark::Get, "Chú chó".to_string()),
            Goal::Clue => (guide::Mark::Get, "Cuốn sổ của bọn trộm".to_string()),
            Goal::Boss => (guide::Mark::Get, r.setup.boss.map_or("Trùm", |b| b.name()).to_string()),
            Goal::Exit(id) => (guide::Mark::Out, exit_name(id).to_string()),
        };
        (world_pos(at), mark, label)
    });
    guide.cones.clear();
    for t in &r.thieves {
        let mood = match t.state {
            TState::Stunned(_) | TState::Down(_) | TState::Tied => continue,
            TState::Chase | TState::Windup(_) | TState::Charge(_) => guide::Mood::Hunting,
            TState::Investigate | TState::Search(_) => guide::Mood::Wary,
            _ if t.alert > 0.3 => guide::Mood::Wary,
            _ => guide::Mood::Calm,
        };
        let (range, half) = r.sight_cone(t);
        guide.cones.push(guide::Cone { at: world_pos(t.pos), turn: yaw_of(t.facing), range: range * TILE_M, half, mood });
    }

    // Tips, in the order things usually happen.
    use hanoi_rescue::raid::Goal;
    match goal.map(|g| g.1) {
        Some(Goal::Clue) => tips.offer("clue", "Tìm cuốn sổ của bọn trộm: cột sáng vàng chỉ chỗ. WASD để đi, di chuột để nhìn."),
        Some(Goal::Boss) => tips.offer("boss", "Hạ gục tên trùm (chuột trái), rồi giữ E để trói hắn. Cột sáng vàng chỉ chỗ hắn."),
        _ => tips.offer("start", "Đi theo cột sáng vàng: chú chó bị nhốt ở đó. WASD để đi, di chuột để nhìn, Shift để đi lén."),
    }
    let me = r.player.pos;
    if r.thieves.iter().any(|t| t.pos.dist(me) * TILE_M < 16.0 && !matches!(t.state, TState::Down(_) | TState::Tied)) {
        tips.offer("thief", "Có trộm gần đây! Vùng sáng trước mặt hắn là tầm nhìn: đừng bước vào. Giữ Shift để lén vòng sau lưng.");
    }
    if matches!(r.context, Context::Cage { .. }) {
        tips.offer("cage", "Giữ E để mở lồng.");
    }
    if r.thieves.iter().any(|t| matches!(t.state, TState::Chase)) {
        tips.offer("seen", "Bị phát hiện (dấu !)! Chạy đi; chuột phải ném dép làm choáng, Space để lăn né.");
    }
    if r.objective_met()
        && let Some((_, Goal::Exit(id))) = goal
    {
        tips.offer("out", format!("Xong rồi! Đi theo cột sáng xanh để thoát ra {}.", exit_name(id)));
    }
    if r.player.hp < r.setup.stats.max_hp * 0.4 {
        tips.offer("hurt", "Máu thấp: tránh đánh nhau, lẩn vào đám đông hoặc ngõ nhỏ.");
    }
    let dt = time.delta_secs();
    let expired = match tips.now.as_mut() {
        Some((_, left)) => {
            *left -= dt;
            *left <= 0.0
        }
        None => true,
    };
    if expired {
        tips.now = tips.queue.pop_front().map(|t| (t, TIP_S));
    }
    guide.tip = tips.now.as_ref().map(|t| t.0.clone());
}

/// The game's speed from the page's address (`?speed=0.1` runs it at a tenth of real time, for
/// recording video on a computer too slow to draw it live), between 0.001 and 4. The game moves at
/// most 0.05 s per frame, so for smooth video pick about 1 / (24 × seconds a frame takes to draw).
fn game_speed(q: &str) -> Option<f32> {
    q.parse::<f32>().ok().filter(|s| s.is_finite() && *s > 0.0).map(|s| s.clamp(0.001, 4.0))
}

/// Night for the chapters the story sets after dark (or what the address asks for), the lights
/// nearest the camera, and a torch in Minh's hand at night.
fn choose_sky(
    mut commands: Commands,
    g: Res<Game>,
    mut sky: ResMut<night::Sky>,
    cam: Query<&GlobalTransform, With<MainCamera>>,
    torches: Query<(Entity, &ChildOf), With<night::Torch>>,
    mut forced: Local<Option<Option<bool>>>,
) {
    let forced = *forced.get_or_insert_with(night::from_page);
    let in_raid = matches!(g.screen, Screen::Playing | Screen::Paused | Screen::Result);
    let story_night = matches!(g.mission, Some(Mission::Story(ch)) if NIGHT_CHAPTERS.contains(&ch));
    let night = forced.unwrap_or(in_raid && story_night);
    if sky.night != night {
        sky.night = night;
    }
    if let Ok(t) = cam.single() {
        sky.focus = t.translation();
    }
    let holder = if night { g.player } else { None };
    let mut held = false;
    for (e, parent) in &torches {
        if Some(parent.parent()) == holder {
            held = true;
        } else {
            commands.entity(e).despawn();
        }
    }
    if !held && let Some(p) = holder {
        night::torch(&mut commands, p);
    }
}

/// How far behind the player the camera can sit (up to `want`): it stops short of buildings, and
/// of shopfronts' counters, which reach out over the pavement the walking map counts as open.
fn camera_room(map: &Map, facades: &facade::Facades, target: Vec3, back: Vec3, want: f32) -> f32 {
    let mut d = 0.0;
    while d < want {
        let q = target + back * (d + 0.6);
        let tile = map.tile_at(V2 { x: q.x / TILE_M, y: q.z / TILE_M });
        let walled = matches!(tile, Tile::Building | Tile::Market | Tile::Landmark) && q.y < 14.0;
        if walled || facades.in_shopfront(q, facade::SHOPFRONT_DEPTH_M) {
            return d.max(1.2);
        }
        d += 0.25;
    }
    want
}

/// Captures the mouse when the player clicks into a raid and lets it go on menus. When the browser
/// drops the capture (the player pressed Esc), the game pauses.
fn pointer_lock(
    mut g: ResMut<Game>,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut since: Local<f32>,
    mut seen: Local<bool>,
) {
    let playing = g.screen == Screen::Playing;
    if playing && !g.locked && !g.touch_seen && mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
        g.locked = true;
        g.drag = None;
        *since = 0.0;
        *seen = false;
        return;
    }
    if !g.locked {
        return;
    }
    *since += time.delta_secs();
    let held = page_has_pointer_lock();
    *seen |= held;
    // The browser grants the lock a frame or two after the click; it may also refuse it.
    let released = (*seen && !held) || (!*seen && *since > 2.0);
    if !playing || released {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
        g.locked = false;
        if playing && *seen {
            g.screen = Screen::Paused;
        }
    }
}

/// Whether the page holds the pointer lock (always true off the web).
fn page_has_pointer_lock() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return web_sys::window().and_then(|w| w.document()).is_some_and(|d| d.pointer_lock_element().is_some());
    }
    #[allow(unreachable_code)]
    true
}

/// What the HUD updates on each of its pieces.
type HudNode =
    (&'static Ui, &'static mut Node, &'static mut Visibility, Option<&'static mut Text>, Option<&'static mut BackgroundColor>, Option<&'static Children>);

/// The heads-up display during a raid, and the loading text.
#[allow(clippy::too_many_arguments)]
fn hud(
    g: Res<Game>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut nodes: Query<HudNode, Without<Label3d>>,
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
        let touch_ui = matches!(kind, Ui::Attack | Ui::Throw | Ui::Roll | Ui::Pause | Ui::JoyBase | Ui::JoyKnob);
        let mut show = playing && (!touch_ui || g.touch_seen);
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
                show = playing && !g.touch_seen;
                place(&mut node, w / 2.0 - 470.0, h - 26.0, 940.0, 20.0);
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
                            t.0 = if g.touch_seen { l.to_string() } else { format!("E · {l}") };
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
            // Names stay clear of the HUD along the top.
            Ok(p) if on && p.x > 0.0 && p.y > 150.0 && p.x < w && p.y < h => {
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
    (hq, lib, facades): (Res<hq::Hq>, Res<AnimLib>, Res<facade::Facades>),
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
                    start_raid(&mut commands, &assets, &hq, &lib, &props, &facades, &mut g, m);
                }
            }
            (MenuButton::Buy(i), _) => {
                if let Some(u) = UPGRADES.get(i)
                    && g.profile.buy(u.kind)
                {
                    save_profile(&g.profile);
                    g.shown = None;
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
    if g.look.is_some() {
        g.shown = Some(g.screen);
        for r in &roots {
            commands.entity(r).despawn();
        }
        return;
    }
    g.shown = Some(g.screen);
    for r in &roots {
        commands.entity(r).despawn();
    }
    let f = &font.0;
    // A full-screen layer that centres the panel, so short landscape screens never cut it off.
    let panel = |commands: &mut Commands| {
        let inner = commands
            .spawn((
                Node {
                    width: Val::Px(360.0),
                    max_width: Val::Vw(92.0),
                    max_height: Val::Vh(94.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.07, 0.1, 0.9)),
            ))
            .id();
        commands
            .spawn((
                MenuRoot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .add_child(inner);
        inner
    };
    let button = |commands: &mut Commands, parent: Entity, label: &str, kind: MenuButton, primary: bool| {
        let b = commands
            .spawn((
                Button,
                kind,
                Node {
                    height: Val::Px(46.0),
                    flex_shrink: 0.0,
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

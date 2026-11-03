//! People, dogs and props: CC0 characters animated with the Universal Animation Library, the dog,
//! cages, crates, motorbikes and the train, kept in step with the game's raid each frame.

use std::time::Duration;

use bevy::animation::{AnimatedBy, AnimationTargetId};
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::world_serialization::{WorldAssetRoot, WorldInstanceReady};

use crate::hq::{Hq, HqState};

/// The moves a person can make, each one clip of the animation library.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Move {
    Idle,
    Walk,
    Jog,
    Sprint,
    Sneak,
    SneakIdle,
    Punch,
    Roll,
    Hit,
    Down,
    Interact,
    Ride,
}

impl Move {
    fn clip(self) -> &'static str {
        match self {
            Move::Idle => "Idle_Loop",
            Move::Walk => "Walk_Loop",
            Move::Jog => "Jog_Fwd_Loop",
            Move::Sprint => "Sprint_Loop",
            Move::Sneak => "Crouch_Fwd_Loop",
            Move::SneakIdle => "Crouch_Idle_Loop",
            Move::Punch => "Punch_Cross",
            Move::Roll => "Roll",
            Move::Hit => "Hit_Chest",
            Move::Down => "Death01",
            Move::Interact => "Interact",
            Move::Ride => "Sitting_Idle_Loop",
        }
    }
    fn looping(self) -> bool {
        !matches!(self, Move::Punch | Move::Roll | Move::Hit | Move::Down | Move::Interact)
    }
    pub const ALL: [Move; 12] = [
        Move::Idle,
        Move::Walk,
        Move::Jog,
        Move::Sprint,
        Move::Sneak,
        Move::SneakIdle,
        Move::Punch,
        Move::Roll,
        Move::Hit,
        Move::Down,
        Move::Interact,
        Move::Ride,
    ];
}

/// Each move and its node in an animation graph.
pub type MoveNodes = Vec<(Move, AnimationNodeIndex)>;

#[derive(Resource)]
pub struct AnimLib {
    pub gltf: Handle<Gltf>,
    pub dog_gltf: Handle<Gltf>,
    /// The dog's model file and its scale to metres (the stand-in fox is in centimetres-ish units).
    pub dog_path: String,
    pub dog_scale: f32,
    pub graph: Option<(Handle<AnimationGraph>, MoveNodes)>,
    pub dog_graph: Option<(Handle<AnimationGraph>, [AnimationNodeIndex; 3])>,
}

/// Builds the animation graphs once the libraries have loaded.
pub fn build_graphs(mut lib: ResMut<AnimLib>, gltfs: Res<Assets<Gltf>>, mut graphs: ResMut<Assets<AnimationGraph>>) {
    if lib.graph.is_none()
        && let Some(g) = gltfs.get(&lib.gltf)
    {
        let mut graph = AnimationGraph::new();
        let mut nodes = Vec::new();
        for m in Move::ALL {
            if let Some(clip) = g.named_animations.get(m.clip()) {
                nodes.push((m, graph.add_clip(clip.clone(), 1.0, graph.root)));
            }
        }
        lib.graph = Some((graphs.add(graph), nodes));
    }
    if lib.dog_graph.is_none()
        && let Some(g) = gltfs.get(&lib.dog_gltf)
    {
        let mut graph = AnimationGraph::new();
        let pick = |names: &[&str], i: usize| names.iter().find_map(|n| g.named_animations.get(*n).cloned()).or_else(|| g.animations.get(i).cloned());
        let (Some(a), Some(b), Some(c)) = (pick(&["Idle", "Survey"], 0), pick(&["Walk"], 1), pick(&["Run"], 2)) else { return };
        let n = [graph.add_clip(a, 1.0, graph.root), graph.add_clip(b, 1.0, graph.root), graph.add_clip(c, 1.0, graph.root)];
        lib.dog_graph = Some((graphs.add(graph), n));
    }
}

/// A person in the world: its body's animation player, once spawned.
#[derive(Component, Default)]
pub struct Person {
    pub player: Option<Entity>,
    pub current: Option<Move>,
    pub hair: &'static str,
    /// A delivered model that brings its own moves (ASSETS.md §5, "own" skeleton).
    pub own: Option<Handle<Gltf>>,
    /// The moves of an `own` model, once its graph is built.
    pub nodes: Option<Vec<(Move, AnimationNodeIndex)>>,
}

/// When the delivered art has been read: the dog takes its delivered model.
pub fn adopt_hq(hq: Res<Hq>, mut lib: ResMut<AnimLib>, assets: Res<AssetServer>, mut done: Local<bool>) {
    if *done || hq.state == HqState::Waiting {
        return;
    }
    *done = true;
    if let Some(e) = hq.get("animals/cho_co.glb") {
        lib.dog_path = Hq::path(&e.file);
        lib.dog_gltf = assets.load(lib.dog_path.clone());
        lib.dog_scale = 1.0;
        lib.dog_graph = None;
    }
}

/// Delivered models for each role, in the order they're preferred.
fn hq_files(who: &str) -> &'static [&'static str] {
    match who {
        "trom" => &["people/trom_01.glb"],
        "trom2" => &["people/trom_02.glb", "people/trom_01.glb"],
        "dai_ca" => &["people/bay_thong_long.glb", "people/tu_long.glb"],
        "co_hang" => &["people/co_hang.glb"],
        "nguoi_di_duong" => &["people/nguoi_di_duong_01.glb"],
        _ => &["people/minh.glb"],
    }
}

#[derive(Component)]
pub struct Dog {
    pub player: Option<Entity>,
    pub current: usize,
}

/// Links an accessory's bones (hair) to the body's animation player.
#[derive(Component)]
struct LinkTo(Entity);

pub fn spawn_person(commands: &mut Commands, assets: &AssetServer, hq: &Hq, who: &'static str, hair: &'static str, at: Vec3) -> Entity {
    if let Some(e) = hq.first(hq_files(who)) {
        // Delivered: its hair is part of it; its moves are the library's or its own.
        let path = Hq::path(&e.file);
        let own = (e.skeleton == "own").then(|| assets.load::<Gltf>(path.clone()));
        return commands
            .spawn((
                Transform::from_translation(at),
                Visibility::default(),
                Person { own, ..default() },
                WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path))),
            ))
            .observe(on_body_ready)
            .id();
    }
    let path = match who {
        "trom" => "people/trom/person.gltf",
        "trom2" => "people/trom2/person.gltf",
        "dai_ca" => "people/dai_ca/person.gltf",
        "co_hang" => "people/co_hang/person.gltf",
        "nguoi_di_duong" => "people/nguoi_di_duong/person.gltf",
        _ => "people/minh/person.gltf",
    };
    commands
        .spawn((
            Transform::from_translation(at),
            Visibility::default(),
            Person { hair, ..default() },
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path))),
        ))
        .observe(on_body_ready)
        .id()
}

fn hair_path(hair: &str) -> Option<&'static str> {
    match hair {
        "Hair_SimpleParted" => Some("hair/Hair_SimpleParted/Hair_SimpleParted.gltf"),
        "Hair_Buzzed" => Some("hair/Hair_Buzzed/Hair_Buzzed.gltf"),
        "Hair_Beard" => Some("hair/Hair_Beard/Hair_Beard.gltf"),
        "Hair_Buns" => Some("hair/Hair_Buns/Hair_Buns.gltf"),
        _ => None,
    }
}

/// Gives every bone under `root` the id the animation library uses for it, driven by `player`.
fn register_bones(commands: &mut Commands, root: Entity, player: Entity, children: &Query<&Children>, names: &Query<&Name>) {
    let mut stack: Vec<(Entity, Vec<Name>)> = vec![];
    if let Ok(n) = names.get(root) {
        stack.push((root, vec![n.clone()]));
    }
    while let Some((e, path)) = stack.pop() {
        commands.entity(e).insert((AnimationTargetId::from_names(path.iter()), AnimatedBy(player)));
        if let Ok(kids) = children.get(e) {
            for k in kids.iter() {
                if let Ok(n) = names.get(k) {
                    let mut p = path.clone();
                    p.push(n.clone());
                    stack.push((k, p));
                }
            }
        }
    }
}

fn find_named(root: Entity, name: &str, children: &Query<&Children>, names: &Query<&Name>) -> Option<Entity> {
    if names.get(root).is_ok_and(|n| n.as_str() == name) {
        return Some(root);
    }
    children.iter_descendants(root).find(|e| names.get(*e).is_ok_and(|n| n.as_str() == name))
}

#[allow(clippy::too_many_arguments)]
fn on_body_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    lib: Res<AnimLib>,
    assets: Res<AssetServer>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut people: Query<&mut Person>,
    with_player: Query<(), With<AnimationPlayer>>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let root = ready.entity;
    let own = people.get(root).ok().and_then(|p| p.own.clone());
    if let Some(own) = own {
        // A model with its own moves: drive the player its file brought, with a graph of its clips.
        let Some(pe) = children.iter_descendants(root).find(|e| with_player.contains(*e)) else { return };
        let Some(g) = gltfs.get(&own) else { return };
        let mut graph = AnimationGraph::new();
        let nodes: Vec<(Move, AnimationNodeIndex)> =
            Move::ALL.iter().filter_map(|m| g.named_animations.get(m.clip()).map(|c| (*m, graph.add_clip(c.clone(), 1.0, graph.root)))).collect();
        let mut player = AnimationPlayer::default();
        let mut transitions = AnimationTransitions::new();
        if let Some((_, idle)) = nodes.iter().find(|(m, _)| *m == Move::Idle) {
            transitions.play(&mut player, *idle, Duration::ZERO).repeat();
        }
        commands.entity(pe).insert((player, transitions, AnimationGraphHandle(graphs.add(graph))));
        if let Ok(mut p) = people.get_mut(root) {
            p.player = Some(pe);
            p.current = Some(Move::Idle);
            p.nodes = Some(nodes);
        }
        return;
    }
    let Some(armature) = find_named(root, "Armature", &children, &names) else { return };
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();
    if let Some((graph, nodes)) = &lib.graph {
        if let Some((_, idle)) = nodes.iter().find(|(m, _)| *m == Move::Idle) {
            transitions.play(&mut player, *idle, Duration::ZERO).repeat();
        }
        commands.entity(armature).insert(AnimationGraphHandle(graph.clone()));
    }
    commands.entity(armature).insert((player, transitions));
    register_bones(&mut commands, armature, armature, &children, &names);
    if let Ok(mut p) = people.get_mut(root) {
        p.player = Some(armature);
        p.current = Some(Move::Idle);
        if let Some(path) = hair_path(p.hair) {
            let hair = commands
                .spawn((Transform::default(), Visibility::default(), LinkTo(armature), WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path)))))
                .observe(on_hair_ready)
                .id();
            commands.entity(root).add_child(hair);
        }
    }
}

fn on_hair_ready(ready: On<WorldInstanceReady>, mut commands: Commands, links: Query<&LinkTo>, children: Query<&Children>, names: Query<&Name>) {
    let Ok(LinkTo(player)) = links.get(ready.entity) else { return };
    if let Some(armature) = find_named(ready.entity, "Armature", &children, &names) {
        register_bones(&mut commands, armature, *player, &children, &names);
    }
}

/// Plays `m` on a person, blending from what they were doing.
pub fn set_move(person: &mut Person, m: Move, lib: &AnimLib, players: &mut Query<(&mut AnimationPlayer, &mut AnimationTransitions)>) {
    if person.current == Some(m) {
        return;
    }
    let Some(nodes) = person.nodes.as_ref().or(lib.graph.as_ref().map(|g| &g.1)) else { return };
    let Some(e) = person.player else { return };
    let Some((_, node)) = nodes.iter().find(|(k, _)| *k == m) else { return };
    let Ok((mut player, mut tr)) = players.get_mut(e) else { return };
    let blend = if matches!(m, Move::Punch | Move::Roll) { 80 } else { 180 };
    let anim = tr.play(&mut player, *node, Duration::from_millis(blend));
    if m.looping() {
        anim.repeat();
    }
    if m == Move::Roll {
        anim.set_speed(1.6);
    }
    person.current = Some(m);
}

pub fn spawn_dog(commands: &mut Commands, assets: &AssetServer, lib: &AnimLib, at: Vec3) -> Entity {
    commands
        .spawn((
            Transform::from_translation(at).with_scale(Vec3::splat(lib.dog_scale)),
            Visibility::default(),
            Dog { player: None, current: 0 },
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(lib.dog_path.clone()))),
        ))
        .observe(on_dog_ready)
        .id()
}

fn on_dog_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    lib: Res<AnimLib>,
    children: Query<&Children>,
    players: Query<Entity, With<AnimationPlayer>>,
    mut dogs: Query<&mut Dog>,
) {
    let Some(pe) = children.iter_descendants(ready.entity).find(|e| players.get(*e).is_ok()) else { return };
    if let Some((graph, nodes)) = &lib.dog_graph {
        let mut player = AnimationPlayer::default();
        let mut tr = AnimationTransitions::new();
        tr.play(&mut player, nodes[0], Duration::ZERO).repeat();
        commands.entity(pe).insert((player, tr, AnimationGraphHandle(graph.clone())));
    }
    if let Ok(mut d) = dogs.get_mut(ready.entity) {
        d.player = Some(pe);
    }
}

pub fn set_dog_move(dog: &mut Dog, which: usize, lib: &AnimLib, players: &mut Query<(&mut AnimationPlayer, &mut AnimationTransitions)>) {
    if dog.current == which {
        return;
    }
    let (Some(e), Some((_, nodes))) = (dog.player, &lib.dog_graph) else { return };
    let Some(node) = nodes.get(which) else { return };
    if let Ok((mut player, mut tr)) = players.get_mut(e) {
        tr.play(&mut player, *node, Duration::from_millis(200)).repeat();
        dog.current = which;
    }
}

/// Shapes for props, built once.
#[derive(Resource)]
pub struct Props {
    pub cage: Handle<Mesh>,
    pub cage_mat: Handle<StandardMaterial>,
    pub crate_mesh: Handle<Mesh>,
    pub crate_mat: Handle<StandardMaterial>,
    pub book_mat: Handle<StandardMaterial>,
    pub slipper_mat: Handle<StandardMaterial>,
    pub small: Handle<Mesh>,
    pub bike_body: Handle<Mesh>,
    pub wheel: Handle<Mesh>,
    pub rider: Handle<Mesh>,
    pub helmet: Handle<Mesh>,
    pub bike_mats: Vec<Handle<StandardMaterial>>,
    pub tyre: Handle<StandardMaterial>,
    pub skin: Handle<StandardMaterial>,
    pub light: Handle<StandardMaterial>,
    pub car: Handle<Mesh>,
    pub car_mat: Handle<StandardMaterial>,
    pub exit_mesh: Handle<Mesh>,
    pub exit_mat: Handle<StandardMaterial>,
}

pub fn make_props(meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>) -> Props {
    let col = |mats: &mut Assets<StandardMaterial>, r: f32, g: f32, b: f32| {
        mats.add(StandardMaterial { base_color: Color::srgb(r, g, b), perceptual_roughness: 0.7, ..default() })
    };
    let glow = |mats: &mut Assets<StandardMaterial>, r: f32, g: f32, b: f32, a: f32| {
        mats.add(StandardMaterial {
            base_color: Color::srgba(r, g, b, a),
            emissive: crate::render::glow(r * 2.0, g * 2.0, b * 2.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })
    };
    Props {
        cage: meshes.add(Cuboid::new(0.9, 0.7, 0.7)),
        cage_mat: mats.add(StandardMaterial { base_color: Color::srgba(0.6, 0.62, 0.66, 0.45), alpha_mode: AlphaMode::Blend, metallic: 0.8, ..default() }),
        crate_mesh: meshes.add(Cuboid::new(0.6, 0.45, 0.45)),
        crate_mat: col(mats, 0.55, 0.38, 0.22),
        book_mat: mats.add(StandardMaterial { base_color: Color::srgb(0.8, 0.1, 0.1), emissive: crate::render::glow(2.0, 1.2, 0.2), ..default() }),
        slipper_mat: col(mats, 1.0, 0.45, 0.7),
        small: meshes.add(Cuboid::new(0.12, 0.05, 0.28)),
        bike_body: meshes.add(Cuboid::new(0.36, 0.5, 1.7)),
        wheel: meshes.add(Cylinder::new(0.3, 0.12)),
        rider: meshes.add(Capsule3d::new(0.22, 0.6)),
        helmet: meshes.add(Sphere::new(0.17)),
        bike_mats: vec![col(mats, 0.75, 0.1, 0.1), col(mats, 0.1, 0.3, 0.75), col(mats, 0.9, 0.9, 0.9), col(mats, 0.08, 0.08, 0.1), col(mats, 0.95, 0.75, 0.2)],
        tyre: col(mats, 0.05, 0.05, 0.05),
        skin: col(mats, 0.35, 0.33, 0.4),
        light: mats.add(StandardMaterial { base_color: Color::WHITE, emissive: crate::render::glow(8.0, 7.0, 5.0), ..default() }),
        car: meshes.add(Cuboid::new(3.1, 3.6, 1.0)),
        car_mat: col(mats, 0.18, 0.36, 0.62),
        exit_mesh: meshes.add(Cylinder::new(1.0, 0.08)),
        exit_mat: glow(mats, 0.2, 0.9, 0.55, 0.35),
    }
}

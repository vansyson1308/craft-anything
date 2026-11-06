//! The game's real art, made to `ASSETS.md` by an artist (or an art-making agent) and dropped into
//! `assets/hq/`. Nothing here draws anything: it finds what was delivered, puts each piece where
//! the city says it belongs, swaps it for the stand-in, and keeps it cheap to show the way big
//! open-world games do on the web: a level of detail by distance (LOD0, LOD1, LOD2, then a flat
//! billboard), and nothing spawned until it is near. Whatever isn't delivered keeps its stand-in.
//!
//! `hq/manifest.json` lists what exists; without it the game runs exactly as before.

use std::collections::{HashMap, HashSet};

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadState};
use bevy::color::LinearRgba;
use bevy::gltf::{Gltf, GltfMaterial, GltfMesh, GltfNode};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Affine2;
use bevy::prelude::*;
use serde::Deserialize;

use crate::world::{Mats, StandIn};

/// A JSON file (the manifest, a material's info.json).
#[derive(Asset, TypePath, Debug)]
pub struct JsonDoc(pub serde_json::Value);

#[derive(Default, TypePath)]
pub struct JsonLoader;

#[derive(Debug)]
pub struct JsonError(String);

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unreadable JSON: {}", self.0)
    }
}

impl std::error::Error for JsonError {}

impl AssetLoader for JsonLoader {
    type Asset = JsonDoc;
    type Settings = ();
    type Error = JsonError;
    async fn load(&self, reader: &mut dyn Reader, _settings: &(), _cx: &mut LoadContext<'_>) -> Result<JsonDoc, JsonError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await.map_err(|e| JsonError(e.to_string()))?;
        serde_json::from_slice(&bytes).map(JsonDoc).map_err(|e| JsonError(e.to_string()))
    }
    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}

/// One delivered piece, as `manifest.json` lists it.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Entry {
    pub file: String,
    #[serde(default)]
    pub lods: Vec<String>,
    #[serde(default)]
    pub size_m: Vec<f32>,
    /// A tree's billboard card: its size in metres and where the trunk's foot is on it (u from
    /// the left, v from the bottom). Without them the card is `size_m`, standing on its bottom edge.
    #[serde(default)]
    pub billboard_plane_size_m: Vec<f32>,
    #[serde(default)]
    pub billboard_pivot_uv: Vec<f32>,
    #[serde(default)]
    pub skeleton: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Manifest {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub assets: Vec<Entry>,
    /// Files the web packer renamed (`materials/co/orm.png` → `materials/co/orm.jpg`).
    #[serde(default)]
    pub renamed: HashMap<String, String>,
}

impl Manifest {
    /// Reads a manifest; anything malformed reads as "nothing delivered".
    pub fn parse(v: &serde_json::Value) -> Manifest {
        let m: Manifest = serde_json::from_value(v.clone()).unwrap_or_default();
        if m.version == 1 { m } else { Manifest::default() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HqState {
    Waiting,
    Ready,
    Absent,
}

/// What has been delivered.
#[derive(Resource)]
pub struct Hq {
    manifest: Handle<JsonDoc>,
    pub state: HqState,
    entries: HashMap<String, Entry>,
    renamed: HashMap<String, String>,
    waited: f32,
}

impl Hq {
    pub fn resolved(&self) -> bool {
        self.state != HqState::Waiting
    }
    /// The entry for a file under `hq/` (for example `people/minh.glb`), if it was delivered.
    pub fn get(&self, file: &str) -> Option<&Entry> {
        self.entries.get(file)
    }
    /// The first of `files` that was delivered.
    pub fn first(&self, files: &[&str]) -> Option<&Entry> {
        files.iter().find_map(|f| self.get(f))
    }
    pub fn path(file: &str) -> String {
        format!("hq/{file}")
    }
    /// The path to load a delivered file from, after any rename by the web packer.
    pub fn locate(&self, file: &str) -> String {
        Hq::path(self.renamed.get(file).map_or(file, String::as_str))
    }
}

pub struct HqPlugin;

impl Plugin for HqPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<JsonDoc>()
            .init_asset_loader::<JsonLoader>()
            .init_resource::<Flat>()
            .init_resource::<Applied>()
            .add_systems(PreStartup, |mut commands: Commands, assets: Res<AssetServer>| {
                commands.insert_resource(Hq {
                    manifest: assets.load("hq/manifest.json"),
                    state: HqState::Waiting,
                    entries: HashMap::new(),
                    renamed: HashMap::new(),
                    waited: 0.0,
                });
            })
            .add_systems(Update, (resolve, place_models, apply_materials, lod_switch).chain())
            .add_systems(Update, timed_emission.after(lod_switch).after(crate::choose_sky));
    }
}

/// Reads the manifest once it arrives (or gives up on it: no `hq/` means the stand-ins stay).
fn resolve(mut hq: ResMut<Hq>, assets: Res<AssetServer>, docs: Res<Assets<JsonDoc>>, time: Res<Time>) {
    if hq.resolved() {
        return;
    }
    hq.waited += time.delta_secs();
    match assets.load_state(&hq.manifest) {
        LoadState::Loaded => {
            let m = docs.get(&hq.manifest).map(|d| Manifest::parse(&d.0)).unwrap_or_default();
            info!("hq: {} delivered pieces", m.assets.len());
            hq.entries = m.assets.into_iter().map(|e| (e.file.clone(), e)).collect();
            hq.renamed = m.renamed;
            hq.state = if hq.entries.is_empty() { HqState::Absent } else { HqState::Ready };
        }
        LoadState::Failed(_) => {
            info!("hq: no manifest, using the stand-ins");
            hq.state = HqState::Absent;
        }
        _ if hq.waited > 15.0 => hq.state = HqState::Absent,
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------------
// Models as flat lists of parts, so a level of detail can be swapped by swapping a few entities.

/// One drawable piece of a model: its mesh, its material and where it sits in the model. A piece
/// under a jointed node (a door) sits on its hinge instead: `at` is then where it is on the hinge,
/// and the hinge is where it is in the model.
#[derive(Clone)]
pub struct Part {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub at: Transform,
    pub hinge: Option<(Transform, Joint)>,
}

/// Models already broken into parts, by glTF file; and every model file asked for so far.
#[derive(Resource, Default)]
pub struct Flat {
    models: HashMap<AssetId<Gltf>, Vec<Part>>,
    files: HashMap<String, Handle<Gltf>>,
    materials: HashMap<AssetId<GltfMaterial>, Handle<StandardMaterial>>,
}

/// A hinge an artist built into a model: a node whose glTF extras hold an `s3d_joint` of type
/// `revolute`. It turns about `axis` (in its own space) between `lower` and `upper` radians from
/// where it rests; something in the game (a cage's door) decides how far.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Joint {
    pub rest: Quat,
    pub axis: Vec3,
    pub lower: f32,
    pub upper: f32,
}

impl Joint {
    /// The joint in a node's glTF extras, resting at `rest`.
    pub fn from_extras(extras: &str, rest: Quat) -> Option<Joint> {
        let v: serde_json::Value = serde_json::from_str(extras).ok()?;
        let j = v.get("s3d_joint")?;
        if j.get("type")?.as_str()? != "revolute" {
            return None;
        }
        let a = j.get("axis")?.as_array()?;
        let f = |i: usize| a.get(i).and_then(|x| x.as_f64()).map(|x| x as f32);
        let axis = Vec3::new(f(0)?, f(1)?, f(2)?).try_normalize()?;
        let lower = j.get("lower")?.as_f64()? as f32;
        let upper = j.get("upper")?.as_f64()? as f32;
        (lower.is_finite() && upper.is_finite()).then_some(Joint { rest, axis, lower, upper })
    }

    /// The hinge's turn when it's `k` of the way (0 to 1) from its lower limit to its upper one.
    pub fn at(&self, k: f32) -> Quat {
        self.rest * Quat::from_axis_angle(self.axis, self.lower + (self.upper - self.lower) * k.clamp(0.0, 1.0))
    }
}

/// Breaks a loaded glTF into parts (static models only: no skins, no animation; hinges are kept).
fn flatten(
    gltf: &Gltf,
    nodes: &Assets<GltfNode>,
    meshes: &Assets<GltfMesh>,
    gmats: &Assets<GltfMaterial>,
    mats: &mut Assets<StandardMaterial>,
    cache: &mut HashMap<AssetId<GltfMaterial>, Handle<StandardMaterial>>,
) -> Vec<Part> {
    let children: HashSet<AssetId<GltfNode>> = gltf.nodes.iter().filter_map(|n| nodes.get(n)).flat_map(|n| n.children.iter().map(|c| c.id())).collect();
    // Each node with where it is: in the model, or on the hinge above it (then with the hinge).
    type Hinge = Option<(Transform, Joint)>;
    let mut stack: Vec<(Handle<GltfNode>, Transform, Hinge, u32)> =
        gltf.nodes.iter().filter(|n| !children.contains(&n.id())).map(|n| (n.clone(), Transform::IDENTITY, None, 0)).collect();
    let mut parts = Vec::new();
    while let Some((h, parent, hinge, depth)) = stack.pop() {
        let Some(node) = nodes.get(&h) else { continue };
        let mut at = parent.mul_transform(node.transform);
        let mut hinge = hinge;
        // A jointed node becomes the hinge for everything under it (the first one met: hinges on
        // hinges turn as one).
        if hinge.is_none()
            && let Some(j) = node.extras.as_ref().and_then(|x| Joint::from_extras(&x.value, at.rotation))
        {
            hinge = Some((at, j));
            at = Transform::IDENTITY;
        }
        if let Some(mesh) = node.mesh.as_ref().and_then(|m| meshes.get(m)) {
            for p in &mesh.primitives {
                let material = match &p.material {
                    Some(gm) => cache
                        .entry(gm.id())
                        .or_insert_with(|| mats.add(gmats.get(gm).map(bevy::pbr::gltf::standard_material_from_gltf_material).unwrap_or_default()))
                        .clone(),
                    None => cache.entry(AssetId::default()).or_insert_with(|| mats.add(StandardMaterial::default())).clone(),
                };
                parts.push(Part { mesh: p.mesh.clone(), material, at, hinge });
            }
        }
        if depth < 64 {
            stack.extend(node.children.iter().map(|c| (c.clone(), at, hinge, depth + 1)));
        }
    }
    parts
}

impl Flat {
    fn parts(&self, id: AssetId<Gltf>) -> Option<&Vec<Part>> {
        self.models.get(&id)
    }
}

/// Only the newly authored street and facade lights follow the day/night sky.
/// Other `lamp_glass` materials, such as the player's torch, are independent.
fn timed_emission_value(path: &str, name: &str, source: LinearRgba, night: bool) -> Option<LinearRgba> {
    let path = path.strip_prefix("hq/").unwrap_or(path);
    let street = matches!(path, "street/den_duong_01.glb" | "street/den_duong_01_lod1.glb");
    let upper = path.starts_with("buildings/mat_tien_tang_");
    let ground = path.starts_with("buildings/mat_tien_tret_");
    if (street && name == "lamp_glass") || (upper && name == "window_glow") || (ground && name == "shop_light") {
        Some(if night { source } else { LinearRgba::BLACK })
    } else {
        None
    }
}

/// Update loaded LOD materials as well as materials loaded after the last sky
/// change. Original emissive values come from the immutable glTF material.
fn timed_emission(
    sky: Res<crate::night::Sky>,
    flat: Res<Flat>,
    gltfs: Res<Assets<Gltf>>,
    gmats: Res<Assets<GltfMaterial>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    for (path, handle) in &flat.files {
        let Some(gltf) = gltfs.get(handle) else { continue };
        for (name, source_handle) in &gltf.named_materials {
            let (Some(source), Some(material_handle)) = (gmats.get(source_handle), flat.materials.get(&source_handle.id())) else { continue };
            let Some(want) = timed_emission_value(path, name, source.emissive, sky.night) else { continue };
            if mats.get(material_handle).is_some_and(|m| m.emissive != want)
                && let Some(mut material) = mats.get_mut(material_handle)
            {
                material.emissive = want;
            }
        }
    }
}

/// Fills [`Flat`] for every loaded model a [`LodModel`] uses.
#[allow(clippy::too_many_arguments)]
fn ensure_flat(
    flat: &mut Flat,
    handle: &Handle<Gltf>,
    gltfs: &Assets<Gltf>,
    nodes: &Assets<GltfNode>,
    meshes: &Assets<GltfMesh>,
    gmats: &Assets<GltfMaterial>,
    mats: &mut Assets<StandardMaterial>,
) -> bool {
    if flat.models.contains_key(&handle.id()) {
        return true;
    }
    let Some(g) = gltfs.get(handle) else { return false };
    let parts = flatten(g, nodes, meshes, gmats, mats, &mut flat.materials);
    flat.models.insert(handle.id(), parts);
    true
}

// ---------------------------------------------------------------------------------------------
// Levels of detail.

/// A placed model with levels of detail: `levels[i]` (a file under `assets/`) is drawn up to
/// `upto[i]` metres from the camera, then the billboard (if any) up to `billboard_upto`, then
/// nothing. A level's file is only downloaded the first time some model needs that level, so a
/// player who never goes near a landmark never downloads its finest model.
#[derive(Component)]
pub struct LodModel {
    pub levels: Vec<String>,
    pub upto: Vec<f32>,
    pub billboard: Option<(Handle<Mesh>, Handle<StandardMaterial>)>,
    pub billboard_upto: f32,
    shown: Option<usize>,
}

impl LodModel {
    pub fn new(levels: Vec<String>, upto: Vec<f32>) -> LodModel {
        LodModel { levels, upto, billboard: None, billboard_upto: 0.0, shown: None }
    }
    /// Which level to show at `d` metres: an index into `levels`, `levels.len()` for the
    /// billboard, or `None` for nothing.
    pub fn pick(&self, d: f32) -> Option<usize> {
        for (i, lim) in self.upto.iter().enumerate().take(self.levels.len()) {
            if d <= *lim {
                return Some(i);
            }
        }
        if self.billboard.is_some() && d <= self.billboard_upto { Some(self.levels.len()) } else { None }
    }
}

/// Marks the entities a [`LodModel`] spawned for its current level.
#[derive(Component)]
struct LodPart;

/// Switches levels of detail a few times a second.
#[allow(clippy::too_many_arguments)]
fn lod_switch(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<AssetServer>,
    mut next: Local<f32>,
    cam: Query<&GlobalTransform, With<Camera3d>>,
    mut models: Query<(Entity, &GlobalTransform, &mut LodModel, Option<&Children>)>,
    parts_q: Query<(), With<LodPart>>,
    mut flat: ResMut<Flat>,
    gltfs: Res<Assets<Gltf>>,
    nodes: Res<Assets<GltfNode>>,
    meshes: Res<Assets<GltfMesh>>,
    gmats: Res<Assets<GltfMaterial>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    *next -= time.delta_secs();
    if *next > 0.0 {
        return;
    }
    *next = 0.25;
    let Ok(cam) = cam.single() else { return };
    let c = cam.translation();
    for (e, at, mut lod, kids) in &mut models {
        let want = lod.pick(at.translation().distance(c));
        if want == lod.shown {
            continue;
        }
        // The level wanted is asked for; until it arrives, the nearest level already loaded stands
        // in (coarser first), so nothing blinks out while a finer file downloads.
        let n = lod.levels.len();
        let mut order: Vec<usize> = Vec::new();
        if let Some(w) = want {
            order.push(w);
            order.extend((w + 1)..n);
            order.extend((0..w.min(n)).rev());
        }
        let mut chosen: Option<(Option<usize>, Vec<Part>)> = if want.is_none() { Some((None, Vec::new())) } else { None };
        for (k, i) in order.iter().enumerate() {
            if *i >= n {
                // The billboard needs no download.
                let card = lod.billboard.iter().map(|(m, mat)| Part { mesh: m.clone(), material: mat.clone(), at: Transform::IDENTITY, hinge: None }).collect();
                chosen = Some((Some(*i), card));
                break;
            }
            let Some(path) = lod.levels.get(*i) else { continue };
            // Only the wanted level starts a download; the others are used if already here.
            let h = match (flat.files.get(path), k == 0) {
                (Some(h), _) => h.clone(),
                (None, true) => {
                    let h: Handle<Gltf> = assets.load(path.clone());
                    flat.files.insert(path.clone(), h.clone());
                    h
                }
                (None, false) => continue,
            };
            if ensure_flat(&mut flat, &h, &gltfs, &nodes, &meshes, &gmats, &mut mats) {
                chosen = Some((Some(*i), flat.parts(h.id()).cloned().unwrap_or_default()));
                break;
            }
        }
        let Some((showing, parts)) = chosen else { continue };
        if showing == lod.shown {
            continue;
        }
        if let Some(kids) = kids {
            for k in kids.iter().filter(|k| parts_q.contains(*k)) {
                commands.entity(k).despawn();
            }
        }
        for p in parts {
            let piece = commands.spawn((Mesh3d(p.mesh), MeshMaterial3d(p.material), p.at)).id();
            match p.hinge {
                // A hinge of its own (it holds the piece), which the game can turn.
                Some((at, joint)) => {
                    let hinge = commands.spawn((LodPart, at, Visibility::default(), joint)).add_child(piece).id();
                    commands.entity(e).add_child(hinge);
                }
                None => {
                    commands.entity(piece).insert(LodPart);
                    commands.entity(e).add_child(piece);
                }
            }
        }
        lod.shown = showing;
    }
}

/// Distances (metres) for each level of a kind of model.
pub fn lod_distances(kind: &str, levels: usize) -> Vec<f32> {
    let table: &[f32] = match kind {
        "tree" => &[35.0, 140.0],
        "landmark" => &[150.0, 450.0, 2500.0],
        "prop" => &[30.0, 90.0],
        _ => &[60.0, 200.0, 600.0],
    };
    (0..levels).map(|i| table.get(i).copied().unwrap_or(2500.0)).collect()
}

/// A delivered prop placed like a stand-in would be, if one of `files` was delivered (levels of detail by
/// distance, nothing until loaded).
pub fn prop(commands: &mut Commands, hq: &Hq, files: &[&str], at: Transform) -> Option<Entity> {
    let e = hq.first(files)?;
    let levels = levels_of(e);
    let mut upto = lod_distances("prop", levels.len());
    if let Some(last) = upto.last_mut() {
        *last = 400.0;
    }
    Some(commands.spawn((at, Visibility::default(), LodModel::new(levels, upto))).id())
}

/// An entry's levels, as paths under `assets/`: the file itself, then its `lods`.
pub fn levels_of(e: &Entry) -> Vec<String> {
    std::iter::once(&e.file).chain(e.lods.iter()).map(|f| Hq::path(f)).collect()
}

// ---------------------------------------------------------------------------------------------
// Placing landmarks and trees where the city says they are.

/// Where a delivered model goes: the stand-in it replaces, and its place in the world.
#[derive(Clone, Debug)]
pub struct Site {
    pub file: &'static str,
    pub kind: &'static str,
    pub at: Vec3,
    pub yaw: f32,
}

/// Every site found in the city.
#[derive(Resource, Default)]
pub struct Sites(pub Vec<Site>);

/// What has already been placed (each file once).
#[derive(Resource, Default)]
struct Applied {
    placed: HashSet<&'static str>,
    materials: bool,
}

/// A two-sided card facing every way (two crossed quads) for a tree seen from afar: `w` × `h`
/// metres, with the point (`pu`, `pv`) of the picture (from its left and its bottom) at the foot.
fn billboard_mesh(w: f32, h: f32, pu: f32, pv: f32) -> Mesh {
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::{Indices, PrimitiveTopology};
    let mut pos = Vec::new();
    let mut nrm = Vec::new();
    let mut uv = Vec::new();
    let mut idx = Vec::new();
    for (dx, dz) in [(1.0f32, 0.0f32), (0.0, 1.0)] {
        let b = pos.len() as u32;
        let (l, r) = (-pu * w, (1.0 - pu) * w);
        let (y0, y1) = (-pv * h, (1.0 - pv) * h);
        let corners =
            [([l * dx, y0, l * dz], [0.0, 1.0]), ([r * dx, y0, r * dz], [1.0, 1.0]), ([r * dx, y1, r * dz], [1.0, 0.0]), ([l * dx, y1, l * dz], [0.0, 0.0])];
        for (p, t) in corners {
            pos.push(p);
            nrm.push([dz, 0.0, -dx]);
            uv.push(t);
        }
        idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nrm)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(idx))
}

/// The tree files the game knows.
pub const TREES: [&str; 4] = ["trees/xa_cu.glb", "trees/sau.glb", "trees/bang.glb", "trees/lieu_ru.glb"];

/// The site name every street tree shares: it takes any delivered tree.
pub const TREE_SITE: &str = "trees/*";

/// Once the manifest is read: landmarks and trees replace their stand-ins.
#[allow(clippy::too_many_arguments)]
fn place_models(
    mut commands: Commands,
    hq: Res<Hq>,
    sites: Option<Res<Sites>>,
    assets: Res<AssetServer>,
    mut applied: ResMut<Applied>,
    mut stand_ins: Query<(Entity, &StandIn, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    if hq.state != HqState::Ready {
        return;
    }
    let Some(sites) = sites else { return };
    let mut hidden: HashSet<&'static str> = HashSet::new();
    let mut billboards: HashMap<&'static str, (Handle<Mesh>, Handle<StandardMaterial>)> = HashMap::new();
    let trees: Vec<&'static str> = TREES.iter().copied().filter(|t| hq.get(t).is_some()).collect();
    let mut levels_cache: HashMap<&'static str, Vec<String>> = HashMap::new();
    for s in &sites.0 {
        if applied.placed.contains(s.file) {
            continue;
        }
        // Trees take whichever kinds were delivered, mixed by position.
        let file = if s.file == TREE_SITE {
            let k = (s.at.x.abs() as usize).wrapping_mul(31).wrapping_add(s.at.z.abs() as usize);
            match trees.get(k % trees.len().max(1)) {
                Some(t) => *t,
                None => continue,
            }
        } else {
            s.file
        };
        let Some(entry) = hq.get(file) else { continue };
        let levels = levels_cache.entry(file).or_insert_with(|| levels_of(entry)).clone();
        let mut lod = LodModel::new(levels.clone(), lod_distances(s.kind, levels.len()));
        if s.kind == "tree" {
            let plane = if entry.billboard_plane_size_m.len() == 2 { &entry.billboard_plane_size_m } else { &entry.size_m };
            let (w, h) = (plane.first().copied().unwrap_or(9.0).clamp(0.5, 80.0), plane.get(1).copied().unwrap_or(13.0).clamp(0.5, 80.0));
            let (pu, pv) = match entry.billboard_pivot_uv.as_slice() {
                [u, v] if (0.0..=1.0).contains(u) && (0.0..=1.0).contains(v) => (*u, *v),
                _ => (0.5, 0.0),
            };
            let card = billboards
                .entry(file)
                .or_insert_with(|| {
                    let png = file.trim_end_matches(".glb").to_string() + "_billboard.png";
                    let mat = mats.add(StandardMaterial {
                        base_color_texture: Some(assets.load(hq.locate(&png))),
                        alpha_mode: AlphaMode::Mask(0.5),
                        double_sided: true,
                        cull_mode: None,
                        perceptual_roughness: 0.9,
                        ..default()
                    });
                    (meshes.add(billboard_mesh(w, h, pu, pv)), mat)
                })
                .clone();
            lod.billboard = Some(card);
            lod.billboard_upto = 600.0;
        }
        commands.spawn((Transform::from_translation(s.at).with_rotation(Quat::from_rotation_y(s.yaw)), Visibility::default(), lod));
        hidden.insert(s.file);
    }
    for f in &hidden {
        applied.placed.insert(f);
    }
    for (e, si, mut v) in &mut stand_ins {
        if (hidden.contains(si.0) || applied.placed.contains(si.0)) && *v != Visibility::Hidden {
            *v = Visibility::Hidden;
            commands.entity(e).remove::<crate::world::Far>();
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Ground materials.

/// Delivered material folders and the stand-in each replaces, with how many metres one repeat of
/// the stand-in's texture covers in the city's meshes.
const MATERIALS: [(&str, &str, f32); 7] = [
    ("via_he_da_xam", "paving", 4.0),
    ("duong_nhua", "asphalt", 4.0),
    ("co", "grass", 5.0),
    ("be_tong_mai", "roof", 4.0),
    ("ngoi_do", "roof_tiles", 4.0),
    ("nuoc_ho", "water", 5.0),
    ("gach_lat_ho", "path_paving", 4.0),
];

fn tiling(assets: &AssetServer, path: String, srgb: bool) -> Handle<Image> {
    assets
        .load_builder()
        .with_settings(move |s: &mut ImageLoaderSettings| {
            s.is_srgb = srgb;
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..ImageSamplerDescriptor::linear()
            });
        })
        .load(path)
}

/// How many metres a delivered material's texture covers (its info.json), defaulting to 2 m.
fn metres(doc: Option<&JsonDoc>) -> f32 {
    let v = doc.map(|d| &d.0);
    ["metres", "meters", "m", "size_m"]
        .iter()
        .find_map(|k| v.and_then(|v| v.get(*k)).and_then(|x| x.as_f64().or_else(|| x.get(0).and_then(|y| y.as_f64()))))
        .map(|m| m as f32)
        .filter(|m| *m > 0.05 && *m < 100.0)
        .unwrap_or(2.0)
}

/// A delivered material being loaded: its info.json and whichever textures were listed.
struct PendingMaterial {
    index: usize,
    info: Handle<JsonDoc>,
    base: Option<Handle<Image>>,
    normal: Option<Handle<Image>>,
    orm: Option<Handle<Image>>,
}

fn settled(assets: &AssetServer, id: impl Into<bevy::asset::UntypedAssetId>) -> bool {
    !matches!(assets.load_state(id), LoadState::Loading | LoadState::NotLoaded)
}

/// Swaps the ground materials for delivered ones (base colour, normal map, ORM). Each texture is
/// used only once it has loaded, so a missing file leaves that part of the stand-in as it was.
#[allow(clippy::too_many_arguments)]
fn apply_materials(
    hq: Res<Hq>,
    m: Option<Res<Mats>>,
    assets: Res<AssetServer>,
    docs: Res<Assets<JsonDoc>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut applied: ResMut<Applied>,
    mut pending: Local<Option<Vec<PendingMaterial>>>,
) {
    if hq.state != HqState::Ready || applied.materials {
        return;
    }
    let Some(m) = m else { return };
    let pending = pending.get_or_insert_with(|| {
        MATERIALS
            .iter()
            .enumerate()
            .filter_map(|(index, (folder, _, _))| {
                // A folder can be listed whole ("materials/co") or file by file.
                let whole = hq.get(&format!("materials/{folder}")).is_some();
                let has = |f: &str| whole || hq.get(&format!("materials/{folder}/{f}")).is_some();
                let load = |f: &str, srgb: bool| has(f).then(|| tiling(&assets, hq.locate(&format!("materials/{folder}/{f}")), srgb));
                let (base, normal, orm) = (load("basecolor.png", true), load("normal.png", false), load("orm.png", false));
                (base.is_some() || normal.is_some() || orm.is_some()).then(|| PendingMaterial {
                    index,
                    info: assets.load(Hq::path(&format!("materials/{folder}/info.json"))),
                    base,
                    normal,
                    orm,
                })
            })
            .collect()
    });
    let all_settled =
        pending.iter().all(|p| settled(&assets, &p.info) && [&p.base, &p.normal, &p.orm].iter().all(|h| h.as_ref().is_none_or(|h| settled(&assets, h))));
    if !all_settled {
        return;
    }
    let loaded = |h: &Option<Handle<Image>>| h.clone().filter(|h| matches!(assets.load_state(h), LoadState::Loaded));
    for p in pending.iter() {
        let Some((_, target, stand_in_m)) = MATERIALS.get(p.index) else { continue };
        let Some(handle) = m.by_name(target) else { continue };
        let Some(mut mat) = mats.get_mut(&handle) else { continue };
        if let Some(base) = loaded(&p.base) {
            mat.base_color = Color::WHITE;
            mat.base_color_texture = Some(base);
        }
        if let Some(normal) = loaded(&p.normal) {
            mat.normal_map_texture = Some(normal);
        }
        if let Some(orm) = loaded(&p.orm) {
            mat.metallic_roughness_texture = Some(orm.clone());
            mat.occlusion_texture = Some(orm);
            mat.perceptual_roughness = 1.0;
            mat.metallic = 1.0;
        }
        mat.uv_transform = Affine2::from_scale(Vec2::splat(stand_in_m / metres(docs.get(&p.info))));
    }
    applied.materials = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timed_lights_follow_sky_and_leave_other_materials_alone() {
        let glow = LinearRgba::new(1.0, 0.5, 0.2, 1.0);
        for (path, name) in [
            ("hq/street/den_duong_01.glb", "lamp_glass"),
            ("hq/street/den_duong_01_lod1.glb", "lamp_glass"),
            ("hq/buildings/mat_tien_tang_01.glb", "window_glow"),
            ("hq/buildings/mat_tien_tang_04_lod1.glb", "window_glow"),
            ("hq/buildings/mat_tien_tret_01.glb", "shop_light"),
            ("hq/buildings/mat_tien_tret_03_lod1.glb", "shop_light"),
        ] {
            assert_eq!(timed_emission_value(path, name, glow, true), Some(glow));
            assert_eq!(timed_emission_value(path, name, glow, false), Some(LinearRgba::BLACK));
        }
        assert_eq!(timed_emission_value("hq/people/den_pin.glb", "lamp_glass", glow, false), None);
        assert_eq!(timed_emission_value("hq/buildings/mat_tien_tang_01.glb", "other", glow, false), None);
        assert_eq!(timed_emission_value("hq/street/den_long_01.glb", "lamp_glass", glow, false), None);
    }

    #[test]
    fn timed_light_updates_when_sky_changes_and_when_lod_arrives_late() {
        let glow = LinearRgba::new(1.0, 0.5, 0.2, 1.0);
        let mut app = App::new();
        app.init_resource::<Assets<Gltf>>()
            .init_resource::<Assets<GltfMaterial>>()
            .init_resource::<Assets<StandardMaterial>>()
            .init_resource::<Flat>()
            .init_resource::<crate::night::Sky>()
            .add_systems(Update, timed_emission);
        let source = app.world_mut().resource_mut::<Assets<GltfMaterial>>().add(GltfMaterial { emissive: glow, ..default() });
        let display = app.world_mut().resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial { emissive: glow, ..default() });
        let gltf = Gltf {
            scenes: vec![], named_scenes: default(), meshes: vec![], named_meshes: default(),
            materials: vec![source.clone()], named_materials: [("window_glow".into(), source.clone())].into_iter().collect(),
            nodes: vec![], named_nodes: default(), skins: vec![], named_skins: default(),
            default_scene: None, animations: vec![], named_animations: default(), source: None,
        };
        let asset = app.world_mut().resource_mut::<Assets<Gltf>>().add(gltf);
        // Material and model can arrive after the first update in daytime.
        app.update();
        {
            let mut flat = app.world_mut().resource_mut::<Flat>();
            flat.files.insert("hq/buildings/mat_tien_tang_01_lod1.glb".into(), asset);
            flat.materials.insert(source.id(), display.clone());
        }
        app.update();
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().get(&display).unwrap().emissive, LinearRgba::BLACK);
        app.world_mut().resource_mut::<crate::night::Sky>().night = true;
        app.update();
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().get(&display).unwrap().emissive, glow);
        app.world_mut().resource_mut::<crate::night::Sky>().night = false;
        app.update();
        assert_eq!(app.world().resource::<Assets<StandardMaterial>>().get(&display).unwrap().emissive, LinearRgba::BLACK);
    }

    #[test]
    fn manifest_reads_the_documented_shape() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"version":1,"assets":[{"file":"people/minh.glb","lods":["people/minh_lod1.glb"],"kind":"character","size_m":[0.55,1.74,0.32],"skeleton":"armature-65","animations":[],"license":"x","made_with":"y"}]}"#,
        )
        .unwrap();
        let m = Manifest::parse(&v);
        assert_eq!(m.assets.len(), 1);
        assert_eq!(m.assets[0].lods, vec!["people/minh_lod1.glb".to_string()]);
        assert_eq!(m.assets[0].skeleton, "armature-65");
    }

    #[test]
    fn bad_manifests_mean_nothing_delivered() {
        for s in [r#"{"version":2,"assets":[{"file":"a.glb"}]}"#, r#"{"assets":"nope"}"#, r#"[1,2]"#, r#"{"version":1,"assets":[{"lods":[]}]}"#] {
            let v: serde_json::Value = serde_json::from_str(s).unwrap();
            assert!(Manifest::parse(&v).assets.is_empty(), "{s}");
        }
    }

    #[test]
    fn levels_by_distance() {
        let mut lod = LodModel::new(vec!["a.glb".into(), "a_lod1.glb".into()], vec![35.0, 140.0]);
        assert_eq!(lod.pick(10.0), Some(0));
        assert_eq!(lod.pick(100.0), Some(1));
        assert_eq!(lod.pick(300.0), None);
        lod.billboard = Some((Handle::default(), Handle::default()));
        lod.billboard_upto = 600.0;
        assert_eq!(lod.pick(300.0), Some(2));
        assert_eq!(lod.pick(700.0), None);
    }

    #[test]
    fn billboard_stands_on_its_pivot() {
        // The delivered xà cừ card: 17 × 34 m, its trunk's foot 29.4 % up from the bottom.
        let m = billboard_mesh(17.0, 34.0, 0.5, 0.294);
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)) = m.attribute(Mesh::ATTRIBUTE_POSITION) else { panic!("positions") };
        let (lo, hi) = pos.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
        assert!((lo + 9.996).abs() < 1e-3 && (hi - 24.004).abs() < 1e-3, "{lo} {hi}");
        let xs: Vec<f32> = pos.iter().map(|p| p[0]).collect();
        assert!(xs.iter().any(|x| (x + 8.5).abs() < 1e-4) && xs.iter().any(|x| (x - 8.5).abs() < 1e-4));
    }

    #[test]
    fn material_scale_comes_from_info() {
        let doc = |s: &str| JsonDoc(serde_json::from_str(s).unwrap());
        assert_eq!(metres(Some(&doc(r#"{"metres": 3.0}"#))), 3.0);
        assert_eq!(metres(Some(&doc(r#"{"size_m": [1.5, 1.5]}"#))), 1.5);
        assert_eq!(metres(Some(&doc(r#"{"metres": -1}"#))), 2.0);
        assert_eq!(metres(None), 2.0);
    }
}

#[cfg(test)]
mod joint_tests {
    use super::*;

    #[test]
    fn reads_the_artists_joint() {
        // As the cage delivered in batch 05 carries it.
        let x =
            r#"{"s3d_joint": {"name": "door_hinge", "type": "revolute", "value": 0.0, "axis": [0.0, -1.0, 0.0], "lower": 0.0, "upper": 1.9198621771937625}}"#;
        let j = Joint::from_extras(x, Quat::IDENTITY).expect("a joint");
        assert_eq!(j.axis, Vec3::NEG_Y);
        assert_eq!(j.lower, 0.0);
        assert!((j.upper.to_degrees() - 110.0).abs() < 0.01);
        // Fully open, the door's +X edge (its free side, as the hinge is at its −X end) has swung
        // 110° about −Y: from +X round towards +Z, the cage's front.
        let edge = j.at(1.0) * Vec3::X;
        assert!((edge.angle_between(Vec3::X).to_degrees() - 110.0).abs() < 0.01);
        assert!(edge.z > 0.9, "{edge:?}");
        assert_eq!(j.at(0.0), Quat::IDENTITY);
        // Anything else is not a hinge.
        assert!(Joint::from_extras(r#"{"s3d_joint": {"type": "prismatic", "axis": [0, 1, 0], "lower": 0, "upper": 1}}"#, Quat::IDENTITY).is_none());
        assert!(Joint::from_extras(r#"{"s3d_joint": {"type": "revolute", "axis": [0, 0, 0], "lower": 0, "upper": 1}}"#, Quat::IDENTITY).is_none());
        assert!(Joint::from_extras(r#"{"note": 1}"#, Quat::IDENTITY).is_none());
        assert!(Joint::from_extras("not json", Quat::IDENTITY).is_none());
    }
}

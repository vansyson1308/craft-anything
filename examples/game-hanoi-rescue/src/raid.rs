//! One trip into the district: the player, the thieves and how they see and hunt, the caged dogs,
//! traffic, the train, and getting out.

use std::f32::consts::PI;

use crate::content::{self, BREEDS, CHAPTERS, DOG_NAMES, Difficulty, LOOT_TABLE, LootKind};
use crate::geom::{Rng, V2, angle_diff, v2};
use crate::map::{ExitId, Map, Tile};

pub const PLAYER_R: f32 = 0.3;
const THIEF_R: f32 = 0.32;
const DOG_R: f32 = 0.22;
pub const EXTRACT_S: f32 = 3.0;
pub const CAGE_S: f32 = 1.5;
pub const TIE_S: f32 = 0.6;
pub const TRAIN_LEN: f32 = 13.0;

#[derive(Clone, Copy, Debug)]
pub struct PlayerStats {
    pub max_hp: f32,
    pub speed: f32,
    pub bag: usize,
    pub leash: usize,
    pub slippers: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mission {
    Story(usize),
    Free(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThiefKind {
    Lookout,
    Grunt,
    TuLong,
    Bay,
}

impl ThiefKind {
    /// (patrol speed, chase speed, stun points, hit damage, sight multiplier)
    fn stats(self) -> (f32, f32, f32, f32, f32) {
        match self {
            ThiefKind::Lookout => (2.0, 3.6, 60.0, 10.0, 1.0),
            ThiefKind::Grunt => (2.2, 3.9, 100.0, 15.0, 1.0),
            ThiefKind::TuLong => (2.0, 3.8, 260.0, 20.0, 1.05),
            ThiefKind::Bay => (2.3, 4.1, 380.0, 22.0, 1.1),
        }
    }
    pub fn is_boss(self) -> bool {
        matches!(self, ThiefKind::TuLong | ThiefKind::Bay)
    }
    pub fn name(self) -> &'static str {
        match self {
            ThiefKind::Lookout => "Thằng Còi",
            ThiefKind::Grunt => "Đàn em",
            ThiefKind::TuLong => "Tư Lồng",
            ThiefKind::Bay => "Bảy Thòng Lọng",
        }
    }
}

pub struct RaidSetup {
    pub mission: Mission,
    pub seed: u64,
    pub diff: Difficulty,
    pub exits: Vec<ExitId>,
    pub cages: u32,
    pub need_dogs: u32,
    pub clue: bool,
    pub boss: Option<ThiefKind>,
    pub puppy: bool,
    /// Where the story happens: x0, y0, x1, y1.
    pub zone: [i32; 4],
    pub spawn: V2,
    pub stats: PlayerStats,
}

impl RaidSetup {
    pub fn story(map: &Map, ch: usize, stats: PlayerStats, seed: u64) -> RaidSetup {
        let diff = content::CURVE.get(ch).or(content::CURVE.last()).copied().unwrap_or(content::free_difficulty(1));
        let sc = &map.scenario;
        let zone = sc.zones.get(ch).or(sc.zones.last()).copied().unwrap_or(sc.play);
        let spawn = sc.spawns.get(ch).or(sc.spawns.first()).copied().unwrap_or(v2(map.w as f32 / 2.0, map.h as f32 / 2.0));
        let base = RaidSetup {
            mission: Mission::Story(ch),
            seed,
            diff,
            exits: content::chapter_exits(ch).to_vec(),
            cages: 1,
            need_dogs: 1,
            clue: false,
            boss: None,
            puppy: false,
            zone,
            spawn,
            stats,
        };
        match ch {
            0 => base,
            1 => RaidSetup { clue: true, ..base },
            2 => RaidSetup { cages: 2, need_dogs: 2, ..base },
            3 => RaidSetup { cages: 2, need_dogs: 0, boss: Some(ThiefKind::TuLong), ..base },
            _ => RaidSetup { cages: 2, need_dogs: 0, boss: Some(ThiefKind::Bay), puppy: true, ..base },
        }
    }

    pub fn free(map: &Map, level: u32, stats: PlayerStats, seed: u64) -> RaidSetup {
        let mut rng = Rng::new(seed ^ 0xF4EE);
        let all: Vec<ExitId> = map.exits.iter().map(|e| e.id).collect();
        let first = rng.range(0, all.len());
        let second = (first + 1 + rng.range(0, all.len().saturating_sub(1))) % all.len().max(1);
        let mut exits: Vec<ExitId> = [first, second].iter().filter_map(|i| all.get(*i).copied()).collect();
        exits.dedup();
        let spawn = rng.pick(&map.scenario.free_spawns).copied().unwrap_or(v2(map.w as f32 / 2.0, map.h as f32 / 2.0));
        RaidSetup {
            mission: Mission::Free(level),
            seed,
            diff: content::free_difficulty(level),
            exits,
            cages: (1 + level / 2).min(4),
            need_dogs: 0,
            clue: false,
            boss: None,
            puppy: false,
            zone: map.scenario.play,
            spawn,
            stats,
        }
    }

    pub fn title(&self) -> String {
        match self.mission {
            Mission::Story(ch) => CHAPTERS.get(ch).map(|c| format!("Chương {}: {}", ch + 1, c.title)).unwrap_or_default(),
            Mission::Free(l) => format!("{} · cấp {l}", content::FREE_RAID_TITLE),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TState {
    Patrol,
    Wait(f32),
    Investigate,
    Chase,
    Search(f32),
    /// Getting ready to hit (or, for a boss with `charging`, to dash).
    Windup(f32),
    Charge(f32),
    Stunned(f32),
    Down(f32),
    Tied,
}

pub struct Thief {
    pub pos: V2,
    pub facing: f32,
    pub kind: ThiefKind,
    pub state: TState,
    pub alert: f32,
    pub stun_hp: f32,
    pub max_stun: f32,
    patrol: Vec<V2>,
    patrol_i: usize,
    path: Vec<V2>,
    path_goal: (i32, i32),
    repath_t: f32,
    pub target: V2,
    cd: f32,
    charge_cd: f32,
    pub charging: bool,
    pub charge_dir: V2,
    seen_t: f32,
    pub flash: f32,
    look_t: f32,
}

impl Thief {
    fn new(kind: ThiefKind, pos: V2, patrol: Vec<V2>) -> Thief {
        let (_, _, stun, _, _) = kind.stats();
        Thief {
            pos,
            facing: 0.0,
            kind,
            state: TState::Patrol,
            alert: 0.0,
            stun_hp: stun,
            max_stun: stun,
            patrol,
            patrol_i: 0,
            path: Vec::new(),
            path_goal: (-1, -1),
            repath_t: 0.0,
            target: pos,
            cd: 0.0,
            charge_cd: 2.0,
            charging: false,
            charge_dir: V2::default(),
            seen_t: 99.0,
            flash: 0.0,
            look_t: 0.0,
        }
    }
    /// Out of the fight for now (stunned, down or tied).
    pub fn out(&self) -> bool {
        matches!(self.state, TState::Stunned(_) | TState::Down(_) | TState::Tied)
    }
    pub fn unaware(&self) -> bool {
        matches!(self.state, TState::Patrol | TState::Wait(_) | TState::Investigate | TState::Search(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DogState {
    Caged,
    Following,
}

pub struct Dog {
    pub pos: V2,
    pub breed: usize,
    pub name: String,
    pub puppy: bool,
    pub state: DogState,
    pub wag: f32,
    pub facing: f32,
}

pub struct Cage {
    pub pos: V2,
    pub dog: usize,
    pub progress: f32,
    pub open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PickupKind {
    Loot(LootKind),
    Clue,
    Slipper,
}

pub struct Pickup {
    pub pos: V2,
    pub kind: PickupKind,
    pub taken: bool,
}

pub struct Vehicle {
    pub route: usize,
    pub s: f32,
    pub speed: f32,
    pub color: u32,
    pub pos: V2,
    pub dir: V2,
}

pub struct Train {
    /// How far along the railway the front of the train is.
    pub s: f32,
    pub active: bool,
    pub next_t: f32,
}

impl Train {
    pub fn warning(&self) -> bool {
        !self.active && self.next_t <= 5.0
    }
}

pub struct Slipper {
    pub pos: V2,
    pub vel: V2,
    left: f32,
}

pub struct Noise {
    pub pos: V2,
    pub r: f32,
    pub t: f32,
}

pub struct Fx {
    pub pos: V2,
    pub text: String,
    pub t: f32,
    pub color: u32,
}

pub struct Player {
    pub pos: V2,
    pub facing: f32,
    pub hp: f32,
    pub stamina: f32,
    pub roll_t: f32,
    roll_dir: V2,
    attack_cd: f32,
    pub swing_t: f32,
    pub hurt_t: f32,
    pub slippers: u32,
    pub sneaking: bool,
    pub moving: bool,
    step_t: f32,
    pub extract_t: f32,
    knock: V2,
    pub walk_t: f32,
}

impl Player {
    pub fn invulnerable(&self) -> bool {
        self.roll_t > 0.0 || self.hurt_t > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Escaped(ExitId),
    Caught,
    TimeUp,
    Abandoned,
}

#[derive(Clone, Debug, Default)]
pub struct Controls {
    pub mv: V2,
    pub sneak: bool,
    pub attack: bool,
    pub throw: bool,
    pub roll: bool,
    pub interact: bool,
}

/// What the context button does right now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Context {
    None,
    Cage { index: usize, progress: f32 },
    Tie { index: usize, progress: f32 },
    LeashFull,
}

pub struct Report {
    pub outcome: Outcome,
    pub objective_met: bool,
    pub cash: u32,
    pub scrap: u32,
    /// Rescued dogs: breed, name, whether it's Bơ.
    pub dogs: Vec<(usize, String, bool)>,
    pub tied: u32,
    pub seconds: f32,
}

pub struct Raid {
    pub setup: RaidSetup,
    pub player: Player,
    pub thieves: Vec<Thief>,
    pub dogs: Vec<Dog>,
    pub cages: Vec<Cage>,
    pub pickups: Vec<Pickup>,
    pub vehicles: Vec<Vehicle>,
    pub train: Train,
    pub slippers: Vec<Slipper>,
    pub noises: Vec<Noise>,
    pub fx: Vec<Fx>,
    pub bag: Vec<LootKind>,
    pub clue: bool,
    pub boss_tied: bool,
    pub time_left: f32,
    pub elapsed: f32,
    pub outcome: Option<Outcome>,
    pub banner: Option<(String, f32)>,
    pub shake: f32,
    reinforced: bool,
    rng: Rng,
    pub context: Context,
    tie_progress: f32,
    pub tied: u32,
    pub spotted: bool,
    bag_msg_t: f32,
}

fn nearest_walkable(map: &Map, p: V2) -> V2 {
    let (x, y) = p.tile();
    for r in 0..12i32 {
        for dy in -r..=r {
            for dx in -r..=r {
                if dx.abs() != r && dy.abs() != r {
                    continue;
                }
                let t = map.tile(x + dx, y + dy);
                if t.walkable() && t != Tile::Rail {
                    return crate::map::center(x + dx, y + dy);
                }
            }
        }
    }
    p
}

impl Raid {
    pub fn new(map: &Map, setup: RaidSetup) -> Raid {
        let mut rng = Rng::new(setup.seed);
        let spawn = nearest_walkable(map, setup.spawn);
        let [zx0, zy0, zx1, zy1] = setup.zone;
        let play = map.scenario.play;
        let reach = map.distances(spawn.tile());
        let reachable = |p: V2| {
            let (x, y) = p.tile();
            reach.get((y * map.w + x) as usize).copied().unwrap_or(-1) >= 0
        };
        let not_rail = |t: Tile| t != Tile::Rail && t != Tile::Bridge && t != Tile::Truss;

        // Cages, far enough from the start and from each other.
        let mut cages: Vec<Cage> = Vec::new();
        let mut dogs: Vec<Dog> = Vec::new();
        let mut used_names: Vec<&str> = Vec::new();
        for i in 0..setup.cages {
            let fixed = (setup.puppy && i == 0).then_some(map.scenario.puppy_cage);
            let pos = fixed.or_else(|| {
                map.random_spot(&mut rng, zx0, zy0, zx1, zy1, |t, p| {
                    not_rail(t) && t != Tile::Island && p.dist(spawn) > 12.0 && reachable(p) && cages.iter().all(|c| c.pos.dist(p) > 6.0)
                })
            });
            let Some(pos) = pos else { continue };
            let puppy = setup.puppy && i == 0;
            let breed = if puppy { content::PUPPY_BREED } else { rng.range(0, BREEDS.len()) };
            let name = if puppy {
                content::PUPPY.to_string()
            } else {
                let free: Vec<&str> = DOG_NAMES.iter().copied().filter(|n| !used_names.contains(n)).collect();
                let n = rng.pick(&free).copied().unwrap_or("Cún");
                used_names.push(n);
                n.to_string()
            };
            dogs.push(Dog { pos, breed, name, puppy, state: DogState::Caged, wag: rng.range_f(0.0, 6.0), facing: 0.0 });
            cages.push(Cage { pos, dog: dogs.len() - 1, progress: 0.0, open: false });
        }

        // Thieves: a guard near each cage, the boss at the heart of the zone, the rest patrol.
        let mut thieves = Vec::new();
        let patrol_around = |rng: &mut Rng, home: V2| -> Vec<V2> {
            let (hx, hy) = home.tile();
            let mut pts = vec![home];
            for _ in 0..3 {
                if let Some(p) = map.random_spot(rng, hx - 8, hy - 8, hx + 9, hy + 9, |t, p| not_rail(t) && reachable(p)) {
                    pts.push(p);
                }
            }
            pts
        };
        let far_from_start = |p: V2| p.dist(spawn) > 14.0;
        for c in &cages {
            let home = map
                .random_spot(&mut rng, c.pos.x as i32 - 4, c.pos.y as i32 - 4, c.pos.x as i32 + 5, c.pos.y as i32 + 5, |t, p| {
                    not_rail(t) && reachable(p) && far_from_start(p)
                })
                .unwrap_or(c.pos);
            let kind = if thieves.is_empty() && matches!(setup.mission, Mission::Story(0)) { ThiefKind::Lookout } else { ThiefKind::Grunt };
            let patrol = patrol_around(&mut rng, home);
            thieves.push(Thief::new(kind, home, patrol));
        }
        if let Some(boss) = setup.boss {
            let mid = v2((zx0 + zx1) as f32 / 2.0, (zy0 + zy1) as f32 / 2.0);
            let home = if boss == ThiefKind::Bay {
                map.scenario.boss_home
            } else {
                map.random_spot(&mut rng, mid.x as i32 - 6, mid.y as i32 - 10, mid.x as i32 + 7, mid.y as i32 + 11, |t, p| {
                    not_rail(t) && reachable(p) && far_from_start(p)
                })
                .unwrap_or(mid)
            };
            let patrol = patrol_around(&mut rng, home);
            thieves.push(Thief::new(boss, home, patrol));
        }
        let total = setup.diff.thieves as usize + setup.boss.map_or(0, |_| 1);
        let mut guard = 0;
        while thieves.len() < total && guard < 200 {
            guard += 1;
            let in_zone = rng.chance(0.6);
            let spot = if in_zone {
                map.random_spot(&mut rng, zx0 - 4, zy0 - 4, zx1 + 4, zy1 + 4, |t, p| not_rail(t) && reachable(p) && far_from_start(p))
            } else {
                map.random_spot(&mut rng, play[0], play[1], play[2], play[3], |t, p| not_rail(t) && reachable(p) && far_from_start(p))
            };
            let Some(home) = spot else { continue };
            let kind = if rng.chance(0.3) { ThiefKind::Lookout } else { ThiefKind::Grunt };
            let patrol = patrol_around(&mut rng, home);
            thieves.push(Thief::new(kind, home, patrol));
        }
        for t in &mut thieves {
            t.facing = rng.range_f(-PI, PI);
        }

        // Crates: anywhere on the streets, more in the zone, always a few slippers and bread.
        let mut pickups = Vec::new();
        let weight: u32 = LOOT_TABLE.iter().map(|(_, w)| *w).sum();
        let n_crates = 16 + rng.range(0, 6);
        for i in 0..n_crates {
            let spot = if i % 2 == 0 {
                map.random_spot(&mut rng, zx0, zy0, zx1, zy1, |t, p| not_rail(t) && reachable(p))
            } else {
                map.random_spot(&mut rng, play[0], play[1], play[2], play[3], |t, p| not_rail(t) && reachable(p))
            };
            let Some(pos) = spot else { continue };
            let kind = match i {
                0 | 1 => LootKind::Slippers(2),
                2 => LootKind::Bread,
                _ => {
                    let mut roll = rng.range(0, weight.max(1) as usize) as u32;
                    LOOT_TABLE
                        .iter()
                        .find(|(_, w)| {
                            if roll < *w {
                                true
                            } else {
                                roll -= *w;
                                false
                            }
                        })
                        .map(|(k, _)| *k)
                        .unwrap_or(LootKind::Cash(5))
                }
            };
            pickups.push(Pickup { pos, kind: PickupKind::Loot(kind), taken: false });
        }
        if setup.clue
            && let Some(pos) = map.random_spot(&mut rng, zx0, zy0, zx1, zy1, |t, p| not_rail(t) && reachable(p) && p.dist(spawn) > 12.0)
        {
            pickups.push(Pickup { pos, kind: PickupKind::Clue, taken: false });
        }

        // Traffic.
        let mut vehicles = Vec::new();
        let colors = [0xd64541, 0x2e86de, 0xf5f6fa, 0x222f3e, 0xfeca57, 0x10ac84, 0x8e44ad];
        let lanes: Vec<(usize, usize, f32, f32)> =
            map.routes.iter().enumerate().map(|(i, r)| (i, ((r.len / 14.0) as usize).clamp(2, 8), 3.0f32, if i == 0 { 5.5f32 } else { 4.8 })).collect();
        for (ri, count, lo, hi) in lanes {
            let len = map.routes.get(ri).map_or(1.0, |r| r.len);
            for k in 0..count {
                let s = len * k as f32 / count as f32 + rng.range_f(0.0, 2.0);
                let (pos, dir) = map.routes.get(ri).map(|r| r.at(s)).unwrap_or_default();
                let color = rng.pick(&colors).copied().unwrap_or(0xffffff);
                vehicles.push(Vehicle { route: ri, s, speed: rng.range_f(lo, hi), color, pos, dir });
            }
        }

        let stats = setup.stats;
        let time = setup.diff.time_s;
        let train_first = setup.diff.train_every_s * 0.5;
        Raid {
            player: Player {
                pos: spawn,
                facing: -PI / 2.0,
                hp: stats.max_hp,
                stamina: 100.0,
                roll_t: 0.0,
                roll_dir: V2::default(),
                attack_cd: 0.0,
                swing_t: 0.0,
                hurt_t: 0.0,
                slippers: stats.slippers,
                sneaking: false,
                moving: false,
                step_t: 0.0,
                extract_t: 0.0,
                knock: V2::default(),
                walk_t: 0.0,
            },
            setup,
            thieves,
            dogs,
            cages,
            pickups,
            vehicles,
            train: Train { s: -TRAIN_LEN, active: false, next_t: train_first },
            slippers: Vec::new(),
            noises: Vec::new(),
            fx: Vec::new(),
            bag: Vec::new(),
            clue: false,
            boss_tied: false,
            time_left: time,
            elapsed: 0.0,
            outcome: None,
            banner: None,
            shake: 0.0,
            reinforced: false,
            rng,
            context: Context::None,
            tie_progress: 0.0,
            tied: 0,
            spotted: false,
            bag_msg_t: 0.0,
        }
    }

    pub fn following(&self) -> usize {
        self.dogs.iter().filter(|d| d.state == DogState::Following).count()
    }

    pub fn exit_open(&self, id: ExitId) -> bool {
        self.setup.exits.contains(&id)
    }

    pub fn objective_met(&self) -> bool {
        let puppy_ok = !self.setup.puppy || self.dogs.iter().any(|d| d.puppy && d.state == DogState::Following);
        self.following() as u32 >= self.setup.need_dogs && (!self.setup.clue || self.clue) && (self.setup.boss.is_none() || self.boss_tied) && puppy_ok
    }

    /// The objective as a short line for the HUD.
    pub fn objective_text(&self) -> String {
        let mut parts = Vec::new();
        if self.setup.puppy {
            let ok = self.dogs.iter().any(|d| d.puppy && d.state == DogState::Following);
            parts.push(format!("{} Cứu {}", if ok { "✓" } else { "•" }, content::PUPPY));
        }
        if let Some(b) = self.setup.boss {
            parts.push(format!("{} Trói {}", if self.boss_tied { "✓" } else { "•" }, b.name()));
        }
        if self.setup.clue {
            parts.push(format!("{} Lấy cuốn sổ", if self.clue { "✓" } else { "•" }));
        }
        if self.setup.need_dogs > 0 {
            parts.push(format!("Cứu chó {}/{}", self.following(), self.setup.need_dogs));
        } else {
            parts.push(format!("Chó đã cứu: {}", self.following()));
        }
        if self.objective_met() {
            parts.push("→ Ra điểm thoát!".to_string());
        }
        parts.join(" · ")
    }

    pub fn report(&self) -> Report {
        let escaped = matches!(self.outcome, Some(Outcome::Escaped(_)));
        let (mut cash, mut scrap) = (0, 0);
        if escaped {
            for k in &self.bag {
                match k {
                    LootKind::Cash(v) => cash += v,
                    LootKind::Scrap(v) => scrap += v,
                    _ => {}
                }
            }
        }
        let dogs = if escaped {
            self.dogs.iter().filter(|d| d.state == DogState::Following).map(|d| (d.breed, d.name.clone(), d.puppy)).collect()
        } else {
            Vec::new()
        };
        Report {
            outcome: self.outcome.unwrap_or(Outcome::Abandoned),
            objective_met: escaped && self.objective_met(),
            cash,
            scrap,
            dogs,
            tied: self.tied,
            seconds: self.elapsed,
        }
    }

    fn say(&mut self, pos: V2, text: impl Into<String>, color: u32) {
        if self.fx.len() < 40 {
            self.fx.push(Fx { pos, text: text.into(), t: 1.2, color });
        }
    }

    fn noise(&mut self, pos: V2, r: f32) {
        if self.noises.len() < 32 {
            self.noises.push(Noise { pos, r, t: 0.12 });
        }
    }

    fn hurt_player(&mut self, dmg: f32, from: V2, push: f32) {
        if self.player.invulnerable() {
            return;
        }
        self.player.hp -= dmg;
        self.player.hurt_t = 0.7;
        self.player.knock = (self.player.pos - from).norm() * push;
        self.shake = self.shake.max(0.35);
        let at = self.player.pos;
        self.say(at, format!("-{}", dmg as i32), 0xff6b6b);
    }

    /// Advances the raid by `dt` seconds.
    pub fn update(&mut self, map: &Map, dt: f32, c: &Controls) {
        if self.outcome.is_some() {
            return;
        }
        let dt = dt.clamp(0.0, 0.05);
        self.elapsed += dt;
        self.time_left -= dt;
        self.shake = (self.shake - dt).max(0.0);
        self.bag_msg_t -= dt;
        if let Some((_, t)) = &mut self.banner {
            *t -= dt;
            if *t <= 0.0 {
                self.banner = None;
            }
        }
        if self.time_left <= 0.0 {
            self.outcome = Some(Outcome::TimeUp);
            return;
        }
        if self.setup.diff.reinforcements && !self.reinforced && self.time_left < self.setup.diff.time_s * 0.5 {
            self.reinforce(map);
        }

        self.update_player(map, dt, c);
        self.update_dogs(map, dt);
        self.update_thieves(map, dt);
        self.update_traffic(map, dt);
        self.update_train(map, dt);
        self.update_slippers(map, dt);

        self.noises.retain_mut(|n| {
            n.t -= dt;
            n.t > 0.0
        });
        self.fx.retain_mut(|f| {
            f.t -= dt;
            f.pos.y -= dt * 0.8;
            f.t > 0.0
        });
        self.spotted = self.thieves.iter().any(|t| matches!(t.state, TState::Chase | TState::Windup(_) | TState::Charge(_)));
        if self.player.hp <= 0.0 {
            self.player.hp = 0.0;
            self.outcome = Some(Outcome::Caught);
        }
    }

    fn reinforce(&mut self, map: &Map) {
        self.reinforced = true;
        let target = self.player.pos;
        for _ in 0..2 {
            let p = self.player.pos;
            // They arrive from somewhere out of sight, 18 to 40 tiles away.
            let edge = |t: Tile, q: V2| t != Tile::Rail && q.dist(p) > 18.0 && q.dist(p) < 40.0;
            let (px, py) = p.tile();
            if let Some(home) = map.random_spot(&mut self.rng, px - 40, py - 40, px + 41, py + 41, edge) {
                let mut t = Thief::new(ThiefKind::Grunt, home, vec![home]);
                t.state = TState::Investigate;
                t.target = target;
                t.alert = 0.4;
                self.thieves.push(t);
            }
        }
        self.banner = Some((content::ui::REINFORCE.to_string(), 3.0));
    }

    fn update_player(&mut self, map: &Map, dt: f32, c: &Controls) {
        let stats = self.setup.stats;
        let following = self.following();
        let p = &mut self.player;
        p.stamina = (p.stamina + 28.0 * dt).min(100.0);
        p.hurt_t -= dt;
        p.attack_cd -= dt;
        p.swing_t -= dt;
        let mv = c.mv.clamp_len(1.0);
        let mag = mv.len();
        p.moving = mag > 0.08;
        p.sneaking = p.moving && (c.sneak || mag < 0.55);
        if p.moving {
            p.facing = mv.angle();
        }
        let in_crowd = map.tile_at(p.pos) == Tile::Crowd;
        let mut speed = stats.speed * (1.0 - 0.07 * following as f32);
        if p.sneaking {
            speed *= 0.5;
        }
        if in_crowd {
            speed *= 0.82;
        }
        let mut step = if p.moving { mv.norm() * speed * dt } else { V2::default() };
        if p.roll_t > 0.0 {
            p.roll_t -= dt;
            step = p.roll_dir * 9.0 * dt;
        } else if c.roll && p.stamina >= 30.0 {
            p.stamina -= 30.0;
            p.roll_t = 0.28;
            p.roll_dir = if p.moving { mv.norm() } else { V2::from_angle(p.facing) };
        }
        step += p.knock * dt;
        p.knock = p.knock * (1.0 - (8.0 * dt).min(1.0));
        p.pos = map.slide(p.pos, step, PLAYER_R);
        if p.moving {
            p.walk_t += dt * if p.sneaking { 5.0 } else { 10.0 };
        }

        // Footsteps are heard; sneaking is silent.
        let mut noises: Vec<(V2, f32)> = Vec::new();
        if p.moving && !p.sneaking && p.roll_t <= 0.0 {
            p.step_t += dt;
            if p.step_t > 0.35 {
                p.step_t = 0.0;
                noises.push((p.pos, 3.2));
            }
        }

        let pos = self.player.pos;
        let facing = self.player.facing;

        // Attack: a swing in front, aimed at the nearest thief close by.
        if c.attack && self.player.attack_cd <= 0.0 {
            let aim = self
                .thieves
                .iter()
                .filter(|t| t.state != TState::Tied && t.pos.dist(pos) < 2.3)
                .min_by(|a, b| a.pos.dist(pos).total_cmp(&b.pos.dist(pos)))
                .map(|t| (t.pos - pos).angle());
            let dir = aim.unwrap_or(facing);
            self.player.facing = dir;
            self.player.attack_cd = 0.45;
            self.player.swing_t = 0.18;
            noises.push((pos, 4.0));
            let mut hits = Vec::new();
            for (i, t) in self.thieves.iter().enumerate() {
                if matches!(t.state, TState::Tied | TState::Down(_)) {
                    continue;
                }
                let d = t.pos - pos;
                if d.len() < 1.55 && angle_diff(d.angle(), dir).abs() < 1.1 {
                    hits.push(i);
                }
            }
            for i in hits {
                self.hit_thief(map, i, 34.0, pos);
            }
        }

        // Throw a slipper, aimed at a thief in view if there is one.
        if c.throw && self.player.slippers > 0 {
            let aim = self
                .thieves
                .iter()
                .filter(|t| !matches!(t.state, TState::Tied | TState::Down(_)) && t.pos.dist(pos) < 7.5 && map.line_of_sight(pos, t.pos))
                .min_by(|a, b| {
                    let sa = angle_diff((a.pos - pos).angle(), facing).abs() + a.pos.dist(pos) * 0.1;
                    let sb = angle_diff((b.pos - pos).angle(), facing).abs() + b.pos.dist(pos) * 0.1;
                    sa.total_cmp(&sb)
                })
                .map(|t| (t.pos - pos).norm());
            let dir = aim.unwrap_or(V2::from_angle(facing));
            self.player.slippers -= 1;
            self.slippers.push(Slipper { pos: pos + dir * 0.4, vel: dir * 13.0, left: 8.0 });
        }

        // The context action: open a cage or tie up a thief.
        let leash_full = self.following() >= stats.leash;
        let tie = self
            .thieves
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                let d = t.pos.dist(pos);
                let behind = angle_diff((pos - t.pos).angle(), t.facing).abs() > 1.9;
                d < 1.35 && (matches!(t.state, TState::Down(_)) || (t.unaware() && behind && self.player.sneaking))
            })
            .min_by(|a, b| a.1.pos.dist(pos).total_cmp(&b.1.pos.dist(pos)))
            .map(|(i, _)| i);
        let cage = self
            .cages
            .iter()
            .enumerate()
            .filter(|(_, k)| !k.open && k.pos.dist(pos) < 1.35)
            .min_by(|a, b| a.1.pos.dist(pos).total_cmp(&b.1.pos.dist(pos)))
            .map(|(i, _)| i);
        self.context = if let Some(i) = tie {
            Context::Tie { index: i, progress: self.tie_progress / TIE_S }
        } else if let Some(i) = cage {
            if leash_full { Context::LeashFull } else { Context::Cage { index: i, progress: self.cages.get(i).map_or(0.0, |k| k.progress / CAGE_S) } }
        } else {
            Context::None
        };
        match self.context {
            Context::Tie { index, .. } if c.interact => {
                self.tie_progress += dt;
                if self.tie_progress >= TIE_S {
                    self.tie_progress = 0.0;
                    self.tie_thief(index);
                }
            }
            Context::Cage { index, .. } if c.interact => {
                let mut opened = None;
                if let Some(k) = self.cages.get_mut(index) {
                    let before = k.progress;
                    k.progress += dt;
                    if (before / 0.5).floor() != (k.progress / 0.5).floor() {
                        noises.push((k.pos, 2.6));
                    }
                    if k.progress >= CAGE_S {
                        k.open = true;
                        opened = Some(k.dog);
                    }
                }
                if let Some(d) = opened
                    && let Some(dog) = self.dogs.get_mut(d)
                {
                    dog.state = DogState::Following;
                    let (at, name) = (dog.pos, dog.name.clone());
                    self.say(at, format!("Cứu được {name}!"), 0xffd166);
                }
            }
            _ => self.tie_progress = 0.0,
        }

        // Pick up what's underfoot.
        let bag_cap = stats.bag;
        let mut said = Vec::new();
        for k in &mut self.pickups {
            if k.taken || k.pos.dist(self.player.pos) > 0.8 {
                continue;
            }
            match k.kind {
                PickupKind::Loot(LootKind::Cash(_) | LootKind::Scrap(_)) => {
                    if self.bag.len() >= bag_cap {
                        if self.bag_msg_t <= 0.0 {
                            said.push((k.pos, content::ui::BAG_FULL.to_string(), 0xffffff));
                            self.bag_msg_t = 2.0;
                        }
                        continue;
                    }
                    if let PickupKind::Loot(l) = k.kind {
                        self.bag.push(l);
                        said.push((k.pos, content::loot_name(l), 0x9be15d));
                    }
                }
                PickupKind::Loot(LootKind::Slippers(n)) => {
                    self.player.slippers = (self.player.slippers + n).min(20);
                    said.push((k.pos, content::loot_name(LootKind::Slippers(n)), 0x9be15d));
                }
                PickupKind::Loot(LootKind::Bread) => {
                    self.player.hp = (self.player.hp + 30.0).min(stats.max_hp);
                    said.push((k.pos, content::loot_name(LootKind::Bread), 0x9be15d));
                }
                PickupKind::Clue => {
                    self.clue = true;
                    said.push((k.pos, "Lấy được cuốn sổ!".to_string(), 0xffd166));
                }
                PickupKind::Slipper => {
                    self.player.slippers = (self.player.slippers + 1).min(20);
                }
            }
            k.taken = true;
        }
        for (p, t, col) in said {
            self.say(p, t, col);
        }
        for (p, r) in noises {
            self.noise(p, r);
        }

        // Getting out: stand in an open exit.
        let here = self.player.pos;
        let exit = map.exits.iter().find(|e| self.setup.exits.contains(&e.id) && e.contains(here)).map(|e| e.id);
        match exit {
            Some(id) => {
                self.player.extract_t += dt;
                if self.player.extract_t >= EXTRACT_S {
                    self.outcome = Some(Outcome::Escaped(id));
                }
            }
            None => self.player.extract_t = 0.0,
        }
    }

    fn hit_thief(&mut self, map: &Map, i: usize, dmg: f32, from: V2) {
        let mut said = None;
        if let Some(t) = self.thieves.get_mut(i) {
            let dmg = if t.unaware() { dmg * 1.6 } else { dmg };
            t.stun_hp -= dmg;
            t.flash = 0.15;
            t.alert = 1.0;
            t.charging = false;
            let push = (t.pos - from).norm() * 0.8;
            t.pos = map.slide(t.pos, push, THIEF_R);
            if t.stun_hp <= 0.0 {
                t.state = TState::Down(7.0);
                said = Some((t.pos, "Gục!".to_string()));
            } else {
                t.state = TState::Stunned(0.35);
            }
        }
        self.shake = self.shake.max(0.12);
        if let Some((p, s)) = said {
            self.say(p, s, 0xffffff);
        }
    }

    fn tie_thief(&mut self, i: usize) {
        let mut drop = None;
        if let Some(t) = self.thieves.get_mut(i) {
            t.state = TState::Tied;
            if t.kind.is_boss() {
                self.boss_tied = true;
            }
            drop = Some((t.pos, t.kind));
        }
        if let Some((pos, kind)) = drop {
            self.tied += 1;
            let text = if kind.is_boss() { format!("Đã trói {}!", kind.name()) } else { "Trói xong!".to_string() };
            self.say(pos, text, 0xffd166);
            if self.rng.chance(0.6) {
                self.pickups.push(Pickup { pos: pos + v2(0.4, 0.3), kind: PickupKind::Loot(LootKind::Cash(10)), taken: false });
            }
        }
    }

    fn update_dogs(&mut self, map: &Map, dt: f32) {
        let p = self.player.pos;
        let back = -V2::from_angle(self.player.facing);
        let speed = (self.setup.stats.speed * 1.3).max(4.6);
        let mut k = 0;
        for d in &mut self.dogs {
            d.wag += dt * if d.state == DogState::Following { 14.0 } else { 6.0 };
            if d.state != DogState::Following {
                continue;
            }
            k += 1;
            let target = p + back * (0.7 * k as f32);
            let to = target - d.pos;
            if to.len() > 6.0 {
                d.pos = p;
            } else if to.len() > 0.25 {
                let step = to.norm() * (speed * dt).min(to.len());
                d.pos = map.slide(d.pos, step, DOG_R);
                d.facing = to.angle();
            }
        }
    }

    /// Whether thief `t` sees the player, and how quickly that raises its alert.
    fn sight(&self, map: &Map, t: &Thief) -> Option<f32> {
        let p = &self.player;
        let (_, _, _, _, mult) = t.kind.stats();
        let range = self.setup.diff.vision * mult;
        let d = t.pos.dist(p.pos);
        if d > range {
            return None;
        }
        let half_fov = self.setup.diff.fov_deg.to_radians() / 2.0;
        let ang = angle_diff((p.pos - t.pos).angle(), t.facing).abs();
        let close = d < 1.3 && !p.sneaking;
        if ang > half_fov && !close {
            return None;
        }
        if !map.line_of_sight(t.pos, p.pos) {
            return None;
        }
        let mut rate = self.setup.diff.alert_rate * (1.25 - d / range).max(0.25) * 1.6;
        if p.sneaking {
            rate *= 0.55;
        }
        if map.tile_at(p.pos) == Tile::Crowd {
            rate *= 0.45;
        }
        Some(rate)
    }

    fn update_thieves(&mut self, map: &Map, dt: f32) {
        let ppos = self.player.pos;
        for i in 0..self.thieves.len() {
            let sight = match self.thieves.get(i) {
                Some(t) if !t.out() && !matches!(t.state, TState::Windup(_) | TState::Charge(_)) => self.sight(map, t),
                _ => None,
            };
            let heard = self.thieves.get(i).and_then(|t| self.noises.iter().find(|n| n.pos.dist(t.pos) <= n.r).map(|n| n.pos));
            let mut events: Vec<(V2, String)> = Vec::new();
            let mut strike: Option<(f32, V2, f32)> = None;
            let Some(t) = self.thieves.get_mut(i) else { continue };
            let (patrol_speed, chase_speed, _, damage, _) = t.kind.stats();
            t.cd -= dt;
            t.charge_cd -= dt;
            t.flash -= dt;
            match t.state {
                TState::Tied => continue,
                TState::Stunned(s) => {
                    t.state = if s - dt <= 0.0 { TState::Chase } else { TState::Stunned(s - dt) };
                    t.seen_t = 0.0;
                    t.target = ppos;
                    continue;
                }
                TState::Down(s) => {
                    if s - dt <= 0.0 {
                        t.stun_hp = t.max_stun * 0.5;
                        t.state = TState::Chase;
                        t.target = ppos;
                    } else {
                        t.state = TState::Down(s - dt);
                    }
                    continue;
                }
                TState::Windup(s) => {
                    t.facing = turn(t.facing, (ppos - t.pos).angle(), 10.0 * dt);
                    if s - dt > 0.0 {
                        t.state = TState::Windup(s - dt);
                    } else if t.charging {
                        t.charge_dir = V2::from_angle(t.facing);
                        t.state = TState::Charge(0.5);
                    } else {
                        if t.pos.dist(ppos) < 1.55 {
                            strike = Some((damage, t.pos, 6.0));
                        }
                        t.state = TState::Chase;
                        t.cd = 1.1;
                    }
                }
                TState::Charge(s) => {
                    let want = t.charge_dir * 9.5 * dt;
                    let before = t.pos;
                    t.pos = map.slide(t.pos, want, THIEF_R);
                    if t.pos.dist(before) < want.len() * 0.5 {
                        t.state = TState::Stunned(1.6);
                        t.charging = false;
                        t.charge_cd = 4.0;
                        events.push((t.pos, "Đâm tường!".to_string()));
                    } else if t.pos.dist(ppos) < 0.9 {
                        strike = Some((damage + 6.0, t.pos, 10.0));
                        t.state = TState::Chase;
                        t.charging = false;
                        t.charge_cd = 4.0;
                    } else if s - dt <= 0.0 {
                        t.state = TState::Chase;
                        t.charging = false;
                        t.charge_cd = 4.0;
                    } else {
                        t.state = TState::Charge(s - dt);
                    }
                }
                _ => {
                    // Perception.
                    match sight {
                        Some(rate) => {
                            t.seen_t = 0.0;
                            t.target = ppos;
                            t.alert = (t.alert + rate * dt).min(1.0);
                        }
                        None => {
                            t.seen_t += dt;
                            if t.state != TState::Chase {
                                t.alert = (t.alert - 0.2 * dt).max(0.0);
                            }
                        }
                    }
                    if let Some(at) = heard
                        && matches!(t.state, TState::Patrol | TState::Wait(_) | TState::Search(_))
                    {
                        t.state = TState::Investigate;
                        t.target = at;
                        t.alert = t.alert.max(0.3);
                        events.push((t.pos, "?".to_string()));
                    }
                    if t.alert >= 1.0 && t.state != TState::Chase {
                        t.state = TState::Chase;
                        events.push((t.pos, content::ui::SPOTTED.to_string()));
                    } else if t.alert >= 0.4 && matches!(t.state, TState::Patrol | TState::Wait(_)) {
                        t.state = TState::Investigate;
                    }

                    // Behaviour.
                    match t.state {
                        TState::Patrol => {
                            let goal = t.patrol.get(t.patrol_i % t.patrol.len().max(1)).copied().unwrap_or(t.pos);
                            if t.pos.dist(goal) < 0.4 {
                                t.patrol_i += 1;
                                t.state = TState::Wait(1.2 + (i as f32 * 0.37) % 1.0);
                            } else {
                                go(map, t, goal, patrol_speed, dt, 3.0);
                            }
                        }
                        TState::Wait(s) | TState::Search(s) => {
                            t.look_t += dt;
                            t.facing += (t.look_t * 1.7).sin() * 1.6 * dt;
                            let left = s - dt;
                            t.state = if left > 0.0 {
                                if matches!(t.state, TState::Wait(_)) { TState::Wait(left) } else { TState::Search(left) }
                            } else {
                                TState::Patrol
                            };
                        }
                        TState::Investigate => {
                            let goal = t.target;
                            if t.pos.dist(goal) < 0.6 {
                                t.state = TState::Search(2.5);
                            } else {
                                go(map, t, goal, patrol_speed * 1.25, dt, 1.0);
                            }
                        }
                        TState::Chase => {
                            let d = t.pos.dist(ppos);
                            if t.seen_t > 4.0 {
                                t.state = TState::Search(3.0);
                                t.alert = 0.6;
                            } else if sight.is_some() && d < 1.15 && t.cd <= 0.0 {
                                t.state = TState::Windup(0.5);
                                t.charging = false;
                            } else if sight.is_some() && t.kind.is_boss() && t.charge_cd <= 0.0 && d > 2.5 && d < 6.5 {
                                t.state = TState::Windup(0.7);
                                t.charging = true;
                            } else {
                                let goal = t.target;
                                if t.pos.dist(goal) < 0.5 && sight.is_none() {
                                    t.state = TState::Search(3.0);
                                } else {
                                    go(map, t, goal, chase_speed, dt, 0.5);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            for (p, s) in events {
                let col = if s == "?" { 0xffd166 } else { 0xff6b6b };
                self.say(p + v2(0.0, -0.9), s, col);
            }
            if let Some((dmg, from, push)) = strike {
                self.hurt_player(dmg, from, push);
            }
        }

        // Keep thieves from standing in each other.
        let n = self.thieves.len();
        for a in 0..n {
            for b in (a + 1)..n {
                let (pa, pb) = match (self.thieves.get(a), self.thieves.get(b)) {
                    (Some(x), Some(y)) if x.state != TState::Tied && y.state != TState::Tied => (x.pos, y.pos),
                    _ => continue,
                };
                let d = pb - pa;
                if d.len() < 0.6 && d.len() > 1e-4 {
                    let push = d.norm() * ((0.6 - d.len()) * 0.5);
                    if let Some(x) = self.thieves.get_mut(a) {
                        x.pos = map.slide(x.pos, -push, THIEF_R);
                    }
                    if let Some(y) = self.thieves.get_mut(b) {
                        y.pos = map.slide(y.pos, push, THIEF_R);
                    }
                }
            }
        }
    }

    fn update_traffic(&mut self, map: &Map, dt: f32) {
        let mut hits_player = None;
        let mut hits_thieves = Vec::new();
        for v in &mut self.vehicles {
            let Some(route) = map.routes.get(v.route) else { continue };
            v.s += v.speed * dt;
            if !route.closed && v.s > route.len {
                v.s -= route.len;
            }
            let (pos, dir) = route.at(v.s);
            v.pos = pos;
            v.dir = dir;
            if v.pos.dist(self.player.pos) < 0.7 {
                hits_player = Some(v.pos);
            }
            for (i, t) in self.thieves.iter().enumerate() {
                if !matches!(t.state, TState::Tied | TState::Down(_) | TState::Stunned(_)) && v.pos.dist(t.pos) < 0.7 {
                    hits_thieves.push((i, v.pos));
                }
            }
        }
        if let Some(from) = hits_player
            && !self.player.invulnerable()
        {
            self.hurt_player(12.0, from, 7.0);
            let at = self.player.pos;
            self.say(at + v2(0.0, -0.8), "Ối!", 0xffffff);
        }
        for (i, from) in hits_thieves {
            let mut said = None;
            if let Some(t) = self.thieves.get_mut(i) {
                t.state = TState::Stunned(2.5);
                t.stun_hp -= 30.0;
                if t.stun_hp <= 0.0 {
                    t.state = TState::Down(7.0);
                }
                t.pos = map.slide(t.pos, (t.pos - from).norm() * 1.2, THIEF_R);
                said = Some(t.pos);
            }
            if let Some(p) = said {
                self.say(p + v2(0.0, -0.8), "Á!", 0xffffff);
            }
        }
    }

    fn update_train(&mut self, map: &Map, dt: f32) {
        let Some(rail) = &map.rail else { return };
        let every = self.setup.diff.train_every_s;
        if !self.train.active {
            self.train.next_t -= dt;
            let near = rail.dist(self.player.pos) < 14.0;
            if self.train.warning() && near && self.banner.is_none() {
                self.banner = Some((content::ui::TRAIN.to_string(), 2.0));
            }
            if self.train.next_t <= 0.0 {
                self.train.active = true;
                self.train.s = -2.0;
            }
            return;
        }
        self.train.s += 15.0 * dt;
        if self.train.s > rail.len + TRAIN_LEN + 2.0 {
            self.train.active = false;
            self.train.next_t = every;
            return;
        }
        // The cars, one point per tile from the tail to the head.
        let cars: Vec<V2> = (0..=(TRAIN_LEN as i32)).map(|k| self.train.s - k as f32).filter(|d| *d >= 0.0 && *d <= rail.len).map(|d| rail.at(d).0).collect();
        let hit = |p: V2| cars.iter().copied().find(|c| c.dist(p) < 1.25);
        if let Some(c) = hit(self.player.pos)
            && !self.player.invulnerable()
        {
            self.hurt_player(40.0, c, 12.0);
        }
        let mut said = Vec::new();
        for t in &mut self.thieves {
            if t.state == TState::Tied || matches!(t.state, TState::Down(_)) {
                continue;
            }
            if let Some(c) = hit(t.pos) {
                t.state = TState::Down(8.0);
                t.stun_hp = 0.0;
                t.pos = map.slide(t.pos, (t.pos - c).norm() * 1.6, THIEF_R);
                said.push(t.pos);
            }
        }
        for p in said {
            self.say(p, "Á á á!", 0xffffff);
        }
    }

    fn update_slippers(&mut self, map: &Map, dt: f32) {
        let mut landed = Vec::new();
        let mut hit = Vec::new();
        for (si, s) in self.slippers.iter_mut().enumerate() {
            let step = s.vel * dt;
            s.pos += step;
            s.left -= step.len();
            if let Some(i) = self.thieves.iter().position(|t| !matches!(t.state, TState::Tied | TState::Down(_)) && t.pos.dist(s.pos) < 0.55) {
                hit.push((si, i));
            } else if s.left <= 0.0 || !map.walkable_at(s.pos) {
                landed.push(si);
            }
        }
        let mut done: Vec<usize> = Vec::new();
        for (si, i) in hit {
            let at = self.slippers.get(si).map(|s| s.pos).unwrap_or_default();
            let mut said = None;
            if let Some(t) = self.thieves.get_mut(i) {
                t.state = TState::Stunned(1.8);
                t.stun_hp -= 15.0;
                t.alert = t.alert.max(0.8);
                t.flash = 0.15;
                t.charging = false;
                said = Some(t.pos);
            }
            if let Some(p) = said {
                self.say(p + v2(0.0, -0.8), "Bốp!", 0xffd166);
            }
            self.pickups.push(Pickup { pos: at, kind: PickupKind::Slipper, taken: false });
            done.push(si);
        }
        for si in landed {
            let mut at = self.slippers.get(si).map(|s| s.pos).unwrap_or_default();
            if !map.walkable_at(at)
                && let Some(s) = self.slippers.get(si)
            {
                at = at - s.vel.norm() * 0.5;
            }
            self.noise(at, 5.5);
            if map.walkable_at(at) {
                self.pickups.push(Pickup { pos: at, kind: PickupKind::Slipper, taken: false });
            }
            done.push(si);
        }
        done.sort_unstable();
        done.dedup();
        for si in done.into_iter().rev() {
            if si < self.slippers.len() {
                self.slippers.remove(si);
            }
        }
    }
}

/// Turns angle `from` toward `to` by at most `max`.
fn turn(from: f32, to: f32, max: f32) -> f32 {
    let d = angle_diff(to, from);
    from + d.clamp(-max, max)
}

/// Moves a thief toward `goal` along a path, re-planning every `repath` seconds.
fn go(map: &Map, t: &mut Thief, goal: V2, speed: f32, dt: f32, repath: f32) {
    t.repath_t -= dt;
    let goal_tile = goal.tile();
    if t.path.is_empty() || t.path_goal != goal_tile || t.repath_t <= 0.0 {
        t.path = map.path(t.pos, goal, 2500).unwrap_or_else(|| vec![goal]);
        t.path_goal = goal_tile;
        t.repath_t = repath;
    }
    let Some(next) = t.path.first().copied() else { return };
    let to = next - t.pos;
    if to.len() < 0.3 && !t.path.is_empty() {
        t.path.remove(0);
    }
    let step = to.norm() * (speed * dt).min(to.len().max(0.0));
    t.pos = map.slide(t.pos, step, THIEF_R);
    if to.len() > 0.05 {
        t.facing = turn(t.facing, to.angle(), 7.0 * dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stats() -> PlayerStats {
        PlayerStats { max_hp: 100.0, speed: 4.0, bag: 6, leash: 2, slippers: 3 }
    }

    /// Walks the player along a path to `goal` with the controls a person would use.
    fn walk_to(r: &mut Raid, map: &Map, goal: V2, max_s: f32, sneak: bool) -> bool {
        let mut t = 0.0;
        while t < max_s {
            if r.outcome.is_some() {
                return false;
            }
            if r.player.pos.dist(goal) < 0.4 {
                return true;
            }
            let path = map.path(r.player.pos, goal, 20_000).unwrap_or_default();
            let next = path.first().copied().unwrap_or(goal);
            let mv = (next - r.player.pos).norm() * if sneak { 0.4 } else { 1.0 };
            for _ in 0..3 {
                r.update(map, 1.0 / 30.0, &Controls { mv, ..Default::default() });
                t += 1.0 / 30.0;
            }
        }
        false
    }

    #[test]
    fn every_chapter_builds_a_fair_raid() {
        let map = Map::hoan_kiem();
        for ch in 0..CHAPTERS.len() {
            for seed in 0..20u64 {
                let r = Raid::new(&map, RaidSetup::story(&map, ch, stats(), seed));
                assert_eq!(r.cages.len() as u32, r.setup.cages, "ch{ch} seed{seed}: cages");
                assert!(r.thieves.len() >= r.setup.diff.thieves as usize, "ch{ch} seed{seed}: thieves");
                for t in &r.thieves {
                    assert!(t.pos.dist(r.player.pos) > 8.0 || t.kind.is_boss(), "a thief starts on top of the player");
                }
                let reach = map.distances(r.player.pos.tile());
                for c in &r.cages {
                    let (x, y) = c.pos.tile();
                    assert!(reach.get((y * crate::map::W + x) as usize).copied().unwrap_or(-1) >= 0, "ch{ch} seed{seed}: unreachable cage");
                }
                assert!(r.setup.exits.iter().all(|id| map.exits.iter().any(|e| e.id == *id)));
                if r.setup.clue {
                    assert!(r.pickups.iter().any(|p| p.kind == PickupKind::Clue));
                }
            }
        }
    }

    /// The game's self-test: with the thieves asleep, chapter 1 can be won by walking to the cage,
    /// opening it and walking out.
    #[test]
    fn chapter_one_can_be_won() {
        let map = Map::hoan_kiem();
        let mut r = Raid::new(&map, RaidSetup::story(&map, 0, stats(), 4));
        r.thieves.clear();
        let cage = r.cages.first().map(|c| c.pos).unwrap_or_default();
        assert!(walk_to(&mut r, &map, cage, 120.0, false), "reach the cage");
        for _ in 0..60 {
            r.update(&map, 1.0 / 30.0, &Controls { interact: true, ..Default::default() });
        }
        assert_eq!(r.following(), 1, "{:?}", r.context);
        assert!(r.objective_met());
        let exit = map.exits.iter().find(|e| e.id == ExitId::QuanChuong).map(|e| e.center()).unwrap_or_default();
        assert!(walk_to(&mut r, &map, exit, 200.0, false), "reach the exit");
        for _ in 0..120 {
            r.update(&map, 1.0 / 30.0, &Controls::default());
        }
        assert_eq!(r.outcome, Some(Outcome::Escaped(ExitId::QuanChuong)));
        let rep = r.report();
        assert!(rep.objective_met && rep.dogs.len() == 1);
    }

    #[test]
    fn a_thief_sees_what_is_in_front_not_behind() {
        let map = Map::hoan_kiem();
        let mut r = Raid::new(&map, RaidSetup::story(&map, 1, stats(), 1));
        r.thieves.truncate(1);
        r.vehicles.clear();
        r.player.pos = v2(30.0, 3.0);
        let Some(t) = r.thieves.first_mut() else { return };
        t.pos = v2(26.0, 3.0);
        t.patrol = vec![t.pos];
        t.facing = 0.0; // looking east, at the player
        t.state = TState::Wait(100.0);
        let seen = r.thieves.first().and_then(|t| r.sight(&map, t));
        assert!(seen.is_some());
        if let Some(t) = r.thieves.first_mut() {
            t.facing = PI; // looking west, away
        }
        r.player.sneaking = true;
        let seen = r.thieves.first().and_then(|t| r.sight(&map, t));
        assert!(seen.is_none());
    }

    #[test]
    fn standing_in_view_gets_you_chased_and_hit() {
        let map = Map::hoan_kiem();
        let mut r = Raid::new(&map, RaidSetup::story(&map, 2, stats(), 2));
        r.thieves.truncate(1);
        r.vehicles.clear();
        r.player.pos = v2(30.0, 3.0);
        if let Some(t) = r.thieves.first_mut() {
            t.pos = v2(25.0, 3.0);
            t.facing = 0.0;
            t.state = TState::Wait(100.0);
        }
        let hp = r.player.hp;
        for _ in 0..300 {
            r.update(&map, 1.0 / 30.0, &Controls::default());
        }
        assert!(r.player.hp < hp, "the thief should have reached and hit the player");
    }

    #[test]
    fn three_hits_knock_a_thief_down_and_he_can_be_tied() {
        let map = Map::hoan_kiem();
        let mut r = Raid::new(&map, RaidSetup::story(&map, 1, stats(), 3));
        r.thieves.truncate(1);
        r.vehicles.clear();
        r.player.pos = v2(30.0, 3.0);
        if let Some(t) = r.thieves.first_mut() {
            t.pos = v2(31.0, 3.0);
            t.state = TState::Chase;
        }
        for _ in 0..3 {
            r.update(&map, 0.02, &Controls { attack: true, ..Default::default() });
            for _ in 0..25 {
                r.update(&map, 0.02, &Controls::default());
            }
            if let Some(t) = r.thieves.first_mut()
                && !matches!(t.state, TState::Down(_))
            {
                t.pos = v2(31.0, 3.0);
            }
            r.player.pos = v2(30.0, 3.0);
        }
        assert!(matches!(r.thieves.first().map(|t| t.state), Some(TState::Down(_))));
        if let Some(t) = r.thieves.first() {
            r.player.pos = t.pos - v2(0.8, 0.0);
        }
        for _ in 0..40 {
            r.update(&map, 0.02, &Controls { interact: true, ..Default::default() });
        }
        assert_eq!(r.tied, 1);
    }

    #[test]
    fn a_thrown_slipper_lands_makes_noise_and_can_be_picked_up() {
        let map = Map::hoan_kiem();
        let mut r = Raid::new(&map, RaidSetup::story(&map, 0, stats(), 5));
        r.thieves.clear();
        r.vehicles.clear();
        r.player.pos = v2(30.0, 3.0);
        r.player.facing = 0.0;
        r.update(&map, 0.02, &Controls { throw: true, ..Default::default() });
        assert_eq!(r.player.slippers, 2);
        let mut heard = false;
        for _ in 0..100 {
            r.update(&map, 0.02, &Controls::default());
            heard |= !r.noises.is_empty();
        }
        assert!(heard);
        assert!(r.pickups.iter().any(|p| p.kind == PickupKind::Slipper && !p.taken));
    }

    #[test]
    fn running_out_of_time_or_health_ends_the_raid() {
        let map = Map::hoan_kiem();
        let mut r = Raid::new(&map, RaidSetup::free(&map, 3, stats(), 9));
        r.time_left = 0.01;
        r.update(&map, 0.02, &Controls::default());
        assert_eq!(r.outcome, Some(Outcome::TimeUp));
        let mut r = Raid::new(&map, RaidSetup::free(&map, 3, stats(), 9));
        r.player.hp = 1.0;
        r.hurt_player(5.0, v2(0.0, 0.0), 1.0);
        r.update(&map, 0.02, &Controls::default());
        assert_eq!(r.outcome, Some(Outcome::Caught));
        assert!(r.report().dogs.is_empty(), "nothing is kept when caught");
    }

    #[test]
    fn free_patrols_hold_together_for_a_long_time() {
        let map = Map::hoan_kiem();
        for seed in 0..4 {
            let mut r = Raid::new(&map, RaidSetup::free(&map, 8, stats(), seed));
            let mut rng = Rng::new(seed);
            let mut mv = V2::default();
            for f in 0..3000 {
                if f % 40 == 0 {
                    mv = V2::from_angle(rng.range_f(-PI, PI));
                }
                let c = Controls { mv, attack: f % 17 == 0, throw: f % 97 == 0, roll: f % 61 == 0, interact: f % 3 == 0, ..Default::default() };
                r.update(&map, 1.0 / 30.0, &c);
                assert!(r.player.pos.x.is_finite() && map.circle_free(r.player.pos, PLAYER_R - 0.01));
                if r.outcome.is_some() {
                    break;
                }
            }
        }
    }
}

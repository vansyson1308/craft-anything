//! Sound. Only what the artist delivered and QC passed plays (the files listed in the art
//! manifest); with none of it, the game is silent as before.
//!
//! - A caged dog whimpers every few seconds from where its cage stands, so a player hears the way
//!   to it before seeing it (3D sound: louder and to one side as they turn).
//! - A freed dog barks from its cage.
//! - Buttons click, and a tip arrives with a soft chime.

use bevy::audio::{SpatialScale, Volume};
use bevy::prelude::*;

use crate::cage::CageDoor;
use crate::hq::Hq;

/// The art manifest's names for each sound; the first delivered of each list is used.
const BARKS: [&str; 4] = ["audio/cho_sua_01.ogg", "audio/cho_sua_02.ogg", "audio/cho_sua_03.ogg", "audio/cho_sua_04.ogg"];
const WHIMPER: &str = "audio/cho_rit.ogg";
const CLICK: &str = "audio/ui_bam.ogg";
const TIP: &str = "audio/ui_meo.ogg";

/// How far a whimper carries: past this a cage stays quiet (saves mixing sounds nobody hears).
const WHIMPER_REACH_M: f32 = 45.0;
/// World metres to the units the mixer fades sound by: a cage 40 m off is still faintly heard.
const SPATIAL_SCALE: f32 = 0.18;

/// The sounds that were delivered, loaded once the art manifest is read.
#[derive(Resource, Default)]
pub struct Sounds {
    loaded: bool,
    barks: Vec<Handle<AudioSource>>,
    whimper: Option<Handle<AudioSource>>,
    click: Option<Handle<AudioSource>>,
    tip: Option<Handle<AudioSource>>,
}

/// Sounds asked for by other parts of the game: a menu button was pressed.
#[derive(Resource, Default)]
pub struct Cue {
    pub click: bool,
}

/// Each cage's time to its next whimper, and the cages whose dog was freed (and barked).
#[derive(Default)]
pub struct Heard {
    waits: std::collections::HashMap<Entity, f32>,
    freed: std::collections::HashSet<Entity>,
}

/// A tiny random source for variety (which bark, small changes of pitch, gaps between whimpers).
#[derive(Default)]
pub struct Dice(u32);

impl Dice {
    /// A number from 0 to 1.
    pub fn roll(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.0 >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// Seconds until a caged dog whimpers again.
pub fn next_whimper(d: &mut Dice) -> f32 {
    3.0 + d.roll() * 3.0
}

/// A small change of pitch, so a repeated sound isn't the same each time.
pub fn pitch(d: &mut Dice) -> f32 {
    0.95 + d.roll() * 0.1
}

pub fn load(mut sounds: ResMut<Sounds>, hq: Res<Hq>, assets: Res<AssetServer>) {
    if sounds.loaded || !hq.resolved() {
        return;
    }
    sounds.loaded = true;
    let get = |f: &str| hq.get(f).map(|e| assets.load(Hq::path(&e.file)));
    sounds.barks = BARKS.iter().filter_map(|f| get(f)).collect();
    sounds.whimper = get(WHIMPER);
    sounds.click = get(CLICK);
    sounds.tip = get(TIP);
}

/// Something heard from a place in the world, gone when it ends.
fn spatial(commands: &mut Commands, clip: &Handle<AudioSource>, at: Vec3, volume: f32, speed: f32) {
    commands.spawn((
        AudioPlayer(clip.clone()),
        PlaybackSettings::DESPAWN.with_spatial(true).with_spatial_scale(SpatialScale::new(SPATIAL_SCALE)).with_volume(Volume::Linear(volume)).with_speed(speed),
        Transform::from_translation(at),
    ));
}

/// Dogs in cages whimper; freed dogs bark.
pub fn dogs(
    mut commands: Commands,
    time: Res<Time>,
    sounds: Res<Sounds>,
    cages: Query<(Entity, &CageDoor, &GlobalTransform)>,
    ear: Query<&GlobalTransform, With<SpatialListener>>,
    mut heard: Local<(Heard, Dice)>,
) {
    let (Heard { waits, freed }, dice) = &mut *heard;
    let ear = ear.single().map(|t| t.translation()).ok();
    for (e, door, at) in &cages {
        let at = at.translation() + Vec3::Y * 0.4;
        if door.open {
            if freed.insert(e)
                && !sounds.barks.is_empty()
                && let Some(b) = sounds.barks.get((dice.roll() * sounds.barks.len() as f32) as usize % sounds.barks.len())
            {
                spatial(&mut commands, b, at, 1.0, pitch(dice));
            }
            continue;
        }
        let Some(clip) = &sounds.whimper else { continue };
        let wait = waits.entry(e).or_insert_with(|| 1.0 + dice.roll() * 2.0);
        *wait -= time.delta_secs();
        if *wait > 0.0 {
            continue;
        }
        *wait = next_whimper(dice);
        if ear.is_none_or(|ear| ear.distance(at) < WHIMPER_REACH_M) {
            spatial(&mut commands, clip, at, 0.8, pitch(dice));
        }
    }
    // Cages of a finished raid are gone: forget them.
    waits.retain(|e, _| cages.contains(*e));
    freed.retain(|e| cages.contains(*e));
}

/// Buttons click; a new tip chimes.
pub fn ui(mut commands: Commands, sounds: Res<Sounds>, mut cue: ResMut<Cue>, guide: Res<crate::guide::Guide>, mut last_tip: Local<Option<String>>) {
    if std::mem::take(&mut cue.click)
        && let Some(c) = &sounds.click
    {
        commands.spawn((AudioPlayer(c.clone()), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.6))));
    }
    let tip = guide.tip.clone();
    if tip != *last_tip {
        if tip.is_some()
            && let Some(t) = &sounds.tip
        {
            commands.spawn((AudioPlayer(t.clone()), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5))));
        }
        *last_tip = tip;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whimpers_come_every_few_seconds_at_small_changes_of_pitch() {
        let mut d = Dice(7);
        for _ in 0..1000 {
            let w = next_whimper(&mut d);
            assert!((3.0..6.0).contains(&w), "{w}");
            let p = pitch(&mut d);
            assert!((0.95..1.05).contains(&p), "{p}");
        }
        // Not stuck on one value.
        let a: Vec<f32> = (0..5).map(|_| d.roll()).collect();
        assert!(a.windows(2).any(|w| w[0] != w[1]));
    }
}

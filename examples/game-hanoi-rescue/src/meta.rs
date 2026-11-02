//! What carries over between trips: money, upgrades, rescued dogs, story progress. Saved in the
//! browser as versioned JSON.

use serde::{Deserialize, Serialize};

use crate::content::{self, CHAPTERS, UPGRADES, UpgradeKind};
use crate::raid::{Mission, Outcome, PlayerStats, Report};

pub const SAVE_KEY: &str = "giai-cuu-cun-save";
pub const SAVE_VERSION: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Upgrades {
    pub shoes: u8,
    pub bag: u8,
    pub jacket: u8,
    pub leash: u8,
    pub pouch: u8,
}

impl Upgrades {
    pub fn level(&self, k: UpgradeKind) -> u8 {
        match k {
            UpgradeKind::Shoes => self.shoes,
            UpgradeKind::Bag => self.bag,
            UpgradeKind::Jacket => self.jacket,
            UpgradeKind::Leash => self.leash,
            UpgradeKind::Pouch => self.pouch,
        }
    }
    fn bump(&mut self, k: UpgradeKind) {
        let slot = match k {
            UpgradeKind::Shoes => &mut self.shoes,
            UpgradeKind::Bag => &mut self.bag,
            UpgradeKind::Jacket => &mut self.jacket,
            UpgradeKind::Leash => &mut self.leash,
            UpgradeKind::Pouch => &mut self.pouch,
        };
        *slot = slot.saturating_add(1);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DogRecord {
    pub breed: u8,
    pub name: String,
    #[serde(default)]
    pub puppy: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub version: u32,
    /// In thousand đồng.
    pub money: u32,
    /// The next story chapter (CHAPTERS.len() when the story is done).
    pub chapter: u32,
    pub seen_intro: bool,
    pub upgrades: Upgrades,
    pub dogs: Vec<DogRecord>,
    pub free_level: u32,
    pub raids: u32,
    pub escapes: u32,
    pub tied: u32,
    pub seed: u64,
}

impl Default for Profile {
    fn default() -> Profile {
        Profile {
            version: SAVE_VERSION,
            money: 0,
            chapter: 0,
            seen_intro: false,
            upgrades: Upgrades::default(),
            dogs: Vec::new(),
            free_level: 1,
            raids: 0,
            escapes: 0,
            tied: 0,
            seed: 1,
        }
    }
}

/// What a finished trip changed, for the result screen.
#[derive(Clone, Debug, Default)]
pub struct Earned {
    pub money: u32,
    pub chapter_done: Option<usize>,
    pub new_dogs: Vec<String>,
}

impl Profile {
    /// Reads a save; anything unreadable or from another version starts a new game.
    pub fn load(s: &str) -> Option<Profile> {
        let p: Profile = serde_json::from_str(s).ok()?;
        (p.version == SAVE_VERSION).then_some(p)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn has_progress(&self) -> bool {
        self.seen_intro || self.raids > 0
    }
    pub fn story_done(&self) -> bool {
        self.chapter as usize >= CHAPTERS.len()
    }
    pub fn free_unlocked(&self) -> bool {
        self.chapter >= 1
    }

    pub fn stats(&self) -> PlayerStats {
        let u = &self.upgrades;
        PlayerStats {
            max_hp: 100.0 + 30.0 * u.jacket as f32,
            speed: 4.0 * (1.0 + 0.06 * u.shoes as f32),
            bag: 6 + 3 * u.bag as usize,
            leash: 1 + u.leash as usize,
            slippers: 3 + 3 * u.pouch as u32,
        }
    }

    /// Price of the next level of an upgrade, if there is one.
    pub fn next_price(&self, k: UpgradeKind) -> Option<u32> {
        let up = UPGRADES.iter().find(|u| u.kind == k)?;
        up.prices.get(self.upgrades.level(k) as usize).copied()
    }

    pub fn buy(&mut self, k: UpgradeKind) -> bool {
        match self.next_price(k) {
            Some(p) if self.money >= p => {
                self.money -= p;
                self.upgrades.bump(k);
                true
            }
            _ => false,
        }
    }

    pub fn next_seed(&mut self) -> u64 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.seed
    }

    /// Records a finished trip.
    pub fn apply(&mut self, mission: Mission, r: &Report) -> Earned {
        let mut e = Earned::default();
        self.raids += 1;
        if let Outcome::Escaped(_) = r.outcome {
            self.escapes += 1;
            self.tied += r.tied;
            e.money += r.cash + r.scrap;
            for (breed, name, puppy) in &r.dogs {
                e.money += content::REWARD_PER_DOG;
                self.dogs.push(DogRecord { breed: *breed as u8, name: name.clone(), puppy: *puppy });
                e.new_dogs.push(name.clone());
            }
            match mission {
                Mission::Story(ch) if r.objective_met && ch == self.chapter as usize => {
                    e.money += content::REWARD_CHAPTER.get(ch).copied().unwrap_or(0);
                    self.chapter += 1;
                    e.chapter_done = Some(ch);
                }
                Mission::Free(l) if l >= self.free_level => self.free_level = l + 1,
                _ => {}
            }
        }
        self.money = self.money.saturating_add(e.money);
        e
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::ExitId;

    fn report(outcome: Outcome, met: bool) -> Report {
        Report { outcome, objective_met: met, cash: 15, scrap: 8, dogs: vec![(1, "Mực".into(), false)], tied: 2, seconds: 90.0 }
    }

    #[test]
    fn saves_round_trip_and_bad_saves_start_over() {
        let mut p = Profile { money: 77, ..Profile::default() };
        p.dogs.push(DogRecord { breed: 2, name: "Bông".into(), puppy: false });
        let back = Profile::load(&p.to_json());
        assert_eq!(back.as_ref(), Some(&p));
        assert!(Profile::load("not json").is_none());
        assert!(Profile::load(r#"{"version": 99}"#).is_none());
        // Fields added later default instead of breaking old saves.
        assert_eq!(Profile::load(r#"{"version": 1, "money": 5}"#).map(|p| p.money), Some(5));
    }

    #[test]
    fn a_story_win_pays_and_moves_on_a_loss_keeps_nothing() {
        let mut p = Profile::default();
        let e = p.apply(Mission::Story(0), &report(Outcome::Escaped(ExitId::QuanChuong), true));
        assert_eq!(e.chapter_done, Some(0));
        assert_eq!(p.chapter, 1);
        assert_eq!(p.money, 15 + 8 + content::REWARD_PER_DOG + content::REWARD_CHAPTER[0]);
        assert_eq!(p.dogs.len(), 1);
        let before = p.clone();
        let e = p.apply(Mission::Story(1), &Report { dogs: vec![], cash: 0, scrap: 0, ..report(Outcome::Caught, false) });
        assert_eq!(e.money, 0);
        assert_eq!(p.money, before.money);
        assert_eq!(p.chapter, 1);
        // Escaping without the objective keeps the loot but not the chapter.
        p.apply(Mission::Story(1), &report(Outcome::Escaped(ExitId::Ga), false));
        assert_eq!(p.chapter, 1);
    }

    #[test]
    fn buying_upgrades_changes_stats_until_maxed() {
        let mut p = Profile { money: 1000, ..Profile::default() };
        assert_eq!(p.stats().leash, 1);
        assert!(p.buy(UpgradeKind::Leash));
        assert!(p.buy(UpgradeKind::Leash));
        assert!(!p.buy(UpgradeKind::Leash), "maxed");
        assert_eq!(p.stats().leash, 3);
        assert_eq!(p.money, 1000 - 60 - 150);
        let mut poor = Profile::default();
        assert!(!poor.buy(UpgradeKind::Shoes));
    }

    #[test]
    fn chapter_three_is_affordable_by_then() {
        // Chapter rewards alone (without any loot) pay for the double leash before chapter 3.
        let paid: u32 = content::REWARD_CHAPTER.iter().take(2).sum::<u32>() + 2 * content::REWARD_PER_DOG;
        let price = Profile::default().next_price(UpgradeKind::Leash).unwrap_or(u32::MAX);
        assert!(paid >= price);
    }
}

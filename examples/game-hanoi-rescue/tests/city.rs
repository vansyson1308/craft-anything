//! The real district (OpenStreetMap data from ../game-hanoi-city/assets) runs the same game.

use hanoi_rescue::city::City;
use hanoi_rescue::content::CHAPTERS;
use hanoi_rescue::geom::v2;
use hanoi_rescue::map::{ExitId, Map, Tile};
use hanoi_rescue::raid::{Controls, Outcome, PlayerStats, Raid, RaidSetup};

#[test]
fn the_real_quan_chuong_street_passes_through_the_gate() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../game-hanoi-city/assets/hoankiem.json");
    let Ok(data) = std::fs::read_to_string(path) else { return };
    let city = City::from_json(&data).expect("valid city data");
    let map = Map::from_city(&city, 2.0);
    let point = |x: f32, y: f32| v2((x - city.bounds[0] as f32) / 20.0, (y - city.bounds[1] as f32) / 20.0);
    let from = point(-55.0, -9419.0);
    let to = point(50.0, -9441.0);
    let mut player = from;
    for i in 1..=21 {
        let target = from.lerp(to, i as f32 / 21.0);
        assert!(map.circle_free(target, hanoi_rescue::raid::PLAYER_R), "blocked at step {i}");
        player = map.slide(player, target - player, hanoi_rescue::raid::PLAYER_R);
    }
    assert!(player.dist(to) < 0.05, "the player reached {player:?}, expected {to:?}");
    assert_eq!(map.tile_at(point(-5.0, -9400.0)), Tile::Building, "side wall stays blocked");
    assert_eq!(map.tile_at(point(-5.0, -9460.0)), Tile::Building, "opposite side wall stays blocked");
}

fn city_map() -> Option<Map> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../game-hanoi-city/assets/hoankiem.json");
    let city = City::from_json(&std::fs::read_to_string(path).ok()?)?;
    Some(Map::from_city(&city, 2.0))
}

fn stats() -> PlayerStats {
    PlayerStats { max_hp: 100.0, speed: 4.0, bag: 6, leash: 2, slippers: 3 }
}

#[test]
fn the_real_district_has_its_landmarks_and_every_exit_is_reachable() {
    let Some(map) = city_map() else { return };
    assert!(map.w > 500 && map.h > 500, "{}x{}", map.w, map.h);
    let ids: Vec<ExitId> = map.exits.iter().map(|e| e.id).collect();
    for id in [ExitId::QuanChuong, ExitId::LongBien, ExitId::NhaHat, ExitId::Ga] {
        assert!(ids.contains(&id), "missing exit {id:?}");
    }
    for name in ["HỒ GƯƠM", "NHÀ THỜ LỚN", "CHỢ ĐỒNG XUÂN", "Ô QUAN CHƯỞNG", "CẦU LONG BIÊN"] {
        assert!(map.labels.iter().any(|l| l.text == name), "missing {name}");
    }
    let lake = map.labels.iter().find(|l| l.text == "HỒ GƯƠM").map(|l| l.at).unwrap_or_default();
    assert_eq!(map.tile_at(lake), Tile::Water, "the lake label sits on water");
    assert!(map.rail.as_ref().is_some_and(|r| r.len > 200.0), "a long railway");
    let reach = map.distances(map.scenario.spawns.first().copied().unwrap_or_default().tile());
    for e in &map.exits {
        let (x, y) = e.center().tile();
        assert!(reach.get((y * map.w + x) as usize).copied().unwrap_or(-1) >= 0, "can't reach {}", e.name);
    }
}

#[test]
fn every_chapter_builds_on_the_real_streets() {
    let Some(map) = city_map() else { return };
    for ch in 0..CHAPTERS.len() {
        let r = Raid::new(&map, RaidSetup::story(&map, ch, stats(), 3));
        assert_eq!(r.cages.len() as u32, r.setup.cages, "chapter {ch}");
        assert!(r.thieves.len() >= r.setup.diff.thieves as usize, "chapter {ch}");
        assert!(map.walkable_at(r.player.pos), "chapter {ch} starts on a wall");
    }
}

#[test]
fn chapter_one_can_be_won_on_the_real_streets() {
    let Some(map) = city_map() else { return };
    let mut r = Raid::new(&map, RaidSetup::story(&map, 0, stats(), 4));
    r.thieves.clear();
    r.vehicles.clear();
    let walk = |r: &mut Raid, goal: hanoi_rescue::geom::V2| {
        let path = map.path(r.player.pos, goal, 400_000).unwrap_or_default();
        for q in path {
            for _ in 0..200 {
                if r.player.pos.dist(q) < 0.35 || r.outcome.is_some() {
                    break;
                }
                let mv = (q - r.player.pos).norm();
                r.update(&map, 1.0 / 30.0, &Controls { mv, ..Default::default() });
            }
        }
    };
    let cage = r.cages.first().map(|c| c.pos).unwrap_or_default();
    walk(&mut r, cage);
    for _ in 0..60 {
        r.update(&map, 1.0 / 30.0, &Controls { interact: true, ..Default::default() });
    }
    assert_eq!(r.following(), 1);
    let exit = map.exits.iter().find(|e| e.id == ExitId::QuanChuong).map(|e| e.center()).unwrap_or_default();
    walk(&mut r, exit);
    for _ in 0..120 {
        r.update(&map, 1.0 / 30.0, &Controls::default());
    }
    assert_eq!(r.outcome, Some(Outcome::Escaped(ExitId::QuanChuong)), "time left {}", r.time_left);
    let _ = v2(0.0, 0.0);
}

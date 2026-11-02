//! Turns OpenStreetMap XML (from the API's `map?bbox=` calls) into the game's city file.
//!
//! `cargo run --release --example osm_import -- <out.json> <lat0> <lon0> <south> <west> <north> <east> <tile.osm>...`
//!
//! Keeps buildings (footprints, heights from `height` or `building:levels`), roads with widths,
//! water and parks (ways and multipolygon relations, with holes), the railway, trees and named
//! places. Map data © OpenStreetMap contributors (ODbL).

use std::collections::{HashMap, HashSet};

use hanoi_rescue::city::{Area, Building, CITY_VERSION, City, Dm, Poi, Road};

#[derive(Default)]
struct Way {
    nodes: Vec<i64>,
    tags: HashMap<String, String>,
}

#[derive(Default)]
struct Rel {
    members: Vec<(String, i64, String)>, // type, ref, role
    tags: HashMap<String, String>,
}

fn attr<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!(" {key}=\"");
    let start = line.find(&pat)? + pat.len();
    let rest = line.get(start..)?;
    rest.find('"').and_then(|end| rest.get(..end))
}

fn unescape(s: &str) -> String {
    s.replace("&quot;", "\"").replace("&apos;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

/// Joins way segments into closed rings.
fn rings(segments: Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    let mut open: Vec<Vec<i64>> = Vec::new();
    let mut done = Vec::new();
    for s in segments {
        if s.len() < 2 {
            continue;
        }
        if s.first() == s.last() {
            done.push(s);
        } else {
            open.push(s);
        }
    }
    while let Some(mut cur) = open.pop() {
        let mut grown = true;
        while grown && cur.first() != cur.last() {
            grown = false;
            let end = cur.last().copied();
            if let Some(i) = open.iter().position(|o| o.first().copied() == end || o.last().copied() == end) {
                let mut o = open.swap_remove(i);
                if o.first().copied() != end {
                    o.reverse();
                }
                cur.extend(o.into_iter().skip(1));
                grown = true;
            }
        }
        if cur.first() == cur.last() && cur.len() > 3 {
            done.push(cur);
        }
    }
    done
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 8 {
        eprintln!("usage: osm_import <out.json> <lat0> <lon0> <south> <west> <north> <east> <tile.osm>...");
        std::process::exit(2);
    }
    let num = |i: usize| args.get(i).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let (lat0, lon0) = (num(1), num(2));
    let (south, west, north, east) = (num(3), num(4), num(5), num(6));
    let kx = 111_320.0 * lat0.to_radians().cos();
    let ky = 110_540.0;
    let proj = |lat: f64, lon: f64| -> Dm { [((lon - lon0) * kx * 10.0).round() as i32, (-(lat - lat0) * ky * 10.0).round() as i32] };
    let b0 = proj(north, west);
    let b1 = proj(south, east);
    let bounds = [b0[0], b0[1], b1[0], b1[1]];
    let inside = |p: &Dm| p[0] >= bounds[0] && p[0] <= bounds[2] && p[1] >= bounds[1] && p[1] <= bounds[3];

    let mut nodes: HashMap<i64, Dm> = HashMap::new();
    let mut node_tags: HashMap<i64, HashMap<String, String>> = HashMap::new();
    let mut ways: HashMap<i64, Way> = HashMap::new();
    let mut rels: HashMap<i64, Rel> = HashMap::new();
    for path in args.iter().skip(7) {
        let Ok(text) = std::fs::read_to_string(path) else {
            eprintln!("skip {path}");
            continue;
        };
        enum Cur {
            None,
            Node(i64),
            Way(i64),
            Rel(i64),
        }
        let mut cur = Cur::None;
        let mut way = Way::default();
        let mut rel = Rel::default();
        let mut ntags: HashMap<String, String> = HashMap::new();
        for line in text.lines() {
            let l = line.trim_start();
            if l.starts_with("<node ") {
                let id = attr(l, "id").and_then(|v| v.parse().ok()).unwrap_or(0);
                let lat = attr(l, "lat").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                let lon = attr(l, "lon").and_then(|v| v.parse().ok()).unwrap_or(0.0);
                nodes.insert(id, proj(lat, lon));
                cur = if l.ends_with("/>") { Cur::None } else { Cur::Node(id) };
                ntags.clear();
            } else if l.starts_with("</node>") {
                if let Cur::Node(id) = cur
                    && !ntags.is_empty()
                {
                    node_tags.insert(id, std::mem::take(&mut ntags));
                }
                cur = Cur::None;
            } else if l.starts_with("<way ") {
                cur = Cur::Way(attr(l, "id").and_then(|v| v.parse().ok()).unwrap_or(0));
                way = Way::default();
            } else if l.starts_with("</way>") {
                if let Cur::Way(id) = cur {
                    ways.entry(id).or_insert_with(|| std::mem::take(&mut way));
                }
                cur = Cur::None;
            } else if l.starts_with("<relation ") {
                cur = Cur::Rel(attr(l, "id").and_then(|v| v.parse().ok()).unwrap_or(0));
                rel = Rel::default();
            } else if l.starts_with("</relation>") {
                if let Cur::Rel(id) = cur {
                    rels.entry(id).or_insert_with(|| std::mem::take(&mut rel));
                }
                cur = Cur::None;
            } else if l.starts_with("<nd ") {
                if let Some(r) = attr(l, "ref").and_then(|v| v.parse().ok()) {
                    way.nodes.push(r);
                }
            } else if l.starts_with("<member ") {
                let t = attr(l, "type").unwrap_or("").to_string();
                let r = attr(l, "ref").and_then(|v| v.parse().ok()).unwrap_or(0);
                let role = attr(l, "role").unwrap_or("").to_string();
                rel.members.push((t, r, role));
            } else if l.starts_with("<tag ") {
                let (Some(k), Some(v)) = (attr(l, "k"), attr(l, "v")) else { continue };
                let (k, v) = (unescape(k), unescape(v));
                match cur {
                    Cur::Node(_) => {
                        ntags.insert(k, v);
                    }
                    Cur::Way(_) => {
                        way.tags.insert(k, v);
                    }
                    Cur::Rel(_) => {
                        rel.tags.insert(k, v);
                    }
                    Cur::None => {}
                }
            }
        }
    }
    eprintln!("{} nodes, {} ways, {} relations", nodes.len(), ways.len(), rels.len());

    let pts = |ids: &[i64]| -> Vec<Dm> { ids.iter().filter_map(|i| nodes.get(i).copied()).collect() };
    let any_inside = |p: &[Dm]| p.iter().any(inside);
    let name_of = |t: &HashMap<String, String>| t.get("name").or_else(|| t.get("name:vi")).cloned();
    let height_of = |t: &HashMap<String, String>, id: i64| -> i32 {
        let parse = |s: &String| s.trim_end_matches('m').trim().replace(',', ".").parse::<f32>().ok();
        if let Some(h) = t.get("height").and_then(parse) {
            return (h * 10.0) as i32;
        }
        if let Some(l) = t.get("building:levels").and_then(parse) {
            return ((l * 3.3 + 1.0) * 10.0) as i32;
        }
        // Tube houses in the old quarter: 3 to 5 floors.
        let floors = 3 + (id.rem_euclid(97) % 3) as i32;
        floors * 33 + 10
    };
    let is_area = |t: &HashMap<String, String>| -> Option<&'static str> {
        if t.contains_key("building") || t.contains_key("building:part") {
            Some("building")
        } else if t.get("natural").is_some_and(|v| v == "water") || t.get("waterway").is_some_and(|v| v == "riverbank") {
            Some("water")
        } else if t.get("leisure").is_some_and(|v| v == "park" || v == "garden")
            || t.get("landuse").is_some_and(|v| v == "grass" || v == "recreation_ground")
            || t.get("natural").is_some_and(|v| v == "wood" || v == "grassland")
        {
            Some("park")
        } else {
            None
        }
    };

    let mut city = City {
        version: CITY_VERSION,
        origin: [lat0, lon0],
        bounds,
        attribution: "Map data © OpenStreetMap contributors (ODbL), openstreetmap.org/copyright".to_string(),
        ..City::default()
    };
    let mut used_in_rel: HashSet<i64> = HashSet::new();

    // Multipolygon relations (the lake with its islands, big buildings).
    for rel in rels.values() {
        if rel.tags.get("type").map(String::as_str) != Some("multipolygon") {
            continue;
        }
        let Some(kind) = is_area(&rel.tags) else { continue };
        let mut outers = Vec::new();
        let mut inners = Vec::new();
        for (t, r, role) in &rel.members {
            if t != "way" {
                continue;
            }
            let Some(w) = ways.get(r) else { continue };
            if is_area(&w.tags).is_none() {
                used_in_rel.insert(*r);
            }
            if role == "inner" { inners.push(w.nodes.clone()) } else { outers.push(w.nodes.clone()) }
        }
        let holes: Vec<Vec<Dm>> = rings(inners).iter().map(|r| pts(r)).collect();
        for outer in rings(outers) {
            let outer = pts(&outer);
            if outer.len() < 4 || !any_inside(&outer) {
                continue;
            }
            match kind {
                "building" => city.buildings.push(Building {
                    outer,
                    holes: holes.clone(),
                    h: height_of(&rel.tags, outer_id(&rel.tags)),
                    name: name_of(&rel.tags),
                    kind: rel.tags.get("building").cloned().unwrap_or_default(),
                }),
                "water" => city.water.push(Area { outer, holes: holes.clone(), name: name_of(&rel.tags) }),
                _ => city.parks.push(Area { outer, holes: holes.clone(), name: name_of(&rel.tags) }),
            }
        }
    }

    let mut ids: Vec<&i64> = ways.keys().collect();
    ids.sort();
    for id in ids {
        let Some(w) = ways.get(id) else { continue };
        let p = pts(&w.nodes);
        if p.len() < 2 || !any_inside(&p) {
            continue;
        }
        let closed = w.nodes.first() == w.nodes.last() && w.nodes.len() > 3;
        if let Some(hw) = w.tags.get("highway") {
            let width = match hw.as_str() {
                "primary" | "trunk" => 120,
                "secondary" => 100,
                "tertiary" => 80,
                "residential" | "unclassified" => 65,
                "living_street" | "pedestrian" => 60,
                "service" => 40,
                "footway" | "path" | "cycleway" | "steps" => 25,
                _ => continue,
            };
            let width = w.tags.get("width").and_then(|s| s.trim_end_matches('m').trim().parse::<f32>().ok()).map_or(width, |m| (m * 10.0) as i32);
            city.roads.push(Road {
                pts: p,
                w: width.clamp(15, 300),
                kind: hw.clone(),
                name: name_of(&w.tags),
                bridge: w.tags.get("bridge").is_some_and(|v| v != "no"),
            });
            continue;
        }
        if w.tags.get("railway").is_some_and(|v| v == "rail") {
            city.rails.push(p);
            continue;
        }
        if !closed || used_in_rel.contains(id) {
            continue;
        }
        match is_area(&w.tags) {
            Some("building") => city.buildings.push(Building {
                outer: p,
                holes: Vec::new(),
                h: height_of(&w.tags, *id),
                name: name_of(&w.tags),
                kind: w.tags.get("building").cloned().unwrap_or_default(),
            }),
            Some("water") => city.water.push(Area { outer: p, holes: Vec::new(), name: name_of(&w.tags) }),
            Some("park") => city.parks.push(Area { outer: p, holes: Vec::new(), name: name_of(&w.tags) }),
            _ => {
                // Named closed ways that aren't areas we draw still name a place.
                if let (Some(n), Some(c)) = (name_of(&w.tags), hanoi_rescue::city::centroid(&p))
                    && (w.tags.contains_key("historic") || w.tags.contains_key("tourism") || w.tags.contains_key("amenity"))
                {
                    city.pois.push(Poi { at: c, name: n, kind: "place".into() });
                }
            }
        }
    }

    let mut nids: Vec<&i64> = node_tags.keys().collect();
    nids.sort();
    for id in nids {
        let (Some(t), Some(p)) = (node_tags.get(id), nodes.get(id)) else { continue };
        if !inside(p) {
            continue;
        }
        if t.get("natural").is_some_and(|v| v == "tree") {
            city.trees.push(*p);
        }
        let Some(name) = name_of(t) else { continue };
        let kind = ["shop", "amenity", "tourism", "historic", "railway", "leisure", "office", "craft"]
            .iter()
            .find_map(|k| t.get(*k).map(|v| format!("{k}:{v}")))
            .unwrap_or_else(|| "place".into());
        // Chains and brands are marked so the game never paints their names on a sign.
        let branded = t.contains_key("brand") || t.contains_key("brand:wikidata") || t.contains_key("operator:wikidata");
        let kind = if branded { format!("brand:{kind}") } else { kind };
        city.pois.push(Poi { at: *p, name, kind });
    }
    for b in &city.buildings {
        if let (Some(n), Some(c)) = (&b.name, hanoi_rescue::city::centroid(&b.outer)) {
            city.pois.push(Poi { at: c, name: n.clone(), kind: format!("building:{}", b.kind) });
        }
    }

    eprintln!(
        "kept {} buildings, {} roads, {} water, {} parks, {} rails, {} trees, {} named places",
        city.buildings.len(),
        city.roads.len(),
        city.water.len(),
        city.parks.len(),
        city.rails.len(),
        city.trees.len(),
        city.pois.len()
    );
    let out = serde_json::to_string(&city).unwrap_or_default();
    if let Err(e) = std::fs::write(args.first().map_or("city.json", String::as_str), out) {
        eprintln!("couldn't write the city file: {e}");
        std::process::exit(1);
    }
}

fn outer_id(t: &HashMap<String, String>) -> i64 {
    t.get("name").map_or(7, |n| n.len() as i64)
}

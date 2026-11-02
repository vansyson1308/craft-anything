//! A real city, from OpenStreetMap: building footprints with heights, roads with widths, water,
//! parks, the railway, trees and named places, in metres around an origin (x east, y south).
//!
//! Map data © OpenStreetMap contributors, available under the Open Database Licence (ODbL).

use serde::{Deserialize, Serialize};

pub const CITY_VERSION: u32 = 1;

/// A point in decimetres (x east, y south), to keep the file small.
pub type Dm = [i32; 2];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Building {
    pub outer: Vec<Dm>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holes: Vec<Vec<Dm>>,
    /// Height in decimetres.
    pub h: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Road {
    pub pts: Vec<Dm>,
    /// Width in decimetres.
    pub w: i32,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub bridge: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Area {
    pub outer: Vec<Dm>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub holes: Vec<Vec<Dm>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Poi {
    pub at: Dm,
    pub name: String,
    pub kind: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct City {
    pub version: u32,
    /// Latitude and longitude of (0, 0).
    pub origin: [f64; 2],
    /// x0, y0, x1, y1 in decimetres: the area kept.
    pub bounds: [i32; 4],
    pub buildings: Vec<Building>,
    pub roads: Vec<Road>,
    pub water: Vec<Area>,
    pub parks: Vec<Area>,
    pub rails: Vec<Vec<Dm>>,
    pub trees: Vec<Dm>,
    pub pois: Vec<Poi>,
    pub attribution: String,
}

impl City {
    pub fn from_json(s: &str) -> Option<City> {
        let c: City = serde_json::from_str(s).ok()?;
        (c.version == CITY_VERSION).then_some(c)
    }

    /// A named place, matched case-insensitively on the whole name or its start.
    pub fn place(&self, name: &str) -> Option<Dm> {
        let want = name.to_lowercase();
        self.pois
            .iter()
            .find(|p| p.name.to_lowercase() == want)
            .map(|p| p.at)
            .or_else(|| self.buildings.iter().find(|b| b.name.as_deref().is_some_and(|n| n.to_lowercase() == want)).and_then(|b| centroid(&b.outer)))
    }

    /// The middle of the longest stretch of a named road.
    pub fn road(&self, name: &str) -> Option<Dm> {
        let want = name.to_lowercase();
        self.roads
            .iter()
            .filter(|r| r.name.as_deref().is_some_and(|n| n.to_lowercase() == want))
            .max_by_key(|r| r.pts.len())
            .and_then(|r| r.pts.get(r.pts.len() / 2).copied())
    }
}

pub fn centroid(pts: &[Dm]) -> Option<Dm> {
    if pts.is_empty() {
        return None;
    }
    let n = pts.len() as i64;
    let (sx, sy) = pts.iter().fold((0i64, 0i64), |(a, b), p| (a + p[0] as i64, b + p[1] as i64));
    Some([(sx / n) as i32, (sy / n) as i32])
}

/// Whether a point is inside a ring (even-odd rule).
pub fn in_ring(x: f32, y: f32, ring: &[[f32; 2]]) -> bool {
    let mut inside = false;
    let n = ring.len();
    for i in 0..n {
        let (Some(a), Some(b)) = (ring.get(i), ring.get((i + n - 1) % n)) else { continue };
        if (a[1] > y) != (b[1] > y) {
            let t = (y - a[1]) / (b[1] - a[1]);
            if x < a[0] + t * (b[0] - a[0]) {
                inside = !inside;
            }
        }
    }
    inside
}

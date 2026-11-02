//! Stand-in pictures drawn in Rust: each seed gives the same small SVG every time.
//!
//! The original shows people's generations; a rebuild has none of its own, so every card gets
//! an abstract scene (sky gradient, glowing shapes, a horizon) instead of borrowed media.

use std::fmt::Write as _;

/// A tiny deterministic generator (SplitMix64), so pictures never change between runs.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A number in `lo..hi` (`lo` when the range is empty).
    fn range(&mut self, lo: u32, hi: u32) -> u32 {
        let span = u64::from(hi.saturating_sub(lo)).max(1);
        lo.saturating_add(u32::try_from(self.next() % span).unwrap_or(0))
    }
}

/// The SVG for `seed`, `w` by `h` (each clamped to 16..=2000).
pub fn svg(seed: u32, w: u32, h: u32) -> String {
    let w = w.clamp(16, 2000);
    let h = h.clamp(16, 2000);
    let mut r = Rng::new(u64::from(seed));
    let hue = r.range(0, 360);
    let hue2 = (hue + r.range(30, 150)) % 360;
    let hue3 = (hue + 180) % 360;
    let mut s = String::with_capacity(2048);
    // `write!` into a String cannot fail; the results are ignored on purpose.
    let _ = write!(s, r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" preserveAspectRatio="xMidYMid slice">"##);
    let _ = write!(
        s,
        r##"<defs><linearGradient id="sky" x1="0" y1="0" x2="0.4" y2="1"><stop offset="0" stop-color="hsl({hue},{s1}%,{l1}%)"/><stop offset="1" stop-color="hsl({hue2},{s2}%,{l2}%)"/></linearGradient><filter id="blur"><feGaussianBlur stdDeviation="{blur}"/></filter><radialGradient id="glow"><stop offset="0" stop-color="hsl({hue3},90%,75%)" stop-opacity="0.9"/><stop offset="1" stop-color="hsl({hue3},90%,60%)" stop-opacity="0"/></radialGradient></defs>"##,
        s1 = r.range(35, 80),
        l1 = r.range(18, 55),
        s2 = r.range(30, 70),
        l2 = r.range(6, 22),
        blur = (w.min(h) / 14).max(2),
    );
    let _ = write!(s, r##"<rect width="{w}" height="{h}" fill="url(#sky)"/>"##);
    // Soft blobs of light.
    let _ = write!(s, r##"<g filter="url(#blur)">"##);
    for _ in 0..r.range(2, 5) {
        let cx = r.range(0, w);
        let cy = r.range(0, h);
        let rad = r.range(w.min(h) / 8, w.min(h) / 3 + 2);
        let hh = (hue + r.range(0, 90)) % 360;
        let op = r.range(25, 70);
        let _ = write!(s, r##"<circle cx="{cx}" cy="{cy}" r="{rad}" fill="hsl({hh},85%,62%)" opacity="0.{op}"/>"##);
    }
    s.push_str("</g>");
    // A sun or moon.
    let cx = r.range(w / 6, w - w / 6);
    let cy = r.range(h / 8, h / 2);
    let rad = r.range(w.min(h) / 10, w.min(h) / 5 + 2);
    let _ = write!(s, r##"<circle cx="{cx}" cy="{cy}" r="{}" fill="url(#glow)"/>"##, rad.saturating_mul(3));
    let _ = write!(s, r##"<circle cx="{cx}" cy="{cy}" r="{rad}" fill="hsl({hue3},70%,82%)" opacity="0.85"/>"##);
    // Two layers of hills along a horizon.
    for layer in 0..2u32 {
        let base = h - h / (3 + layer * 2);
        let mut d = format!("M0 {h} L0 {base}");
        let steps = r.range(3, 7);
        for i in 1..=steps {
            let x = w * i / steps;
            let y = base.saturating_sub(r.range(0, h / 6 + 1)).saturating_add(r.range(0, h / 12 + 1));
            let _ = write!(d, " Q{} {} {x} {y}", x.saturating_sub(w / steps / 2), y.saturating_sub(r.range(0, h / 8 + 1)));
        }
        let _ = write!(d, " L{w} {h} Z");
        let light = 4 + layer * 5;
        let _ = write!(s, r##"<path d="{d}" fill="hsl({hue2},30%,{light}%)" opacity="0.9"/>"##);
    }
    // Film grain lines, for texture.
    for _ in 0..r.range(0, 4) {
        let y = r.range(0, h);
        let _ = write!(s, r##"<rect x="0" y="{y}" width="{w}" height="1" fill="#fff" opacity="0.05"/>"##);
    }
    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_picture() {
        assert_eq!(svg(7, 400, 300), svg(7, 400, 300));
        assert_ne!(svg(7, 400, 300), svg(8, 400, 300));
    }

    #[test]
    fn hostile_sizes_are_clamped() {
        for (w, h) in [(0, 0), (u32::MAX, u32::MAX), (1, 99_999)] {
            let s = svg(u32::MAX, w, h);
            assert!(s.starts_with("<svg") && s.ends_with("</svg>"));
        }
    }
}

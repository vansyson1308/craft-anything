//! What a frame looks like, as a list of simple shapes in screen pixels. The web layer paints them
//! on a canvas; tests can look at them without a browser.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        c: u32,
        a: f32,
    },
    RRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        r: f32,
        c: u32,
        a: f32,
    },
    Circle {
        x: f32,
        y: f32,
        r: f32,
        c: u32,
        a: f32,
    },
    Ring {
        x: f32,
        y: f32,
        r: f32,
        w: f32,
        c: u32,
        a: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        w: f32,
        c: u32,
        a: f32,
    },
    /// A filled pie slice (angles in radians, clockwise from +x).
    Wedge {
        x: f32,
        y: f32,
        r: f32,
        a0: f32,
        a1: f32,
        c: u32,
        a: f32,
    },
    /// An arc stroke, for progress rings.
    Arc {
        x: f32,
        y: f32,
        r: f32,
        a0: f32,
        a1: f32,
        w: f32,
        c: u32,
        a: f32,
    },
    Poly {
        pts: Vec<(f32, f32)>,
        c: u32,
        a: f32,
    },
    Text {
        x: f32,
        y: f32,
        s: String,
        size: f32,
        c: u32,
        a: f32,
        align: Align,
        bold: bool,
        outline: bool,
    },
    /// The minimap picture of the whole district, stretched over this box.
    Minimap {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        a: f32,
    },
}

/// How wide text is, so it can be wrapped and centred. The browser measures for real.
pub trait Measure {
    fn text_width(&self, s: &str, size: f32, bold: bool) -> f32;
}

/// An estimate good enough for tests.
pub struct ApproxMeasure;

impl Measure for ApproxMeasure {
    fn text_width(&self, s: &str, size: f32, bold: bool) -> f32 {
        s.chars().count() as f32 * size * if bold { 0.58 } else { 0.53 }
    }
}

/// Splits `text` into lines no wider than `max_w`.
pub fn wrap(m: &dyn Measure, text: &str, size: f32, bold: bool, max_w: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split_whitespace() {
            let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
            if !line.is_empty() && m.text_width(&candidate, size, bold) > max_w {
                lines.push(std::mem::take(&mut line));
                line = word.to_string();
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    lines
}

/// A frame being built.
#[derive(Default)]
pub struct Canvas {
    pub list: Vec<Draw>,
}

impl Canvas {
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32, c: u32, a: f32) {
        self.list.push(Draw::Rect { x, y, w, h, c, a });
    }
    pub fn rrect(&mut self, x: f32, y: f32, w: f32, h: f32, r: f32, c: u32, a: f32) {
        self.list.push(Draw::RRect { x, y, w, h, r, c, a });
    }
    pub fn circle(&mut self, x: f32, y: f32, r: f32, c: u32, a: f32) {
        self.list.push(Draw::Circle { x, y, r, c, a });
    }
    pub fn ring(&mut self, x: f32, y: f32, r: f32, w: f32, c: u32, a: f32) {
        self.list.push(Draw::Ring { x, y, r, w, c, a });
    }
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, w: f32, c: u32, a: f32) {
        self.list.push(Draw::Line { x1, y1, x2, y2, w, c, a });
    }
    pub fn wedge(&mut self, x: f32, y: f32, r: f32, a0: f32, a1: f32, c: u32, a: f32) {
        self.list.push(Draw::Wedge { x, y, r, a0, a1, c, a });
    }
    pub fn arc(&mut self, x: f32, y: f32, r: f32, a0: f32, a1: f32, w: f32, c: u32, a: f32) {
        self.list.push(Draw::Arc { x, y, r, a0, a1, w, c, a });
    }
    pub fn poly(&mut self, pts: Vec<(f32, f32)>, c: u32, a: f32) {
        self.list.push(Draw::Poly { pts, c, a });
    }
    pub fn text(&mut self, x: f32, y: f32, s: impl Into<String>, size: f32, c: u32, align: Align, bold: bool) {
        self.list.push(Draw::Text { x, y, s: s.into(), size, c, a: 1.0, align, bold, outline: false });
    }
    /// Text with a dark outline, readable over the map.
    pub fn label(&mut self, x: f32, y: f32, s: impl Into<String>, size: f32, c: u32, a: f32, bold: bool) {
        self.list.push(Draw::Text { x, y, s: s.into(), size, c, a, align: Align::Center, bold, outline: true });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapping_keeps_every_word_and_fits() {
        let m = ApproxMeasure;
        let text = "Hà Nội, cuối thu. Minh, sinh viên năm hai, thuê một phòng trọ nhỏ sau phố cổ.";
        let lines = wrap(&m, text, 16.0, false, 160.0);
        assert!(lines.len() > 2);
        assert_eq!(lines.join(" "), text);
        for l in &lines {
            assert!(m.text_width(l, 16.0, false) <= 160.0 || !l.contains(' '));
        }
        assert_eq!(wrap(&m, "", 16.0, false, 100.0), vec![String::new()]);
    }
}

//! Touch, mouse and keyboard, gathered into one place each frame.

use crate::geom::V2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PEvent {
    Down { id: i32, x: f32, y: f32, touch: bool },
    Move { id: i32, x: f32, y: f32 },
    Up { id: i32, x: f32, y: f32 },
    Cancel { id: i32 },
}

#[derive(Clone, Copy, Debug)]
pub struct Pointer {
    pub id: i32,
    pub start: V2,
    pub pos: V2,
}

#[derive(Default)]
pub struct Input {
    pub pointers: Vec<Pointer>,
    /// Pointers that went down this frame.
    pub downs: Vec<V2>,
    /// Taps finished this frame: where they started and ended.
    pub taps: Vec<(V2, V2)>,
    held: Vec<String>,
    pressed: Vec<String>,
    /// The last pointer was a finger (show on-screen controls).
    pub touch_mode: bool,
}

const TAP_SLOP: f32 = 26.0;

impl Input {
    pub fn pointer(&mut self, e: PEvent) {
        match e {
            PEvent::Down { id, x, y, touch } => {
                self.touch_mode = touch;
                let p = V2 { x, y };
                self.pointers.retain(|q| q.id != id);
                if self.pointers.len() < 10 {
                    self.pointers.push(Pointer { id, start: p, pos: p });
                }
                self.downs.push(p);
            }
            PEvent::Move { id, x, y } => {
                if let Some(q) = self.pointers.iter_mut().find(|q| q.id == id) {
                    q.pos = V2 { x, y };
                }
            }
            PEvent::Up { id, x, y } => {
                if let Some(i) = self.pointers.iter().position(|q| q.id == id) {
                    let q = self.pointers.remove(i);
                    let end = V2 { x, y };
                    if q.start.dist(end) < TAP_SLOP {
                        self.taps.push((q.start, end));
                    }
                }
            }
            PEvent::Cancel { id } => self.pointers.retain(|q| q.id != id),
        }
    }

    pub fn key(&mut self, code: &str, down: bool) {
        if down {
            if !self.held.iter().any(|k| k == code) {
                self.held.push(code.to_string());
                self.pressed.push(code.to_string());
            }
            self.touch_mode = false;
        } else {
            self.held.retain(|k| k != code);
        }
    }

    pub fn held(&self, code: &str) -> bool {
        self.held.iter().any(|k| k == code)
    }
    pub fn pressed(&self, code: &str) -> bool {
        self.pressed.iter().any(|k| k == code)
    }
    pub fn any_pressed(&self, codes: &[&str]) -> bool {
        codes.iter().any(|c| self.pressed(c))
    }

    /// Movement from the keyboard (WASD or arrows).
    pub fn key_move(&self) -> V2 {
        let mut v = V2::default();
        if self.held("KeyW") || self.held("ArrowUp") {
            v.y -= 1.0;
        }
        if self.held("KeyS") || self.held("ArrowDown") {
            v.y += 1.0;
        }
        if self.held("KeyA") || self.held("ArrowLeft") {
            v.x -= 1.0;
        }
        if self.held("KeyD") || self.held("ArrowRight") {
            v.x += 1.0;
        }
        v.norm()
    }

    /// Forget what was held, e.g. when the page loses focus.
    pub fn release_all(&mut self) {
        self.held.clear();
        self.pointers.clear();
    }

    pub fn end_frame(&mut self) {
        self.downs.clear();
        self.taps.clear();
        self.pressed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taps_and_drags() {
        let mut i = Input::default();
        i.pointer(PEvent::Down { id: 1, x: 10.0, y: 10.0, touch: true });
        i.pointer(PEvent::Up { id: 1, x: 14.0, y: 12.0 });
        assert_eq!(i.taps.len(), 1);
        assert!(i.touch_mode);
        i.end_frame();
        i.pointer(PEvent::Down { id: 2, x: 10.0, y: 10.0, touch: true });
        i.pointer(PEvent::Move { id: 2, x: 90.0, y: 10.0 });
        assert_eq!(i.pointers.first().map(|p| p.pos.x), Some(90.0));
        i.pointer(PEvent::Up { id: 2, x: 90.0, y: 10.0 });
        assert!(i.taps.is_empty(), "a drag is not a tap");
        i.pointer(PEvent::Up { id: 99, x: 0.0, y: 0.0 });
    }

    #[test]
    fn keys_press_once_and_hold() {
        let mut i = Input::default();
        i.key("KeyD", true);
        i.key("KeyD", true);
        assert!(i.pressed("KeyD"));
        assert_eq!(i.key_move().x, 1.0);
        i.end_frame();
        assert!(!i.pressed("KeyD") && i.held("KeyD"));
        i.key("KeyD", false);
        assert_eq!(i.key_move(), V2::default());
    }
}

//! The whole game: screens, the on-screen controls, and the flow between them.

use std::f32::consts::{PI, TAU};

use crate::content::{self, CHAPTERS, Line, UPGRADES, ui};
use crate::draw::{Align, Canvas, Draw, Measure, wrap};
use crate::geom::{Rect, V2, rect, v2};
use crate::input::{Input, PEvent};
use crate::map::{LAKE, Map};
use crate::meta::{Earned, Profile};
use crate::raid::{Context, Controls, Mission, Outcome, Raid, RaidSetup, Report};
use crate::render::{Cam, dog, draw_map, draw_raid};

enum Scene {
    Title { confirm: bool },
    Story { lines: &'static [Line], i: usize },
    Hideout,
    Brief(Mission),
    Shop,
    Dogs,
    Raid { paused: bool },
    Result,
}

struct ResultInfo {
    report: Report,
    earned: Earned,
}

pub struct Game {
    pub map: Map,
    pub profile: Profile,
    scene: Scene,
    raid: Option<Raid>,
    input: Input,
    t: f32,
    save_due: bool,
    /// Safe-area insets: top, right, bottom, left.
    insets: [f32; 4],
    result: Option<ResultInfo>,
    toast: Option<(String, f32)>,
}

#[derive(Clone, Copy, PartialEq)]
enum Btn {
    Primary,
    Secondary,
    Disabled,
}

/// Where the raid's on-screen controls and readouts go for a screen size.
pub struct Hud {
    pub u: f32,
    pub attack: (V2, f32),
    pub throw: (V2, f32),
    pub roll: (V2, f32),
    pub context: (V2, f32),
    pub pause: (V2, f32),
    pub minimap: Rect,
    pub joy_home: V2,
    pub joy_r: f32,
    pub top: f32,
    pub left: f32,
}

impl Hud {
    pub fn new(w: f32, h: f32, ins: [f32; 4]) -> Hud {
        let [top, right, bottom, left] = ins;
        let u = (w.min(h) * 0.115).clamp(44.0, 72.0);
        let (rx, by) = (w - right, h - bottom);
        let mm = (w.min(h) * 0.27).clamp(92.0, 176.0);
        let minimap = rect(rx - 10.0 - mm, top + 10.0, mm, mm * 0.8);
        Hud {
            u,
            attack: (v2(rx - 1.3 * u, by - 1.35 * u), 0.72 * u),
            throw: (v2(rx - 2.95 * u, by - 0.85 * u), 0.5 * u),
            roll: (v2(rx - 1.0 * u, by - 3.0 * u), 0.48 * u),
            context: (v2(rx - 2.8 * u, by - 2.5 * u), 0.62 * u),
            pause: (v2(minimap.x - 12.0 - 0.36 * u, top + 10.0 + 0.36 * u), 0.36 * u),
            minimap,
            joy_home: v2(left + 1.7 * u, by - 1.7 * u),
            joy_r: 1.1 * u,
            top: top + 10.0,
            left: left + 10.0,
        }
    }
    fn on_button(&self, p: V2) -> bool {
        [self.attack, self.throw, self.roll, self.context, self.pause].iter().any(|(c, r)| c.dist(p) < r * 1.25) || self.minimap.contains(p)
    }
    pub fn in_joystick_zone(&self, p: V2, w: f32, h: f32) -> bool {
        p.x < w * 0.5 && p.y > h * 0.3 && !self.on_button(p)
    }
}

fn hit(c: (V2, f32), p: V2) -> bool {
    c.0.dist(p) < c.1 * 1.25
}

fn fmt_time(s: f32) -> String {
    let s = s.max(0.0) as u32;
    format!("{:02}:{:02}", s / 60, s % 60)
}

impl Game {
    pub fn new(save: Option<&str>) -> Game {
        let profile = save.and_then(Profile::load).unwrap_or_default();
        Game {
            map: Map::hoan_kiem(),
            profile,
            scene: Scene::Title { confirm: false },
            raid: None,
            input: Input::default(),
            t: 0.0,
            save_due: false,
            insets: [0.0; 4],
            result: None,
            toast: None,
        }
    }

    pub fn pointer(&mut self, e: PEvent) {
        self.input.pointer(e);
    }
    pub fn key(&mut self, code: &str, down: bool) {
        self.input.key(code, down);
    }
    pub fn set_insets(&mut self, insets: [f32; 4]) {
        self.insets = insets.map(|v| if v.is_finite() { v.clamp(0.0, 200.0) } else { 0.0 });
    }
    /// The page lost focus: let go of everything and pause a raid.
    pub fn blur(&mut self) {
        self.input.release_all();
        if let Scene::Raid { paused } = &mut self.scene {
            *paused = true;
        }
    }
    /// The save to write, when something changed.
    pub fn take_save(&mut self) -> Option<String> {
        if self.save_due {
            self.save_due = false;
            Some(self.profile.to_json())
        } else {
            None
        }
    }
    pub fn in_raid(&self) -> bool {
        matches!(self.scene, Scene::Raid { paused: false })
    }

    /// Runs one frame and returns what to draw.
    pub fn frame(&mut self, dt: f32, w: f32, h: f32, m: &dyn Measure) -> Vec<Draw> {
        let dt = if dt.is_finite() { dt.clamp(0.0, 0.1) } else { 0.0 };
        let (w, h) = (w.max(240.0), h.max(240.0));
        self.t += dt;
        if let Some((_, t)) = &mut self.toast {
            *t -= dt;
        }
        if self.toast.as_ref().is_some_and(|(_, t)| *t <= 0.0) {
            self.toast = None;
        }
        let mut cv = Canvas::default();
        match self.scene {
            Scene::Title { confirm } => self.title(&mut cv, w, h, m, confirm),
            Scene::Story { lines, i } => self.story(&mut cv, w, h, m, lines, i),
            Scene::Hideout => self.hideout(&mut cv, w, h, m),
            Scene::Brief(mission) => self.brief(&mut cv, w, h, m, mission),
            Scene::Shop => self.shop(&mut cv, w, h, m),
            Scene::Dogs => self.dogs(&mut cv, w, h, m),
            Scene::Raid { paused } => self.raid_frame(&mut cv, w, h, m, dt, paused),
            Scene::Result => self.result_screen(&mut cv, w, h, m),
        }
        if let Some((msg, t)) = &self.toast {
            let size = 16.0;
            let tw = m.text_width(msg, size, true) + 32.0;
            let y = h - self.insets[2] - 90.0;
            cv.rrect(w / 2.0 - tw / 2.0, y - 20.0, tw, 40.0, 20.0, 0x111827, 0.92 * t.min(1.0));
            cv.text(w / 2.0, y, msg.clone(), size, 0xfde68a, Align::Center, true);
        }
        self.input.end_frame();
        cv.list
    }

    // ---------------------------------------------------------------------------------------
    // Building blocks

    /// A centred column for menus: x and width.
    fn column(&self, w: f32) -> (f32, f32) {
        let [_, right, _, left] = self.insets;
        let avail = w - left - right - 32.0;
        let cw = avail.min(520.0);
        (left + 16.0 + (avail - cw) / 2.0, cw)
    }

    fn tapped(&self, r: Rect) -> bool {
        self.input.taps.iter().any(|(a, b)| r.contains(*a) && r.contains(*b))
    }

    fn button(&self, cv: &mut Canvas, r: Rect, label: &str, kind: Btn) -> bool {
        let (bg, fg) = match kind {
            Btn::Primary => (0xf59e0b, 0x1c1206),
            Btn::Secondary => (0x2f3542, 0xf3f4f6),
            Btn::Disabled => (0x22262e, 0x6b7280),
        };
        cv.rrect(r.x, r.y + 3.0, r.w, r.h, 12.0, 0x000000, 0.3);
        cv.rrect(r.x, r.y, r.w, r.h, 12.0, bg, 1.0);
        let size = (r.h * 0.36).clamp(14.0, 19.0);
        cv.text(r.x + r.w / 2.0, r.y + r.h / 2.0, label, size, fg, Align::Center, true);
        kind != Btn::Disabled && self.tapped(r)
    }

    fn dim_map(&self, cv: &mut Canvas, w: f32, h: f32, center: V2, alpha: f32) {
        let cam = Cam::zoomed(center, w, h, 0.62);
        draw_map(cv, &self.map, &cam, self.t, &[], false);
        cv.rect(0.0, 0.0, w, h, 0x0b0c10, alpha);
    }

    fn panel(&self, cv: &mut Canvas, r: Rect) {
        cv.rrect(r.x, r.y, r.w, r.h, 16.0, 0x12151c, 0.94);
    }

    fn texts(&self, cv: &mut Canvas, m: &dyn Measure, x: f32, mut y: f32, max_w: f32, s: &str, size: f32, c: u32, bold: bool) -> f32 {
        for line in wrap(m, s, size, bold, max_w) {
            cv.text(x, y, line, size, c, Align::Left, bold);
            y += size * 1.4;
        }
        y
    }

    // ---------------------------------------------------------------------------------------
    // Screens

    fn title(&mut self, cv: &mut Canvas, w: f32, h: f32, _m: &dyn Measure, confirm: bool) {
        let a = self.t * 0.04;
        self.dim_map(cv, w, h, LAKE + v2(a.cos() * 5.0, 2.0 + a.sin() * 4.0), 0.4);
        let size = (w * 0.11).clamp(34.0, 72.0);
        let ty = h * 0.24 + self.insets[0];
        cv.label(w / 2.0, ty, content::TITLE, size, 0xfbbf24, 1.0, true);
        cv.label(w / 2.0, ty + size * 0.85, content::SUBTITLE, (size * 0.38).clamp(15.0, 26.0), 0xffffff, 0.95, false);
        let ppt = (size * 0.9).clamp(36.0, 64.0);
        let cam = Cam { c: v2(w / 2.0 / ppt, h / 2.0 / ppt), ppt, w, h };
        dog(cv, &cam, v2(w / 2.0 / ppt, (ty + size * 1.9) / ppt), PI / 2.0 + (self.t * 1.3).sin() * 0.3, crate::content::PUPPY_BREED, self.t * 12.0, true);

        let (x, cw) = self.column(w);
        let bh = 54.0;
        let mut y = h - self.insets[2] - 40.0 - bh;
        if confirm {
            let pr = rect(x, h * 0.42, cw, 200.0);
            self.panel(cv, pr);
            cv.text(w / 2.0, pr.y + 40.0, ui::CONFIRM_NEW, 18.0, 0xffffff, Align::Center, true);
            if self.button(cv, rect(x + 16.0, pr.y + 76.0, cw - 32.0, bh), ui::YES, Btn::Primary) {
                self.profile = Profile::default();
                self.profile.seen_intro = true;
                self.save_due = true;
                self.scene = Scene::Story { lines: content::INTRO, i: 0 };
            } else if self.button(cv, rect(x + 16.0, pr.y + 76.0 + bh + 10.0, cw - 32.0, bh), ui::NO, Btn::Secondary) {
                self.scene = Scene::Title { confirm: false };
            }
            return;
        }
        let progress = self.profile.has_progress();
        if progress {
            if self.button(cv, rect(x, y - bh - 12.0, cw, bh), ui::CONTINUE, Btn::Primary) || self.input.any_pressed(&["Enter", "Space"]) {
                self.scene = Scene::Hideout;
            } else if self.button(cv, rect(x, y, cw, bh), ui::NEW_GAME, Btn::Secondary) {
                self.scene = Scene::Title { confirm: true };
            }
        } else if self.button(cv, rect(x, y, cw, bh), ui::PLAY, Btn::Primary) || self.input.any_pressed(&["Enter", "Space"]) {
            self.profile.seen_intro = true;
            self.save_due = true;
            self.scene = Scene::Story { lines: content::INTRO, i: 0 };
        }
        y += bh + 18.0;
        cv.text(w / 2.0, y, "Làm bằng Rust · chạy ngay trên trình duyệt", 12.0, 0x9ca3af, Align::Center, false);
    }

    fn story(&mut self, cv: &mut Canvas, w: f32, h: f32, m: &dyn Measure, lines: &'static [Line], i: usize) {
        self.dim_map(cv, w, h, v2(30.0 + self.t * 0.3, 16.0), 0.45);
        let [top, right, bottom, _] = self.insets;
        let skip = rect(w - right - 110.0, top + 12.0, 96.0, 40.0);
        if self.button(cv, skip, ui::SKIP, Btn::Secondary) || self.input.pressed("Escape") {
            self.scene = Scene::Hideout;
            return;
        }
        let Some((who, text)) = lines.get(i) else {
            self.scene = Scene::Hideout;
            return;
        };
        let (x, cw) = self.column(w);
        let size = if w < 500.0 { 17.0 } else { 19.0 };
        let wrapped = wrap(m, text, size, false, cw - 40.0);
        let box_h = 70.0 + wrapped.len() as f32 * size * 1.45;
        let r = rect(x, h - bottom - 24.0 - box_h, cw, box_h);
        self.panel(cv, r);
        let mut y = r.y + 28.0;
        if !who.is_empty() {
            cv.text(r.x + 20.0, y, *who, 16.0, 0xfbbf24, Align::Left, true);
            y += 28.0;
        }
        for l in wrapped {
            cv.text(r.x + 20.0, y, l, size, 0xf3f4f6, Align::Left, false);
            y += size * 1.45;
        }
        cv.text(r.x + r.w - 16.0, r.y + r.h - 16.0, format!("{} · {}/{}", ui::TAP, i + 1, lines.len()), 12.0, 0x9ca3af, Align::Right, false);
        let tapped_elsewhere = self.input.taps.iter().any(|(_, b)| !skip.contains(*b));
        if tapped_elsewhere || self.input.any_pressed(&["Enter", "Space", "KeyE"]) {
            self.scene = if i + 1 < lines.len() { Scene::Story { lines, i: i + 1 } } else { Scene::Hideout };
        }
    }

    fn hideout(&mut self, cv: &mut Canvas, w: f32, h: f32, m: &dyn Measure) {
        let [top, right, bottom, left] = self.insets;
        // The room: wooden floor, a window onto the old quarter, a bed and a desk.
        cv.rect(0.0, 0.0, w, h, 0x5a3e2b, 1.0);
        let mut y = 0.0;
        while y < h {
            cv.line(0.0, y, w, y, 1.0, 0x4a3223, 1.0);
            y += 26.0;
        }
        let wall_h = (h * 0.24).max(120.0);
        cv.rect(0.0, 0.0, w, wall_h, 0x334155, 1.0);
        let win = rect(w * 0.5 - (w * 0.22).min(160.0), top + 16.0, (w * 0.44).min(320.0), wall_h - top - 32.0);
        cv.rect(win.x, win.y, win.w, win.h, 0x0f1b33, 1.0);
        cv.circle(win.x + win.w * 0.8, win.y + win.h * 0.3, 14.0, 0xfef3c7, 1.0);
        for i in 0..8 {
            let bx = win.x + i as f32 * win.w / 8.0;
            let bh = win.h * (0.3 + 0.08 * ((i * 7) % 5) as f32);
            cv.rect(bx + 2.0, win.y + win.h - bh, win.w / 8.0 - 4.0, bh, 0x1e293b, 1.0);
            if i % 2 == 0 {
                cv.rect(bx + 8.0, win.y + win.h - bh + 10.0, 5.0, 5.0, 0xfbbf24, 0.9);
            }
        }
        cv.line(win.x + win.w / 2.0, win.y, win.x + win.w / 2.0, win.y + win.h, 3.0, 0x64748b, 1.0);
        cv.rect(left + 12.0, wall_h + 10.0, (w * 0.28).min(180.0), 70.0, 0x9ca3af, 1.0);
        cv.rect(left + 16.0, wall_h + 14.0, 40.0, 62.0, 0xe5e7eb, 1.0);
        cv.rect(w - right - 12.0 - (w * 0.22).min(140.0), wall_h + 10.0, (w * 0.22).min(140.0), 46.0, 0x7c5a3a, 1.0);
        if !self.profile.dogs.iter().any(|d| d.puppy) {
            // Bơ's basket, waiting.
            let (bx, by) = (w - right - 70.0, wall_h + 110.0);
            cv.circle(bx, by, 30.0, 0x8b5a2b, 1.0);
            cv.circle(bx, by, 22.0, 0xc2410c, 1.0);
            cv.label(bx, by + 44.0, format!("Ổ của {} (trống)", content::PUPPY), 12.0, 0xfecdd3, 0.9, false);
        }

        let title = if self.profile.story_done() { ui::RESCUE_STATION } else { ui::HIDEOUT };
        cv.label(left + 16.0 + m.text_width(title, 20.0, true) / 2.0, top + 26.0, title, 20.0, 0xffffff, 1.0, true);
        let money = format!("{}k", self.profile.money);
        cv.rrect(w - right - 20.0 - m.text_width(&money, 18.0, true) - 20.0, top + 10.0, m.text_width(&money, 18.0, true) + 24.0, 34.0, 17.0, 0x111827, 0.85);
        cv.text(w - right - 28.0, top + 27.0, money, 18.0, 0xfde68a, Align::Right, true);

        // Buttons from the bottom up.
        let (x, cw) = self.column(w);
        let bh = 52.0;
        let gap = 10.0;
        let mut y = h - bottom - 20.0 - bh;
        let half = (cw - gap) / 2.0;
        let dogs_label = format!("{} ({})", ui::DOGS, self.profile.dogs.len());
        if self.button(cv, rect(x, y, half, bh), ui::SHOP, Btn::Secondary) {
            self.scene = Scene::Shop;
        }
        if self.button(cv, rect(x + half + gap, y, half, bh), &dogs_label, Btn::Secondary) {
            self.scene = Scene::Dogs;
        }
        y -= bh + gap;
        if self.profile.free_unlocked() {
            let label = format!("{} · cấp {}", content::FREE_RAID_TITLE, self.profile.free_level);
            let kind = if self.profile.story_done() { Btn::Primary } else { Btn::Secondary };
            if self.button(cv, rect(x, y, cw, bh), &label, kind) {
                self.scene = Scene::Brief(Mission::Free(self.profile.free_level));
            }
            y -= bh + gap;
        }
        let rooms_top = wall_h + 90.0;
        if let Some(ch) = CHAPTERS.get(self.profile.chapter as usize) {
            let label = format!("Chương {}: {}", self.profile.chapter + 1, ch.title);
            if self.button(cv, rect(x, y, cw, bh), &label, Btn::Primary) || self.input.any_pressed(&["Enter", "Space"]) {
                self.scene = Scene::Brief(Mission::Story(self.profile.chapter as usize));
            }
            y -= bh + gap;
        }
        let menu = rect(left + 12.0, h - bottom - 20.0 - bh - (bh + gap) * 3.0 - 44.0, 44.0, 36.0);
        let _ = menu;

        // The rescued dogs live here.
        let floor = rect(left + 20.0, rooms_top, w - left - right - 40.0, (y - rooms_top).max(40.0));
        let ppt = (w.min(h) / 9.0).clamp(44.0, 80.0);
        let cam = Cam { c: v2(w / 2.0 / ppt, h / 2.0 / ppt), ppt, w, h };
        if self.profile.dogs.is_empty() {
            cv.label(w / 2.0, floor.y + floor.h / 2.0, ui::NO_DOGS_YET, 15.0, 0xfde68a, 0.9, false);
        }
        for (i, d) in self.profile.dogs.iter().enumerate().take(18) {
            let k = i as f32 * 1.7;
            let s = self.t * (0.15 + (i % 5) as f32 * 0.03);
            let px = floor.x + floor.w * (0.5 + 0.42 * (s + k).sin());
            let py = floor.y + floor.h * (0.5 + 0.38 * (s * 1.3 + k * 0.7).cos());
            let facing = (0.15 * (s + k).cos() * floor.w).atan2(-0.38 * 1.3 * (s * 1.3 + k * 0.7).sin() * floor.h) - PI / 2.0;
            let facing = if facing.is_finite() { -facing + PI } else { 0.0 };
            dog(cv, &cam, v2(px / ppt, py / ppt), facing, d.breed as usize, self.t * 10.0 + k, d.puppy);
            cv.label(px, py - 22.0, d.name.clone(), 12.0, 0xffffff, 0.85, false);
        }
        let back = rect(left + 12.0, top + 48.0, 40.0, 36.0);
        if self.button(cv, back, "≡", Btn::Secondary) {
            self.scene = Scene::Title { confirm: false };
        }
    }

    fn setup_for(&self, mission: Mission) -> RaidSetup {
        let stats = self.profile.stats();
        match mission {
            Mission::Story(ch) => RaidSetup::story(&self.map, ch, stats, self.profile.seed),
            Mission::Free(l) => RaidSetup::free(&self.map, l, stats, self.profile.seed),
        }
    }

    /// A reason the mission can't start yet.
    fn blocker(&self, mission: Mission) -> Option<String> {
        let setup = self.setup_for(mission);
        if setup.need_dogs as usize > self.profile.stats().leash {
            let price = self.profile.next_price(content::UpgradeKind::Leash).unwrap_or(0);
            return Some(format!("Cần mua Dây xích ở cửa hàng ({price}k) để dắt {} chú chó một lúc.", setup.need_dogs));
        }
        None
    }

    fn brief(&mut self, cv: &mut Canvas, w: f32, h: f32, m: &dyn Measure, mission: Mission) {
        self.dim_map(cv, w, h, v2(50.0, 30.0), 0.6);
        let setup = self.setup_for(mission);
        let (x, cw) = self.column(w);
        let [top, _, bottom, _] = self.insets;
        let (title, place, brief, tip) = match mission {
            Mission::Story(ch) => {
                let c = CHAPTERS.get(ch);
                (setup.title(), c.map_or("", |c| c.place).to_string(), c.map_or("", |c| c.brief), c.map_or("", |c| c.tip))
            }
            Mission::Free(_) => {
                (setup.title(), "Quận Hoàn Kiếm".to_string(), content::FREE_RAID_BRIEF, "Bị tóm hoặc hết giờ là mất hết đồ trong balo. Thoát được mới tính.")
            }
        };
        let r = rect(x, top + 16.0, cw, h - top - bottom - 32.0);
        self.panel(cv, r);
        let ix = r.x + 20.0;
        let iw = r.w - 40.0;
        let mut y = r.y + 34.0;
        y = self.texts(cv, m, ix, y, iw, &title, 21.0, 0xfbbf24, true) + 4.0;
        cv.text(ix, y, place, 14.0, 0x9ca3af, Align::Left, false);
        y += 30.0;
        y = self.texts(cv, m, ix, y, iw, brief, 16.0, 0xf3f4f6, false) + 8.0;
        y = self.texts(cv, m, ix, y, iw, &format!("Mẹo: {tip}"), 14.0, 0xa7f3d0, false) + 8.0;
        let exits: Vec<&str> = self.map.exits.iter().filter(|e| setup.exits.contains(&e.id)).map(|e| e.name).collect();
        y = self.texts(cv, m, ix, y, iw, &format!("{}: {}", ui::EXITS, exits.join(", ")), 14.0, 0x93c5fd, false) + 4.0;
        let d = setup.diff;
        let _ = self.texts(
            cv,
            m,
            ix,
            y,
            iw,
            &format!("Bọn trộm: {} · Thời gian: {}", d.thieves + setup.boss.map_or(0, |_| 1), fmt_time(d.time_s)),
            14.0,
            0xfca5a5,
            false,
        );

        let bh = 52.0;
        let by = r.y + r.h - 20.0 - bh;
        let blocked = self.blocker(mission);
        if let Some(msg) = &blocked {
            let _ = self.texts(cv, m, ix, by - 80.0, iw, msg, 14.0, 0xfca5a5, true);
            if self.button(cv, rect(ix, by - bh - 10.0, iw, bh), ui::SHOP, Btn::Primary) {
                self.scene = Scene::Shop;
                return;
            }
        }
        let half = (iw - 10.0) / 2.0;
        if self.button(cv, rect(ix, by, half, bh), ui::BACK, Btn::Secondary) || self.input.pressed("Escape") {
            self.scene = Scene::Hideout;
            return;
        }
        let go_kind = if blocked.is_some() { Btn::Disabled } else { Btn::Primary };
        if self.button(cv, rect(ix + half + 10.0, by, half, bh), ui::GO, go_kind) || (blocked.is_none() && self.input.any_pressed(&["Enter", "Space"])) {
            let _ = self.profile.next_seed();
            self.save_due = true;
            let setup = self.setup_for(mission);
            self.raid = Some(Raid::new(&self.map, setup));
            self.scene = Scene::Raid { paused: false };
        }
    }

    fn shop(&mut self, cv: &mut Canvas, w: f32, h: f32, m: &dyn Measure) {
        self.dim_map(cv, w, h, v2(40.0, 20.0), 0.7);
        let [top, _, bottom, _] = self.insets;
        let (x, cw) = self.column(w);
        let wide = w > 760.0;
        let (x, cw) = if wide { (x - 160.0, cw + 320.0) } else { (x, cw) };
        cv.text(x, top + 30.0, format!("{} · {}", ui::SHOP, content::SHOPKEEPER), 20.0, 0xfbbf24, Align::Left, true);
        cv.text(x + cw, top + 30.0, format!("{}k", self.profile.money), 18.0, 0xfde68a, Align::Right, true);
        let cols = if wide { 2 } else { 1 };
        let gap = 10.0;
        let rw = (cw - gap * (cols as f32 - 1.0)) / cols as f32;
        let avail = h - top - bottom - 60.0 - 80.0;
        let rows = UPGRADES.len().div_ceil(cols);
        let rh = (avail / rows as f32 - gap).clamp(58.0, 84.0);
        let mut bought = None;
        for (i, up) in UPGRADES.iter().enumerate() {
            let (cx, cy) = ((i % cols) as f32, (i / cols) as f32);
            let r = rect(x + cx * (rw + gap), top + 56.0 + cy * (rh + gap), rw, rh);
            self.panel(cv, r);
            let lvl = self.profile.upgrades.level(up.kind);
            let max = up.prices.len();
            cv.text(r.x + 14.0, r.y + 22.0, format!("{}  {}/{}", up.name, lvl, max), 16.0, 0xffffff, Align::Left, true);
            let what_w = r.w - 130.0;
            let mut ty = r.y + 44.0;
            for l in wrap(m, up.what, 13.0, false, what_w).into_iter().take(2) {
                cv.text(r.x + 14.0, ty, l, 13.0, 0x9ca3af, Align::Left, false);
                ty += 17.0;
            }
            let br = rect(r.x + r.w - 104.0, r.y + (rh - 42.0) / 2.0, 92.0, 42.0);
            match self.profile.next_price(up.kind) {
                Some(p) => {
                    let kind = if self.profile.money >= p { Btn::Primary } else { Btn::Disabled };
                    if self.button(cv, br, &format!("{p}k"), kind) {
                        bought = Some((up.kind, up.name));
                    }
                }
                None => {
                    let _ = self.button(cv, br, ui::MAXED, Btn::Disabled);
                }
            }
        }
        if let Some((k, name)) = bought
            && self.profile.buy(k)
        {
            self.save_due = true;
            self.toast = Some((format!("Đã mua {name}"), 2.0));
        }
        let (bx, bw) = self.column(w);
        if self.button(cv, rect(bx, h - bottom - 70.0, bw, 52.0), ui::BACK, Btn::Secondary) || self.input.pressed("Escape") {
            self.scene = Scene::Hideout;
        }
    }

    fn dogs(&mut self, cv: &mut Canvas, w: f32, h: f32, _m: &dyn Measure) {
        cv.rect(0.0, 0.0, w, h, 0x1a1d24, 1.0);
        let [top, right, bottom, left] = self.insets;
        cv.text(left + 16.0, top + 30.0, format!("{} · {} chú", ui::DOGS, self.profile.dogs.len()), 20.0, 0xfbbf24, Align::Left, true);
        let card = 112.0;
        let gap = 10.0;
        let avail_w = w - left - right - 32.0;
        let cols = ((avail_w + gap) / (card + gap)).floor().max(1.0) as usize;
        let rows = ((h - top - bottom - 150.0 + gap) / (card + gap)).floor().max(1.0) as usize;
        let ox = left + 16.0 + (avail_w - (cols as f32 * (card + gap) - gap)) / 2.0;
        let ppt = 56.0;
        let cam = Cam { c: v2(w / 2.0 / ppt, h / 2.0 / ppt), ppt, w, h };
        let n = self.profile.dogs.len();
        let shown = (cols * rows).min(n);
        if n == 0 {
            cv.text(w / 2.0, h / 2.0, ui::NO_DOGS_YET, 16.0, 0x9ca3af, Align::Center, false);
        }
        for (i, d) in self.profile.dogs.iter().rev().take(shown).enumerate() {
            let x = ox + (i % cols) as f32 * (card + gap);
            let y = top + 56.0 + (i / cols) as f32 * (card + gap);
            cv.rrect(x, y, card, card, 14.0, if d.puppy { 0x3b1d2a } else { 0x262b35 }, 1.0);
            dog(cv, &cam, v2((x + card / 2.0) / ppt, (y + 44.0) / ppt), PI / 2.0, d.breed as usize, self.t * 6.0 + i as f32, d.puppy);
            cv.text(x + card / 2.0, y + 82.0, d.name.clone(), 15.0, 0xffffff, Align::Center, true);
            let breed = content::BREEDS.get(d.breed as usize).map_or("", |b| b.name);
            cv.text(x + card / 2.0, y + 100.0, breed, 11.0, 0x9ca3af, Align::Center, false);
        }
        if n > shown {
            cv.text(w / 2.0, h - bottom - 92.0, format!("… và {} chú nữa", n - shown), 14.0, 0x9ca3af, Align::Center, false);
        }
        let (bx, bw) = self.column(w);
        if self.button(cv, rect(bx, h - bottom - 70.0, bw, 52.0), ui::BACK, Btn::Secondary) || self.input.pressed("Escape") {
            self.scene = Scene::Hideout;
        }
    }

    fn raid_controls(&self, hud: &Hud, w: f32, h: f32) -> (Controls, Option<(V2, V2)>, bool) {
        let i = &self.input;
        let mut c = Controls { mv: i.key_move(), sneak: i.held("ShiftLeft") || i.held("ShiftRight"), ..Default::default() };
        let mut joy = None;
        for p in &i.pointers {
            if joy.is_none() && hud.in_joystick_zone(p.start, w, h) {
                let v = (p.pos - p.start) * (1.0 / hud.joy_r);
                c.mv = v.clamp_len(1.0);
                joy = Some((p.start, p.start + v.clamp_len(1.0) * hud.joy_r));
            }
            if hit(hud.attack, p.start) {
                c.attack = true;
            }
            if hit(hud.context, p.start) {
                c.interact = true;
            }
        }
        for d in &i.downs {
            c.throw |= hit(hud.throw, *d);
            c.roll |= hit(hud.roll, *d);
        }
        c.attack |= i.held("KeyJ");
        c.throw |= i.pressed("KeyK");
        c.roll |= i.pressed("Space");
        c.interact |= i.held("KeyE");
        let pause = i.taps.iter().any(|(a, _)| hit(hud.pause, *a)) || i.any_pressed(&["Escape", "KeyP"]);
        (c, joy, pause)
    }

    fn raid_frame(&mut self, cv: &mut Canvas, w: f32, h: f32, m: &dyn Measure, dt: f32, paused: bool) {
        let hud = Hud::new(w, h, self.insets);
        let (controls, joy, pause_pressed) = self.raid_controls(&hud, w, h);
        if !paused {
            if let Some(r) = &mut self.raid {
                r.update(&self.map, dt, &controls);
            }
            if pause_pressed {
                self.scene = Scene::Raid { paused: true };
            }
        }
        let Some(raid) = &self.raid else {
            self.scene = Scene::Hideout;
            return;
        };
        let shake = if raid.shake > 0.0 { v2((self.t * 91.0).sin(), (self.t * 77.0).cos()) * (raid.shake * 0.25) } else { V2::default() };
        let cam = Cam::new(raid.player.pos + shake, w, h);
        draw_map(cv, &self.map, &cam, self.t, &raid.setup.exits, raid.train.warning());
        draw_raid(cv, &self.map, raid, &cam, self.t);
        self.draw_hud(cv, &hud, raid, m, w, h, joy);

        if paused {
            cv.rect(0.0, 0.0, w, h, 0x000000, 0.6);
            let (x, cw) = self.column(w);
            cv.text(w / 2.0, h * 0.3, ui::PAUSED, 28.0, 0xffffff, Align::Center, true);
            if self.button(cv, rect(x, h * 0.42, cw, 54.0), ui::RESUME, Btn::Primary) || self.input.any_pressed(&["Escape", "Enter"]) {
                self.scene = Scene::Raid { paused: false };
            } else if self.button(cv, rect(x, h * 0.42 + 66.0, cw, 54.0), ui::ABANDON, Btn::Secondary)
                && let Some(r) = &mut self.raid
            {
                r.outcome = Some(Outcome::Abandoned);
            }
        }

        let finished = self.raid.as_ref().and_then(|r| r.outcome.map(|_| (r.report(), r.setup.mission)));
        if let Some((report, mission)) = finished {
            let earned = self.profile.apply(mission, &report);
            self.save_due = true;
            self.result = Some(ResultInfo { report, earned });
            self.scene = Scene::Result;
        }
    }

    fn draw_hud(&self, cv: &mut Canvas, hud: &Hud, raid: &Raid, m: &dyn Measure, w: f32, h: f32, joy: Option<(V2, V2)>) {
        let p = &raid.player;
        let stats = raid.setup.stats;
        if raid.spotted {
            let a = 0.25 + 0.15 * (self.t * 8.0).sin();
            cv.rect(0.0, 0.0, w, 6.0, 0xff3b3b, a);
            cv.rect(0.0, h - 6.0, w, 6.0, 0xff3b3b, a);
            cv.rect(0.0, 0.0, 6.0, h, 0xff3b3b, a);
            cv.rect(w - 6.0, 0.0, 6.0, h, 0xff3b3b, a);
        }

        // Top left: health, time, what's carried.
        let (x, y) = (hud.left, hud.top);
        let bw = (w * 0.34).clamp(120.0, 220.0);
        cv.rrect(x - 4.0, y - 4.0, bw + 8.0, 66.0, 10.0, 0x0b0d12, 0.7);
        cv.rrect(x, y, bw, 12.0, 6.0, 0x3f1d1d, 1.0);
        let hp = (p.hp / stats.max_hp).clamp(0.0, 1.0);
        let hp_c = if hp > 0.5 {
            0x22c55e
        } else if hp > 0.25 {
            0xf59e0b
        } else {
            0xef4444
        };
        cv.rrect(x, y, bw * hp, 12.0, 6.0, hp_c, 1.0);
        cv.rrect(x, y + 16.0, bw * (p.stamina / 100.0).clamp(0.0, 1.0), 4.0, 2.0, 0x93c5fd, 0.9);
        let low = raid.time_left < 60.0;
        cv.text(x, y + 36.0, fmt_time(raid.time_left), 18.0, if low { 0xff6b6b } else { 0xffffff }, Align::Left, true);
        let carried = format!("Chó {}/{} · Dép {} · Balo {}/{}", raid.following(), stats.leash, p.slippers, raid.bag.len(), stats.bag);
        cv.text(x + 62.0, y + 36.0, carried, 12.0, 0xd1d5db, Align::Left, false);
        cv.text(x, y + 56.0, if p.sneaking { "Đang lén" } else { "" }, 11.0, 0xa7f3d0, Align::Left, false);

        // The objective, under it.
        let ow = (hud.pause.0.x - hud.pause.1 - 10.0 - x).max(140.0);
        let mut oy = y + 84.0;
        for l in wrap(m, &raid.objective_text(), 14.0, true, ow).into_iter().take(3) {
            cv.label(x + m.text_width(&l, 14.0, true) / 2.0, oy, l, 14.0, 0xfde68a, 1.0, true);
            oy += 20.0;
        }

        // Minimap.
        let mm = hud.minimap;
        cv.rrect(mm.x - 3.0, mm.y - 3.0, mm.w + 6.0, mm.h + 6.0, 8.0, 0x000000, 0.6);
        cv.list.push(Draw::Minimap { x: mm.x, y: mm.y, w: mm.w, h: mm.h, a: 1.0 });
        let to_mm = |q: V2| (mm.x + q.x / self.map.w as f32 * mm.w, mm.y + q.y / self.map.h as f32 * mm.h);
        for e in &self.map.exits {
            if raid.exit_open(e.id) {
                let (ex, ey) = to_mm(e.center());
                cv.circle(ex, ey, 4.5, 0x34d399, 1.0);
            }
        }
        for c in raid.cages.iter().filter(|c| !c.open) {
            let (cx, cy) = to_mm(c.pos);
            cv.circle(cx, cy, 3.5, 0xfbbf24, 0.6 + 0.4 * (self.t * 5.0).sin().abs());
        }
        for k in raid.pickups.iter().filter(|k| !k.taken && k.kind == crate::raid::PickupKind::Clue) {
            let (cx, cy) = to_mm(k.pos);
            cv.circle(cx, cy, 3.5, 0xe11d48, 1.0);
        }
        for t in raid.thieves.iter().filter(|t| matches!(t.state, crate::raid::TState::Chase)) {
            let (tx, ty) = to_mm(t.pos);
            cv.circle(tx, ty, 2.5, 0xff3b3b, 1.0);
        }
        let (px, py) = to_mm(p.pos);
        cv.circle(px, py, 3.5, 0xffffff, 1.0);
        cv.ring(px, py, 6.0, 1.5, 0xffffff, 0.6);

        // Pause.
        let (pc, pr) = hud.pause;
        cv.circle(pc.x, pc.y, pr, 0x111827, 0.8);
        cv.rect(pc.x - pr * 0.3, pc.y - pr * 0.35, pr * 0.2, pr * 0.7, 0xffffff, 1.0);
        cv.rect(pc.x + pr * 0.1, pc.y - pr * 0.35, pr * 0.2, pr * 0.7, 0xffffff, 1.0);

        // Banner and getting out.
        if let Some((msg, t)) = &raid.banner {
            let a = t.min(1.0);
            cv.rect(0.0, h * 0.3 - 26.0, w, 52.0, 0x000000, 0.55 * a);
            cv.text(w / 2.0, h * 0.3, msg.clone(), 18.0, 0xfde68a, Align::Center, true);
        }
        if p.extract_t > 0.0 {
            let (cx, cy) = (w / 2.0, h * 0.42);
            cv.circle(cx, cy, 42.0, 0x000000, 0.5);
            cv.arc(cx, cy, 36.0, -PI / 2.0, -PI / 2.0 + TAU * (p.extract_t / crate::raid::EXTRACT_S), 6.0, 0x34d399, 1.0);
            cv.text(cx, cy + 62.0, ui::EXTRACTING, 16.0, 0xa7f3d0, Align::Center, true);
            if !raid.objective_met() {
                cv.text(cx, cy + 84.0, ui::RETREAT, 13.0, 0xfca5a5, Align::Center, false);
            }
        }

        // Controls.
        let ctx_label = match raid.context {
            Context::Cage { .. } => Some((ui::OPEN_CAGE, true)),
            Context::Tie { .. } => Some((ui::TIE, true)),
            Context::LeashFull => Some((ui::LEASH_FULL, false)),
            Context::None => None,
        };
        if self.input.touch_mode {
            let (home, knob) = joy.unwrap_or((hud.joy_home, hud.joy_home));
            cv.circle(home.x, home.y, hud.joy_r, 0xffffff, if joy.is_some() { 0.12 } else { 0.06 });
            cv.ring(home.x, home.y, hud.joy_r, 2.0, 0xffffff, 0.25);
            cv.circle(knob.x, knob.y, hud.u * 0.42, 0xffffff, if joy.is_some() { 0.45 } else { 0.2 });
            let button = |cv: &mut Canvas, (c, r): (V2, f32), label: &str, col: u32, a: f32| {
                cv.circle(c.x, c.y, r, 0x000000, 0.35 * a);
                cv.circle(c.x, c.y, r * 0.92, col, 0.75 * a);
                cv.text(c.x, c.y, label, (r * 0.42).clamp(12.0, 18.0), 0xffffff, Align::Center, true);
            };
            button(cv, hud.attack, ui::ATTACK, 0xdc2626, 1.0);
            button(cv, hud.throw, &format!("{} {}", ui::THROW, p.slippers), 0xdb2777, if p.slippers > 0 { 1.0 } else { 0.4 });
            button(cv, hud.roll, ui::ROLL, 0x2563eb, if p.stamina >= 30.0 { 1.0 } else { 0.4 });
            if let Some((label, ok)) = ctx_label {
                button(cv, hud.context, label, if ok { 0xd97706 } else { 0x4b5563 }, 1.0);
                let progress = match raid.context {
                    Context::Cage { progress, .. } | Context::Tie { progress, .. } => progress,
                    _ => 0.0,
                };
                if progress > 0.0 {
                    let (c, r) = hud.context;
                    cv.arc(c.x, c.y, r * 1.08, -PI / 2.0, -PI / 2.0 + TAU * progress.min(1.0), 5.0, 0xfde68a, 1.0);
                }
            }
        } else {
            if let Some((label, ok)) = ctx_label {
                let s = if ok { format!("Giữ E: {label}") } else { label.to_string() };
                cv.label(w / 2.0, h * 0.62, s, 16.0, 0xfde68a, 1.0, true);
            }
            cv.label(w / 2.0, h - self.insets[2] - 18.0, ui::KEYS, 12.0, 0xd1d5db, 0.8, false);
        }
    }

    fn result_screen(&mut self, cv: &mut Canvas, w: f32, h: f32, m: &dyn Measure) {
        let center = self.raid.as_ref().map_or(LAKE, |r| r.player.pos);
        self.dim_map(cv, w, h, center, 0.65);
        let Some(info) = &self.result else {
            self.scene = Scene::Hideout;
            return;
        };
        let (x, cw) = self.column(w);
        let [top, _, bottom, _] = self.insets;
        let r = rect(x, top + 24.0, cw, h - top - bottom - 48.0);
        self.panel(cv, r);
        let (title, color) = match info.report.outcome {
            Outcome::Escaped(_) => (ui::SUCCESS, 0x34d399),
            Outcome::Caught | Outcome::Abandoned => (ui::CAUGHT, 0xef4444),
            Outcome::TimeUp => (ui::TIMEOUT, 0xf59e0b),
        };
        let ix = r.x + 22.0;
        let iw = r.w - 44.0;
        cv.text(r.x + r.w / 2.0, r.y + 44.0, title, 26.0, color, Align::Center, true);
        let mut y = r.y + 90.0;
        let mut line = |cv: &mut Canvas, s: String, c: u32| {
            y = self.texts(cv, m, ix, y, iw, &s, 16.0, c, false) + 2.0;
        };
        if let Some(ch) = info.earned.chapter_done {
            let reward = content::REWARD_CHAPTER.get(ch).copied().unwrap_or(0);
            line(cv, format!("Hoàn thành chương {}: +{reward}k", ch + 1), 0xfde68a);
        }
        if !info.earned.new_dogs.is_empty() {
            line(cv, format!("Đã cứu: {} (+{}k mỗi chú)", info.earned.new_dogs.join(", "), content::REWARD_PER_DOG), 0xffffff);
        }
        if info.report.cash + info.report.scrap > 0 {
            line(cv, format!("Tiền lẻ {}k · Đồng nát bán được {}k", info.report.cash, info.report.scrap), 0xd1d5db);
        }
        if info.report.tied > 0 {
            line(cv, format!("Trói được {} tên trộm", info.report.tied), 0xd1d5db);
        }
        line(cv, format!("Thời gian: {}", fmt_time(info.report.seconds)), 0x9ca3af);
        match info.report.outcome {
            Outcome::Escaped(_) if !info.report.objective_met => line(cv, "Chưa xong nhiệm vụ chương này. Đồ đã mang về vẫn được giữ.".to_string(), 0xfca5a5),
            Outcome::Escaped(_) => {}
            _ => line(cv, "Mất toàn bộ đồ trong balo. Nâng cấp ở cửa hàng rồi thử lại nhé.".to_string(), 0xfca5a5),
        }
        line(cv, format!("Tổng cộng: +{}k", info.earned.money), 0xfbbf24);

        let chapter = info.earned.chapter_done;
        if self.button(cv, rect(ix, r.y + r.h - 74.0, iw, 54.0), ui::TO_HIDEOUT, Btn::Primary) || self.input.any_pressed(&["Enter", "Space"]) {
            self.raid = None;
            self.result = None;
            self.scene = match chapter.and_then(|c| CHAPTERS.get(c)) {
                Some(c) => Scene::Story { lines: c.after, i: 0 },
                None => Scene::Hideout,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::ApproxMeasure;

    fn tap(g: &mut Game, x: f32, y: f32) {
        g.pointer(PEvent::Down { id: 1, x, y, touch: true });
        g.pointer(PEvent::Up { id: 1, x, y });
    }

    fn texts(list: &[Draw]) -> Vec<String> {
        list.iter()
            .filter_map(|d| match d {
                Draw::Text { s, .. } => Some(s.clone()),
                _ => None,
            })
            .collect()
    }

    /// Where a button with this label was drawn (the rounded rectangle just before its text).
    fn find_button(list: &[Draw], label: &str) -> Option<V2> {
        let i = list.iter().position(|d| matches!(d, Draw::Text { s, .. } if s == label))?;
        list.get(..i)?.iter().rev().find_map(|d| match d {
            Draw::RRect { x, y, w, h, .. } => Some(v2(x + w / 2.0, y + h / 2.0)),
            _ => None,
        })
    }

    fn press(g: &mut Game, w: f32, h: f32, label: &str) -> bool {
        let list = g.frame(0.016, w, h, &ApproxMeasure);
        let Some(p) = find_button(&list, label) else { return false };
        tap(g, p.x, p.y);
        g.frame(0.016, w, h, &ApproxMeasure);
        true
    }

    #[test]
    fn a_new_player_goes_from_title_through_the_story_to_the_room_and_out() {
        for (w, h) in [(390.0, 844.0), (844.0, 390.0), (1440.0, 900.0)] {
            let mut g = Game::new(None);
            assert!(press(&mut g, w, h, ui::PLAY), "{w}x{h}");
            assert!(matches!(g.scene, Scene::Story { .. }));
            for _ in 0..content::INTRO.len() {
                tap(&mut g, w / 2.0, h / 3.0);
                g.frame(0.016, w, h, &ApproxMeasure);
            }
            assert!(matches!(g.scene, Scene::Hideout));
            let label = format!("Chương 1: {}", CHAPTERS.first().map_or("", |c| c.title));
            assert!(press(&mut g, w, h, &label), "{w}x{h}: chapter button");
            assert!(matches!(g.scene, Scene::Brief(Mission::Story(0))));
            assert!(press(&mut g, w, h, ui::GO), "{w}x{h}: go");
            assert!(matches!(g.scene, Scene::Raid { paused: false }));
            // Play a few seconds with the joystick, then pause and give up.
            g.pointer(PEvent::Down { id: 5, x: w * 0.2, y: h * 0.8, touch: true });
            g.pointer(PEvent::Move { id: 5, x: w * 0.2 + 60.0, y: h * 0.8 });
            for _ in 0..90 {
                g.frame(1.0 / 30.0, w, h, &ApproxMeasure);
            }
            g.pointer(PEvent::Up { id: 5, x: w * 0.2 + 60.0, y: h * 0.8 });
            g.key("Escape", true);
            g.frame(0.016, w, h, &ApproxMeasure);
            g.key("Escape", false);
            assert!(matches!(g.scene, Scene::Raid { paused: true }));
            assert!(press(&mut g, w, h, ui::ABANDON));
            assert!(matches!(g.scene, Scene::Result));
            assert!(press(&mut g, w, h, ui::TO_HIDEOUT));
            assert!(matches!(g.scene, Scene::Hideout));
            assert_eq!(g.profile.raids, 1);
            assert!(g.take_save().is_some());
        }
    }

    #[test]
    fn chapter_three_sends_you_to_the_shop_first() {
        let mut g = Game::new(None);
        g.profile.chapter = 2;
        g.profile.money = 100;
        g.scene = Scene::Brief(Mission::Story(2));
        let list = g.frame(0.016, 390.0, 844.0, &ApproxMeasure);
        assert!(texts(&list).iter().any(|t| t.starts_with("Cần mua Dây xích")));
        assert!(press(&mut g, 390.0, 844.0, ui::SHOP));
        assert!(matches!(g.scene, Scene::Shop));
        assert!(press(&mut g, 390.0, 844.0, "60k"));
        assert_eq!(g.profile.stats().leash, 2);
        assert_eq!(g.profile.money, 40);
    }

    #[test]
    fn controls_stay_on_screen_and_apart() {
        for (w, h) in [(360.0, 640.0), (390.0, 844.0), (844.0, 390.0), (667.0, 375.0), (1440.0, 900.0)] {
            let hud = Hud::new(w, h, [0.0, 44.0, 21.0, 44.0]);
            let buttons = [hud.attack, hud.throw, hud.roll, hud.context, hud.pause];
            for (c, r) in buttons {
                assert!(c.x - r >= 0.0 && c.y - r >= 0.0 && c.x + r <= w && c.y + r <= h, "{w}x{h}: {c:?}");
            }
            for (i, a) in buttons.iter().enumerate() {
                for b in buttons.iter().skip(i + 1) {
                    assert!(a.0.dist(b.0) >= a.1 + b.1, "{w}x{h}: buttons overlap");
                }
            }
            assert!(hud.in_joystick_zone(hud.joy_home, w, h), "{w}x{h}: joystick home");
        }
    }

    #[test]
    fn a_broken_save_starts_fresh_and_bad_sizes_dont_crash() {
        let mut g = Game::new(Some("{oops"));
        assert_eq!(g.profile, Profile::default());
        g.set_insets([f32::NAN, -5.0, 1e9, 3.0]);
        for (w, h) in [(0.0, 0.0), (f32::NAN, 100.0), (100000.0, 50.0)] {
            let _ = g.frame(f32::INFINITY, w, h, &ApproxMeasure);
        }
    }
}

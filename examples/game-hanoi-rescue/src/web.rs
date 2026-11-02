//! The browser side: a full-screen canvas, pointer and keyboard events, the save in local storage,
//! and a requestAnimationFrame loop that paints each frame's shapes.

use std::cell::RefCell;
use std::f64::consts::TAU;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, KeyboardEvent, PointerEvent, Window};

use crate::draw::{Align, Draw, Measure};
use crate::game::Game;
use crate::input::PEvent;
use crate::map::{H, W};
use crate::meta::SAVE_KEY;

type FrameFn = Closure<dyn FnMut(f64)>;

const FONT: &str = "system-ui, -apple-system, 'Segoe UI', Roboto, 'Noto Sans', Arial, sans-serif";

fn font(size: f32, bold: bool) -> String {
    format!("{} {}px {}", if bold { 700 } else { 500 }, size.round(), FONT)
}

fn color(c: u32) -> String {
    format!("#{:06x}", c & 0xff_ffff)
}

struct CtxMeasure<'a>(&'a CanvasRenderingContext2d);

impl Measure for CtxMeasure<'_> {
    fn text_width(&self, s: &str, size: f32, bold: bool) -> f32 {
        self.0.set_font(&font(size, bold));
        self.0.measure_text(s).map(|m| m.width() as f32).unwrap_or(s.chars().count() as f32 * size * 0.55)
    }
}

fn paint(ctx: &CanvasRenderingContext2d, mini: &HtmlCanvasElement, list: &[Draw]) {
    for d in list {
        match d {
            Draw::Rect { x, y, w, h, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_fill_style_str(&color(*c));
                ctx.fill_rect(f64::from(*x), f64::from(*y), f64::from(*w), f64::from(*h));
            }
            Draw::RRect { x, y, w, h, r, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_fill_style_str(&color(*c));
                ctx.begin_path();
                let r = f64::from(r.min(w / 2.0).min(h / 2.0).max(0.0));
                let (x, y, w, h) = (f64::from(*x), f64::from(*y), f64::from(*w), f64::from(*h));
                ctx.move_to(x + r, y);
                let _ = ctx.arc_to(x + w, y, x + w, y + h, r);
                let _ = ctx.arc_to(x + w, y + h, x, y + h, r);
                let _ = ctx.arc_to(x, y + h, x, y, r);
                let _ = ctx.arc_to(x, y, x + w, y, r);
                ctx.close_path();
                ctx.fill();
            }
            Draw::Circle { x, y, r, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_fill_style_str(&color(*c));
                ctx.begin_path();
                let _ = ctx.arc(f64::from(*x), f64::from(*y), f64::from(r.max(0.0)), 0.0, TAU);
                ctx.fill();
            }
            Draw::Ring { x, y, r, w, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_stroke_style_str(&color(*c));
                ctx.set_line_width(f64::from(*w));
                ctx.begin_path();
                let _ = ctx.arc(f64::from(*x), f64::from(*y), f64::from(r.max(0.0)), 0.0, TAU);
                ctx.stroke();
            }
            Draw::Line { x1, y1, x2, y2, w, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_stroke_style_str(&color(*c));
                ctx.set_line_width(f64::from(*w));
                ctx.begin_path();
                ctx.move_to(f64::from(*x1), f64::from(*y1));
                ctx.line_to(f64::from(*x2), f64::from(*y2));
                ctx.stroke();
            }
            Draw::Wedge { x, y, r, a0, a1, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_fill_style_str(&color(*c));
                ctx.begin_path();
                ctx.move_to(f64::from(*x), f64::from(*y));
                let _ = ctx.arc(f64::from(*x), f64::from(*y), f64::from(r.max(0.0)), f64::from(*a0), f64::from(*a1));
                ctx.close_path();
                ctx.fill();
            }
            Draw::Arc { x, y, r, a0, a1, w, c, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_stroke_style_str(&color(*c));
                ctx.set_line_width(f64::from(*w));
                ctx.begin_path();
                let _ = ctx.arc(f64::from(*x), f64::from(*y), f64::from(r.max(0.0)), f64::from(*a0), f64::from(*a1));
                ctx.stroke();
            }
            Draw::Poly { pts, c, a } => {
                let Some((first, rest)) = pts.split_first() else { continue };
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_fill_style_str(&color(*c));
                ctx.begin_path();
                ctx.move_to(f64::from(first.0), f64::from(first.1));
                for (x, y) in rest {
                    ctx.line_to(f64::from(*x), f64::from(*y));
                }
                ctx.close_path();
                ctx.fill();
            }
            Draw::Text { x, y, s, size, c, a, align, bold, outline } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_font(&font(*size, *bold));
                ctx.set_text_align(match align {
                    Align::Left => "left",
                    Align::Center => "center",
                    Align::Right => "right",
                });
                ctx.set_text_baseline("middle");
                if *outline {
                    ctx.set_line_width(f64::from((size * 0.22).max(2.5)));
                    ctx.set_stroke_style_str("#0b0c10");
                    ctx.set_line_join("round");
                    let _ = ctx.stroke_text(s, f64::from(*x), f64::from(*y));
                }
                ctx.set_fill_style_str(&color(*c));
                let _ = ctx.fill_text(s, f64::from(*x), f64::from(*y));
            }
            Draw::Minimap { x, y, w, h, a } => {
                ctx.set_global_alpha(f64::from(*a));
                ctx.set_image_smoothing_enabled(false);
                let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(mini, f64::from(*x), f64::from(*y), f64::from(*w), f64::from(*h));
            }
        }
    }
    ctx.set_global_alpha(1.0);
}

/// Safe-area insets (notch, home bar) from a probe element padded with `env(safe-area-inset-*)`.
fn insets(window: &Window) -> [f32; 4] {
    let Some(doc) = window.document() else { return [0.0; 4] };
    let Some(el) = doc.get_element_by_id("safe") else { return [0.0; 4] };
    let Ok(Some(style)) = window.get_computed_style(&el) else { return [0.0; 4] };
    let px = |prop: &str| style.get_property_value(prop).ok().and_then(|v| v.trim_end_matches("px").trim().parse::<f32>().ok()).unwrap_or(0.0);
    [px("padding-top"), px("padding-right"), px("padding-bottom"), px("padding-left")]
}

fn listen<E: JsCast + wasm_bindgen::convert::FromWasmAbi + 'static>(
    target: &web_sys::EventTarget,
    name: &str,
    f: impl FnMut(E) + 'static,
) -> Result<(), JsValue> {
    let c = Closure::<dyn FnMut(E)>::new(f);
    target.add_event_listener_with_callback(name, c.as_ref().unchecked_ref())?;
    c.forget();
    Ok(())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = document.get_element_by_id("game").ok_or("no #game canvas")?.dyn_into()?;
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.ok_or("no 2d context")?.dyn_into()?;
    let storage = window.local_storage().ok().flatten();
    let save = storage.as_ref().and_then(|s| s.get_item(SAVE_KEY).ok().flatten());
    let game = Rc::new(RefCell::new(Game::new(save.as_deref())));

    // The minimap is drawn once from the map's colours and stretched each frame.
    let mini: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
    mini.set_width(W as u32);
    mini.set_height(H as u32);
    let mctx: CanvasRenderingContext2d = mini.get_context("2d")?.ok_or("no 2d context")?.dyn_into()?;
    let pixels = game.borrow().map.minimap_rgba();
    let img = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&pixels[..]), W as u32, H as u32)?;
    mctx.put_image_data(&img, 0.0, 0.0)?;

    let target: &web_sys::EventTarget = canvas.as_ref();
    let pos = |e: &PointerEvent| (e.client_x() as f32, e.client_y() as f32);
    {
        let g = game.clone();
        listen(target, "pointerdown", move |e: PointerEvent| {
            e.prevent_default();
            let (x, y) = pos(&e);
            let touch = e.pointer_type() != "mouse";
            if touch || e.button() == 0 {
                g.borrow_mut().pointer(PEvent::Down { id: e.pointer_id(), x, y, touch });
            }
        })?;
    }
    {
        let g = game.clone();
        listen(target, "pointermove", move |e: PointerEvent| {
            let (x, y) = pos(&e);
            g.borrow_mut().pointer(PEvent::Move { id: e.pointer_id(), x, y });
        })?;
    }
    {
        let g = game.clone();
        listen(target, "pointerup", move |e: PointerEvent| {
            let (x, y) = pos(&e);
            g.borrow_mut().pointer(PEvent::Up { id: e.pointer_id(), x, y });
        })?;
    }
    for name in ["pointercancel", "pointerleave"] {
        let g = game.clone();
        listen(target, name, move |e: PointerEvent| {
            if name == "pointercancel" || e.pointer_type() == "mouse" {
                g.borrow_mut().pointer(PEvent::Cancel { id: e.pointer_id() });
            }
        })?;
    }
    listen(target, "contextmenu", |e: web_sys::Event| e.prevent_default())?;
    let wtarget: &web_sys::EventTarget = window.as_ref();
    {
        let g = game.clone();
        listen(wtarget, "keydown", move |e: KeyboardEvent| {
            let code = e.code();
            if matches!(code.as_str(), "Space" | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight") {
                e.prevent_default();
            }
            if !e.repeat() {
                g.borrow_mut().key(&code, true);
            }
        })?;
    }
    {
        let g = game.clone();
        listen(wtarget, "keyup", move |e: KeyboardEvent| g.borrow_mut().key(&e.code(), false))?;
    }
    {
        let g = game.clone();
        listen(wtarget, "blur", move |_: web_sys::Event| g.borrow_mut().blur())?;
    }
    {
        let g = game.clone();
        let doc = document.clone();
        let dtarget: &web_sys::EventTarget = document.as_ref();
        listen(dtarget, "visibilitychange", move |_: web_sys::Event| {
            if doc.visibility_state() == web_sys::VisibilityState::Hidden {
                g.borrow_mut().blur();
            }
        })?;
    }

    if let Some(boot) = document.get_element_by_id("boot") {
        boot.remove();
    }

    // The frame loop.
    let tick: Rc<RefCell<Option<FrameFn>>> = Rc::new(RefCell::new(None));
    let first = tick.clone();
    let mut last = -1.0f64;
    let mut size = (0u32, 0u32, 0.0f64);
    let mut inset_t = 0.0f32;
    *first.borrow_mut() = Some(Closure::new(move |ts: f64| {
        let dt = if last < 0.0 { 0.0 } else { ((ts - last) / 1000.0) as f32 };
        last = ts;
        let dpr = window.device_pixel_ratio().clamp(1.0, 2.0);
        let (cw, ch) = (canvas.client_width().max(1) as u32, canvas.client_height().max(1) as u32);
        if size != (cw, ch, dpr) {
            size = (cw, ch, dpr);
            canvas.set_width((f64::from(cw) * dpr) as u32);
            canvas.set_height((f64::from(ch) * dpr) as u32);
        }
        let _ = ctx.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
        let mut g = game.borrow_mut();
        inset_t -= dt;
        if inset_t <= 0.0 {
            inset_t = 1.0;
            g.set_insets(insets(&window));
        }
        let list = g.frame(dt, cw as f32, ch as f32, &CtxMeasure(&ctx));
        paint(&ctx, &mini, &list);
        if let (Some(save), Some(s)) = (g.take_save(), &storage) {
            let _ = s.set_item(SAVE_KEY, &save);
        }
        drop(g);
        if let Some(cb) = tick.borrow().as_ref() {
            let _ = window.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }));
    let w = web_sys::window().ok_or("no window")?;
    if let Some(cb) = first.borrow().as_ref() {
        w.request_animation_frame(cb.as_ref().unchecked_ref())?;
    }
    Ok(())
}

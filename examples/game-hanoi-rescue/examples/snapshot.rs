//! Renders one raid frame to SVG, for looking at the game without a browser:
//! `cargo run --example snapshot -- <chapter> <seed> <thief index> > frame.svg`
use hanoi_rescue::draw::{Align, Canvas, Draw};
use hanoi_rescue::map::Map;
use hanoi_rescue::raid::{Controls, PlayerStats, Raid, RaidSetup};
use hanoi_rescue::render::{Cam, draw_map, draw_raid};

fn main() {
    let args: Vec<usize> = std::env::args().skip(1).filter_map(|a| a.parse().ok()).collect();
    let (ch, seed, who) = (args.first().copied().unwrap_or(0), args.get(1).copied().unwrap_or(1), args.get(2).copied().unwrap_or(0));
    let map = Map::hoan_kiem();
    let stats = PlayerStats { max_hp: 100.0, speed: 4.0, bag: 6, leash: 2, slippers: 3 };
    let mut raid = Raid::new(&map, RaidSetup::story(&map, ch, stats, seed as u64));
    for _ in 0..30 {
        raid.update(&map, 1.0 / 30.0, &Controls::default());
    }
    let focus = raid.thieves.get(who).map(|t| t.pos).unwrap_or(raid.player.pos);
    if let Some(c) = raid.cages.first() {
        raid.player.pos = c.pos + hanoi_rescue::geom::v2(2.5, 0.5);
    }
    let (w, h) = (844.0, 390.0);
    let mut cv = Canvas::default();
    let cam = Cam::new(focus, w, h);
    draw_map(&mut cv, &map, &cam, 1.0, &raid.setup.exits, false);
    draw_raid(&mut cv, &map, &raid, &cam, 1.0);
    let col = |c: u32| format!("#{:06x}", c & 0xffffff);
    println!("<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' font-family='sans-serif'>");
    for d in cv.list {
        match d {
            Draw::Rect { x, y, w, h, c, a } => println!("<rect x='{x}' y='{y}' width='{w}' height='{h}' fill='{}' fill-opacity='{a}'/>", col(c)),
            Draw::RRect { x, y, w, h, r, c, a } => println!("<rect x='{x}' y='{y}' width='{w}' height='{h}' rx='{r}' fill='{}' fill-opacity='{a}'/>", col(c)),
            Draw::Circle { x, y, r, c, a } => println!("<circle cx='{x}' cy='{y}' r='{r}' fill='{}' fill-opacity='{a}'/>", col(c)),
            Draw::Ring { x, y, r, w, c, a } => {
                println!("<circle cx='{x}' cy='{y}' r='{r}' fill='none' stroke='{}' stroke-width='{w}' stroke-opacity='{a}'/>", col(c))
            }
            Draw::Line { x1, y1, x2, y2, w, c, a } => {
                println!("<line x1='{x1}' y1='{y1}' x2='{x2}' y2='{y2}' stroke='{}' stroke-width='{w}' stroke-opacity='{a}'/>", col(c))
            }
            Draw::Poly { pts, c, a } => println!(
                "<polygon points='{}' fill='{}' fill-opacity='{a}'/>",
                pts.iter().map(|(x, y)| format!("{x},{y}")).collect::<Vec<_>>().join(" "),
                col(c)
            ),
            Draw::Wedge { x, y, r, a0, a1, c, a } => {
                let (x0, y0, x1, y1) = (x + r * a0.cos(), y + r * a0.sin(), x + r * a1.cos(), y + r * a1.sin());
                let large = if a1 - a0 > std::f32::consts::PI { 1 } else { 0 };
                println!("<path d='M{x},{y} L{x0},{y0} A{r},{r} 0 {large} 1 {x1},{y1} Z' fill='{}' fill-opacity='{a}'/>", col(c));
            }
            Draw::Text { x, y, s, size, c, a, align, bold, .. } => {
                let anchor = match align {
                    Align::Left => "start",
                    Align::Center => "middle",
                    Align::Right => "end",
                };
                println!(
                    "<text x='{x}' y='{y}' font-size='{size}' fill='{}' fill-opacity='{a}' text-anchor='{anchor}' dominant-baseline='middle' font-weight='{}' stroke='#000' stroke-width='0.6'>{s}</text>",
                    col(c),
                    if bold { 700 } else { 400 }
                );
            }
            _ => {}
        }
    }
    println!("</svg>");
}

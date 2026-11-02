//! The pages, written with maud (HTML checked by the Rust compiler).

use maud::{DOCTYPE, Markup, html};

use crate::content::{BLOCKS, BRAND, Banner, Block, FEATURES, FOOTER, Gallery, Icon, LEGAL, Layout, MORE_FEATURES, NAV, PROJECTS, SOCIAL, TOOLS, slug};

/// The URL of the stand-in picture for `seed` at `w`×`h`.
pub fn art_url(seed: u32, w: u32, h: u32) -> String {
    format!("/art/{seed}-{w}x{h}.svg")
}

/// Every picture URL the pages use, so the static export can write them all.
pub fn all_art() -> Vec<(u32, u32, u32)> {
    let mut v: Vec<(u32, u32, u32)> = FEATURES.iter().map(|f| (f.seed, 960, 540)).collect();
    v.push((900, 1280, 560));
    v.push((901, 1600, 400));
    v.extend(PROJECTS.iter().map(|p| (p.seed, 640, 360)));
    v.extend(masonry_cells(200, 18).into_iter().map(|(seed, h)| (seed, 400, h)));
    for b in BLOCKS {
        if let Block::Gallery(g) = b {
            v.extend(cells(g).into_iter().map(|(seed, h)| (seed, 400, h)));
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

/// The pictures of a gallery: (seed, height for a 400-wide picture).
fn cells(g: &Gallery) -> Vec<(u32, u32)> {
    match g.layout {
        Layout::Masonry => masonry_cells(g.seed, g.count),
        Layout::Grid => (0..g.count).map(|i| (g.seed.saturating_add(i), 225)).collect(),
    }
}

/// Mixed heights (portrait, square, landscape), the same every time for a seed.
fn masonry_cells(seed: u32, count: u32) -> Vec<(u32, u32)> {
    const HEIGHTS: [u32; 5] = [225, 400, 533, 711, 300];
    (0..count)
        .map(|i| {
            let s = seed.saturating_add(i);
            let pick = (s.wrapping_mul(2_654_435_761) >> 7) as usize % HEIGHTS.len();
            (s, HEIGHTS.get(pick).copied().unwrap_or(400))
        })
        .collect()
}

fn icon(i: Icon) -> Markup {
    let path = match i {
        Icon::Spark => "M12 2l2.2 6.6L21 11l-6.8 2.4L12 20l-2.2-6.6L3 11l6.8-2.4z",
        Icon::Bars => "M4 20V10M9 20V4M14 20v-7M19 20V8",
        Icon::Chip => "M7 7h10v10H7zM9 3v4M15 3v4M9 17v4M15 17v4M3 9h4M3 15h4M17 9h4M17 15h4",
        Icon::Film => "M4 4h16v16H4zM8 4v16M16 4v16M4 9h4M4 15h4M16 9h4M16 15h4",
        Icon::Chat => "M4 5h16v11H9l-5 4z",
        Icon::Star => "M12 3l2.6 5.6 6.1.7-4.5 4.2 1.2 6L12 16.6 6.6 19.5l1.2-6L3.3 9.3l6.1-.7z",
        Icon::Check => "M5 12.5l4.5 4.5L19 7.5",
        Icon::Arrow => "M7 17L17 7M9 7h8v8",
        Icon::Heart => "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
        Icon::Menu => "M4 7h16M4 12h16M4 17h16",
        Icon::Close => "M6 6l12 12M18 6L6 18",
        Icon::Diamond => "M6 4h12l3 5-9 11L3 9z",
        Icon::Left => "M15 5l-7 7 7 7",
        Icon::Right => "M9 5l7 7-7 7",
    };
    html! {
        svg.icon viewBox="0 0 24 24" aria-hidden="true" { path d=(path) {} }
    }
}

fn logo() -> Markup {
    html! {
        svg.logo viewBox="0 0 32 32" aria-hidden="true" {
            rect width="32" height="32" rx="8" fill="#fff" {}
            path d="M10 8v16M10 16l9-8M13 13.5L21 24" stroke="#0f1113" stroke-width="3.2" stroke-linecap="round" fill="none" {}
        }
    }
}

/// The page frame shared by every page: head, header, footer.
fn frame(title: &str, description: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                meta name="description" content=(description);
                meta name="theme-color" content="#0f1113";
                link rel="icon" href="/favicon.svg" type="image/svg+xml";
                link rel="stylesheet" href="/static/style.css";
                script src="/static/app.js" defer {}
            }
            body {
                a.skip href="#main" { "Skip to content" }
                (header())
                main #main { (body) }
                (footer())
            }
        }
    }
}

fn header() -> Markup {
    html! {
        header.top {
            a.brand href="/" aria-label=(format!("{BRAND} home")) { (logo()) }
            nav.nav aria-label="Main" {
                ul {
                    @for (i, l) in NAV.iter().enumerate() {
                        li {
                            a href=(l.href) aria-current=[(i == 0).then_some("page")] {
                                (l.label)
                                @if let Some(b) = l.badge { span.pill-new { (b) } }
                            }
                        }
                    }
                }
            }
            div.top-actions {
                a.chip href="/pricing" { (icon(Icon::Diamond)) "Pricing" span.deal { "30% off" } }
                a.chip href="/enterprise" { (icon(Icon::Spark)) "Enterprise" }
                a.btn-ghost href="/login" { "Log in" }
                a.btn-lime href="/signup" { "Sign up" }
                button.menu-btn type="button" aria-controls="drawer" aria-expanded="false" aria-label="Open menu" {
                    (icon(Icon::Menu))
                }
            }
        }
        div #drawer .drawer hidden {
            div.drawer-head {
                a.brand href="/" aria-label=(format!("{BRAND} home")) { (logo()) }
                button.menu-close type="button" aria-label="Close menu" { (icon(Icon::Close)) }
            }
            nav aria-label="Menu" {
                @for l in NAV {
                    a href=(l.href) { (l.label) @if let Some(b) = l.badge { span.pill-new { (b) } } }
                }
                a href="/pricing" { "Pricing" span.deal { "30% off" } }
                a href="/enterprise" { "Enterprise" }
            }
            div.drawer-actions {
                a.btn-ghost href="/login" { "Log in" }
                a.btn-lime href="/signup" { "Sign up" }
            }
        }
    }
}

fn footer() -> Markup {
    html! {
        footer.foot {
            div.foot-main {
                p.foot-title { "The AI creative" br; "studio" }
                div.foot-cols {
                    @for col in FOOTER {
                        div.foot-col {
                            @for (heading, links) in col.groups {
                                div.foot-group {
                                    h3 { (heading) }
                                    ul { @for l in *links { li { a href=(slug(l)) { (l) } } } }
                                }
                            }
                        }
                    }
                }
            }
            div.foot-meta {
                p { "A Rust rebuild made with Craft Anything. Example content only." }
                ul.social { @for s in SOCIAL { li { a href=(slug(s)) { (s) } } } }
            }
        }
        div.legal {
            p { "© 2026 " (BRAND) " (example site)." }
            ul {
                li { button.lang type="button" { span.globe {} "English" } }
                @for l in LEGAL { li { a href=(slug(l)) { (l) } } }
            }
        }
    }
}

fn picture(seed: u32, w: u32, h: u32, alt: &str) -> Markup {
    html! {
        img src=(art_url(seed, w, h)) width=(w) height=(h) alt=(alt) loading="lazy" decoding="async";
    }
}

fn section_head(id: &str, title: &str, text: &str, actions: Option<Markup>) -> Markup {
    html! {
        div.sec-head {
            div {
                h2 #(format!("{id}-title")) { (title) }
                p { (text) }
            }
            @if let Some(a) = actions { div.sec-actions { (a) } }
        }
    }
}

fn view_all(label: &str, href: &str) -> Markup {
    html! {
        div.fade-out {
            a.btn-view href=(href) { (label) (icon(Icon::Arrow)) }
        }
    }
}

fn masonry(cells: &[(u32, u32)], remix: bool, label: &str) -> Markup {
    html! {
        div.masonry {
            @for (i, (seed, h)) in cells.iter().enumerate() {
                figure.tile {
                    (picture(*seed, 400, *h, &format!("{label}, example {}", i + 1)))
                    @if remix {
                        div.tile-bar {
                            button.like type="button" aria-pressed="false" {
                                (icon(Icon::Heart)) span { (seed.wrapping_mul(37) % 600 + 2) }
                            }
                            a.remix href="/weave" { "Remix" }
                        }
                    }
                }
            }
        }
    }
}

fn gallery(g: &Gallery) -> Markup {
    let cells = cells(g);
    html! {
        section.block #(g.id) aria-labelledby=(format!("{}-title", g.id)) {
            (section_head(g.id, g.title, g.text, None))
            div.capped {
                @match g.layout {
                    Layout::Masonry => (masonry(&cells, false, g.title)),
                    Layout::Grid => div.grid4 {
                        @for (i, (seed, h)) in cells.iter().enumerate() {
                            figure.tile { (picture(*seed, 400, *h, &format!("{}, example {}", g.title, i + 1))) }
                        }
                    },
                }
                (view_all(g.view_all, &slug(g.title)))
            }
        }
    }
}

fn banner(b: &Banner) -> Markup {
    html! {
        section.block {
            div class=(format!("banner {}", b.class)) {
                span.banner-tag { (b.tag) }
                h2 { @for l in b.lines { (l) br; } }
                p { (b.text) }
                a.btn-white href=(b.href) { (icon(Icon::Spark)) (b.button) }
            }
        }
    }
}

pub fn home() -> Markup {
    let community = masonry_cells(200, 18);
    let body = html! {
        // Carousel of what's new.
        section.carousel aria-label="What's new" {
            button.car-btn.prev type="button" aria-label="Previous" { (icon(Icon::Left)) }
            div.car-track {
                @for f in FEATURES {
                    a.feature href=(f.href) {
                        div.feature-media { (picture(f.seed, 960, 540, "")) span.feature-mark { (f.title) } }
                        h3 { (f.title) }
                        p { (f.text) }
                    }
                }
            }
            button.car-btn.next type="button" aria-label="Next" { (icon(Icon::Right)) }
        }

        // Sign-up offer and the tool tiles beside it.
        section.promo-row aria-label="Get started" {
            a.promo href="/signup" {
                (picture(900, 1280, 560, ""))
                div.promo-body {
                    h2 { "Join today and unlock" br; span.lime { "a launch bonus" } }
                    ul.checks {
                        li { (icon(Icon::Check)) "Every top model on one plan" }
                        li { (icon(Icon::Check)) "Bonus credits for your first month" }
                    }
                    span.btn-lime.big { "Create your free account" }
                }
            }
            div.tools {
                @for t in TOOLS {
                    a.tool href=(t.href) {
                        (icon(t.icon))
                        div {
                            h3 { (t.title) @if let Some(b) = t.badge { span.pill-lime { (b) } } }
                            p { (t.text) }
                        }
                    }
                }
            }
        }

        // Awards banner.
        section.block aria-label="Short film awards" {
            div.awards {
                (picture(901, 1600, 400, ""))
                div.awards-body {
                    p.awards-kicker { (BRAND) " Short Film Awards" }
                    h2 { "The winners are in" }
                    p { "Meet the filmmakers and watch the winning shorts from this year's awards." }
                    a.btn-gold href="/winners" { "See the winning films" }
                }
            }
        }

        // Community remixes.
        section.block #weave aria-labelledby="weave-title" {
            (section_head("weave", "Weave by the community",
                "Take any motion and recast it with your own characters, places and products, or swap one detail and keep the rest.",
                Some(html! {
                    a.btn-lime href="/weave" { "Start creating" }
                    a.btn-white href="/school" { "How it works" }
                })))
            div.capped.tall {
                (masonry(&community, true, "Community remix"))
                (view_all("View all presets", "/weave"))
            }
        }

        // Projects with their prompts and assets.
        section.block #projects aria-labelledby="projects-title" {
            (section_head("projects", "See inside every project", "Every prompt and asset, and how each project was made.", None))
            div.capped {
                div.projects {
                    @for p in PROJECTS {
                        a.project href="/community" {
                            (picture(p.seed, 640, 360, ""))
                            span.project-title { (p.title) }
                            span.views { (icon(Icon::Heart)) (p.views) }
                        }
                    }
                }
                (view_all("Explore the community", "/community"))
            }
        }

        @for b in BLOCKS {
            @match b {
                Block::Gallery(g) => (gallery(g)),
                Block::Banner(bn) => (banner(bn)),
            }
        }

        section.block.more aria-labelledby="more-title" {
            h2 #more-title { "More ways to create" }
            ul.tags { @for t in MORE_FEATURES { li { a href=(slug(t)) { (t) } } } }
        }
    };
    frame(&format!("{BRAND}: the AI creative studio"), "Make video, images and sound with every top AI model in one place.", body)
}

/// A page the original has that this rebuild doesn't cover yet.
pub fn placeholder(title: &str) -> Markup {
    let body = html! {
        section.block.placeholder {
            h1 { (title) }
            p { "This rebuild covers the home page; this one is on the way." }
            a.btn-lime href="/" { "Back to the home page" }
        }
    };
    frame(&format!("{title} · {BRAND}"), title, body)
}

pub fn not_found() -> Markup {
    let body = html! {
        section.block.placeholder {
            h1 { "Page not found" }
            p { "Nothing lives at this address." }
            a.btn-lime href="/" { "Back to the home page" }
        }
    };
    frame(&format!("Not found · {BRAND}"), "Page not found", body)
}

/// The logo as a file of its own, for the browser tab.
pub fn favicon() -> String {
    logo().into_string().replacen("<svg class=\"logo\"", "<svg xmlns=\"http://www.w3.org/2000/svg\"", 1)
}

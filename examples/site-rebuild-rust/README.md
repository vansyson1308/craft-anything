# Example: a website rebuilt in Rust

A landing page for **Kinora**, a made-up AI creative studio, rebuilt in Rust with Craft Anything
from a public AI video-and-image platform's home page (studied 2026-11-02). It follows that
page's layout and design language: sticky top bar, a carousel of what's new, a sign-up offer
beside six tool tiles, an awards banner, community galleries (masonry and grid) that fade into a
"View all" button, wide coloured feature banners, a tag cloud and a lime footer.

It copies nothing: the name, logo, words and every picture are our own. The pictures are
abstract scenes drawn in Rust from a seed (`src/art.rs`), where the original shows people's
generations.

## Run it

```sh
cargo run --release                         # http://127.0.0.1:8080
cargo run --release -- --addr 0.0.0.0:3000  # another address
cargo run --release -- export dist          # the whole site as static files in dist/
```

## How it's made

| Part | Where |
|---|---|
| Server: axum, one route per kind of file, a page for every link the site has | `src/main.rs` |
| Every word on the page, as typed data | `src/content.rs` |
| The page, as maud templates (HTML the compiler checks) | `src/pages.rs` |
| Stand-in pictures, deterministic SVG | `src/art.rs` |
| Design tokens and layout (desktop, tablet, phone) | `assets/style.css` |
| Phone menu, carousel arrows, likes (optional: the page works without it) | `assets/app.js` |

Only the home page is rebuilt; every other link opens a short page saying so, never a 404.

## How it was checked

With Craft Anything's own tools, in Clean room (only what a visitor sees), over its MCP server
(`craft-anything-cli --allow-run mcp --all-tools`):

1. `web_site_open` + `web_site_capture` on the original at 1440 and 390 px, with `noise_pass`
   to mark what moves by itself. The capture's design tokens gave the colours (`#0f1113` page,
   `#1c1e20` cards, `#d1fe17` accent), radii, fonts and spacing used in `style.css`.
2. The same capture of this site, served locally.
3. `web_pixels_compare` with `common_area` and the original's noise mask:

| Width | Layout similarity | Perceptual similarity |
|---|---|---|
| 1440 px | 0.31 | 0.16 |
| 390 px | 0.61 | 0.56 |

The page heights match within 2% (11,080 vs 11,278 px at 1440) and the sections sit in the same
order and places; most of the remaining difference is the pictures, which are deliberately not
the original's. No console errors and no sideways scrolling at either width.

`cargo test` covers every page, picture and link on the home page resolving, hostile picture
addresses (404, never a crash), the static export and the command line.

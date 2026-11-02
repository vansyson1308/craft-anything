//! Everything the page says, as typed data. Templates in `pages.rs` only lay it out.
//!
//! The brand, names and words are our own: the rebuild follows the original's layout and
//! design language, never its name, logo, wording or media.

pub const BRAND: &str = "Kinora";

pub struct NavLink {
    pub label: &'static str,
    pub href: &'static str,
    pub badge: Option<&'static str>,
}

const fn nav(label: &'static str, href: &'static str, badge: Option<&'static str>) -> NavLink {
    NavLink { label, href, badge }
}

pub const NAV: &[NavLink] = &[
    nav("Explore", "/", None),
    nav("Image", "/image", None),
    nav("Video", "/video", None),
    nav("Audio", "/audio", None),
    nav("Agents", "/agents", None),
    nav("API", "/api-docs", None),
    nav("Weave", "/weave", Some("New")),
    nav("Content Mill", "/content-mill", Some("New")),
    nav("Arcade", "/arcade", Some("New")),
    nav("Avatars", "/avatars", None),
    nav("School", "/school", Some("New")),
    nav("Effects", "/effects", None),
    nav("Cinema Room", "/cinema-room", None),
    nav("Contests", "/contests", None),
    nav("Store Studio", "/store-studio", None),
    nav("Canvas", "/canvas", None),
    nav("Community", "/community", None),
];

/// One wide card in the carousel at the top.
pub struct Feature {
    pub title: &'static str,
    pub text: &'static str,
    pub seed: u32,
    pub href: &'static str,
}

pub const FEATURES: &[Feature] = &[
    Feature { title: "Kinora Weave", text: "Drop in one clip. Get a dozen new takes.", seed: 11, href: "/weave" },
    Feature { title: "Vector Mode", text: "Turn a prompt or a picture into clean, editable SVG.", seed: 23, href: "/image" },
    Feature { title: "Kinora Arcade", text: "Describe a game and play it on your phone a minute later.", seed: 37, href: "/arcade" },
    Feature { title: "Content Mill", text: "A month of posts from a single product photo.", seed: 41, href: "/content-mill" },
    Feature { title: "Motion 4", text: "Long, steady shots with camera moves you direct.", seed: 53, href: "/video" },
    Feature { title: "Weave Lite", text: "Fast drafts for quick ideas, at a fraction of the cost.", seed: 67, href: "/weave" },
];

/// A small tile beside the sign-up banner.
pub struct Tool {
    pub icon: Icon,
    pub title: &'static str,
    pub text: &'static str,
    pub badge: Option<&'static str>,
    pub href: &'static str,
}

#[derive(Clone, Copy)]
pub enum Icon {
    Spark,
    Bars,
    Chip,
    Film,
    Chat,
    Star,
    Check,
    Arrow,
    Heart,
    Menu,
    Close,
    Diamond,
    Left,
    Right,
}

pub const TOOLS: &[Tool] = &[
    Tool { icon: Icon::Spark, title: "Kinora Weave", text: "One clip, many versions", badge: Some("New"), href: "/weave" },
    Tool { icon: Icon::Bars, title: "Motion 4", text: "Our most capable video model", badge: None, href: "/video" },
    Tool { icon: Icon::Chip, title: "Director Agent", text: "Plans, prompts and renders for you", badge: None, href: "/agents" },
    Tool { icon: Icon::Film, title: "Cinema Room 2", text: "Cinematic scenes, shot by shot", badge: None, href: "/cinema-room" },
    Tool { icon: Icon::Chat, title: "Kinora in your assistant", text: "Make images and videos from any chat", badge: None, href: "/agents" },
    Tool { icon: Icon::Star, title: "Avatars", text: "A presenter that never needs a break", badge: None, href: "/avatars" },
];

/// How a gallery lays out its pictures.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Columns of pictures of different heights.
    Masonry,
    /// Rows of equal pictures.
    Grid,
}

pub struct Gallery {
    pub id: &'static str,
    pub title: &'static str,
    pub text: &'static str,
    pub layout: Layout,
    pub count: u32,
    pub seed: u32,
    pub view_all: &'static str,
}

pub struct Project {
    pub title: &'static str,
    pub seed: u32,
    pub views: &'static str,
}

pub const PROJECTS: &[Project] = &[
    Project { title: "Paper Moon", seed: 101, views: "1.4K" },
    Project { title: "Static", seed: 102, views: "669" },
    Project { title: "Saltwater", seed: 103, views: "583" },
    Project { title: "Night Bus", seed: 104, views: "1.3K" },
    Project { title: "The Lantern", seed: 105, views: "403" },
    Project { title: "Low Tide", seed: 106, views: "1.3K" },
    Project { title: "Glasshouse", seed: 107, views: "754" },
    Project { title: "Overtime", seed: 108, views: "220" },
    Project { title: "Northbound", seed: 109, views: "214" },
    Project { title: "Second Draft", seed: 110, views: "832" },
    Project { title: "Kite Season", seed: 111, views: "659" },
    Project { title: "Small Hours", seed: 112, views: "559" },
];

/// A wide coloured banner between galleries.
pub struct Banner {
    pub class: &'static str,
    pub tag: &'static str,
    pub lines: &'static [&'static str],
    pub text: &'static str,
    pub button: &'static str,
    pub href: &'static str,
}

pub enum Block {
    Gallery(Gallery),
    Banner(Banner),
}

pub const BLOCKS: &[Block] = &[
    Block::Gallery(Gallery {
        id: "motion-4",
        title: "Motion 4",
        text: "Our most capable AI video model.",
        layout: Layout::Masonry,
        count: 16,
        seed: 300,
        view_all: "View all of Motion 4",
    }),
    Block::Gallery(Gallery {
        id: "prism-image-2",
        title: "Prism Image 2",
        text: "Crisp 4K images, with lettering that reads right.",
        layout: Layout::Masonry,
        count: 14,
        seed: 400,
        view_all: "View all of Prism Image 2",
    }),
    Block::Banner(Banner {
        class: "banner-canvas",
        tag: "New feature",
        lines: &["One board.", "Every step."],
        text: "Collect references, chain tools together and share it all with your team, on one board.",
        button: "Open Canvas",
        href: "/canvas",
    }),
    Block::Gallery(Gallery {
        id: "store-studio",
        title: "Store Studio",
        text: "What brands and creators are making with Store Studio.",
        layout: Layout::Masonry,
        count: 14,
        seed: 500,
        view_all: "View all of Store Studio",
    }),
    Block::Gallery(Gallery {
        id: "motion-3",
        title: "Motion 3",
        text: "Popular video generations from the Kinora community.",
        layout: Layout::Grid,
        count: 16,
        seed: 600,
        view_all: "View all of Motion 3",
    }),
    Block::Banner(Banner {
        class: "banner-photoset",
        tag: "Photo Set",
        lines: &["New looks.", "Same face."],
        text: "Design your character once. We'll shoot the rest.",
        button: "Try Photo Set",
        href: "/avatars",
    }),
    Block::Gallery(Gallery {
        id: "film-look",
        title: "Kinora Film Look",
        text: "Community stills with the grain, glow and colour of real film stock.",
        layout: Layout::Grid,
        count: 16,
        seed: 700,
        view_all: "View all of Film Look",
    }),
    Block::Gallery(Gallery {
        id: "portrait-2",
        title: "Kinora Portrait 2",
        text: "A photo model made for fashion, style and self-expression.",
        layout: Layout::Masonry,
        count: 14,
        seed: 800,
        view_all: "View all of Portrait 2",
    }),
];

pub const MORE_FEATURES: &[&str] = &[
    "Cinema Room",
    "Visual Effects",
    "Portrait",
    "Camera Moves",
    "Trending",
    "Action Moves",
    "Commercials",
    "Motion 3",
    "Motion 4",
    "Community",
    "Prism Image 2",
    "Vector Mode",
    "Upscale",
    "Lip Sync",
    "Weave",
    "Product Placement",
    "Edit Image",
    "Multi Reference",
    "Short Video",
    "Reels",
    "Stories",
    "Film Look",
    "Character Lock",
    "Sound Effects",
    "Music",
    "Voice",
    "Storyboards",
    "Effects Library",
];

pub struct FooterColumn {
    pub groups: &'static [(&'static str, &'static [&'static str])],
}

pub const FOOTER: &[FooterColumn] = &[
    FooterColumn {
        groups: &[(
            "Create",
            &["AI Video", "Video Effects", "AI Image", "Edit Image", "Inpaint", "Upscale", "Video Upscaler", "Mixed Media", "Face Swap", "Avatars", "Apps"],
        )],
    },
    FooterColumn {
        groups: &[
            ("Video models", &["Motion 4", "Motion 3", "Motion Lite", "Weave", "Weave Lite"]),
            ("Image models", &["Prism Image 2", "Prism Image", "Portrait 2", "Vector Mode"]),
        ],
    },
    FooterColumn {
        groups: &[("Studios", &["Cinema Room", "Store Studio", "Lip Sync Studio", "Canvas"]), ("Portrait", &["Portrait 2", "Character Lock", "Film Look"])],
    },
    FooterColumn {
        groups: &[
            ("Platform", &["Director Agent", "Motion Designer", "Agents", "API", "Teams", "Arcade", "Browser Extension"]),
            ("Resources", &["Blog", "Help Center", "School", "Prompt Guide"]),
        ],
    },
    FooterColumn {
        groups: &[
            ("Company", &["About", "Trust", "Security", "Enterprise", "Team", "Pricing", "Careers", "Contact"]),
            ("Community", &["Community", "Contests", "Creative Partners"]),
        ],
    },
];

pub const SOCIAL: &[&str] = &["X", "YouTube", "LinkedIn", "TikTok", "Discord"];

pub const LEGAL: &[&str] = &["Help center", "Cookies", "Cookie settings", "Terms", "Privacy", "Security"];

/// Turns a label into the path its page would have ("Edit Image" -> "/edit-image").
pub fn slug(label: &str) -> String {
    let mut out = String::from("/");
    let mut dash = false;
    for c in label.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            dash = false;
        } else if !dash && out.len() > 1 {
            out.push('-');
            dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

/// Every page the site links to besides the home page: (path, title).
fn entries() -> impl Iterator<Item = (String, &'static str)> {
    let nav = NAV.iter().filter(|l| l.href != "/").map(|l| (l.href.to_string(), l.label));
    let footer = FOOTER.iter().flat_map(|c| c.groups.iter()).flat_map(|(_, links)| links.iter()).map(|l| (slug(l), *l));
    let more = MORE_FEATURES.iter().map(|l| (slug(l), *l));
    let legal = LEGAL.iter().chain(SOCIAL.iter()).map(|l| (slug(l), *l));
    let extra = [("/pricing", "Pricing"), ("/enterprise", "Enterprise"), ("/login", "Log in"), ("/signup", "Sign up"), ("/winners", "Short Film Awards")]
        .into_iter()
        .map(|(p, t)| (p.to_string(), t));
    let galleries = BLOCKS.iter().filter_map(|b| match b {
        Block::Gallery(g) => Some((slug(g.title), g.title)),
        Block::Banner(_) => None,
    });
    nav.chain(footer).chain(more).chain(legal).chain(galleries).chain(extra)
}

/// The title of a page the rebuild doesn't have yet, from its path, if the site links to it.
pub fn known_page(path: &str) -> Option<&'static str> {
    entries().find(|(href, _)| href == path).map(|(_, label)| label)
}

/// The paths of every such page, once each.
pub fn all_pages() -> Vec<String> {
    let mut v: Vec<String> = entries().map(|(p, _)| p).collect();
    v.sort_unstable();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_are_clean_paths() {
        assert_eq!(slug("Edit Image"), "/edit-image");
        assert_eq!(slug("X"), "/x");
        assert_eq!(slug("Prism Image 2"), "/prism-image-2");
        assert_eq!(slug("  --  "), "/");
    }

    #[test]
    fn every_nav_link_has_a_page() {
        for l in NAV.iter().filter(|l| l.href != "/") {
            assert!(known_page(l.href).is_some(), "{}", l.href);
        }
        assert_eq!(known_page("/edit-image"), Some("Edit Image"));
        assert_eq!(known_page("/nope"), None);
    }
}

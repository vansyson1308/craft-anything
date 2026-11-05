//! Mipmaps and anisotropic filtering for every texture loaded from a file. PNG and JPEG arrive
//! with a single level, so pavements and walls shimmer and turn to noise a few metres away; each
//! one gets its smaller levels built once, on load (averaged in linear light for colour images).
//!
//! Building them is real work for the page's single thread (a 2048 px image is millions of
//! samples), and textures keep arriving as the player walks into new streets. A profile of the
//! game showed this system taking almost half the busy time, in single frames of 70–110 ms: the
//! hitches people felt as stutter. So the work is cut into rows: each frame builds rows for at
//! most `BUDGET_MS`, whatever the image's size, and an image gets its levels in one go when they are
//! all built (one upload). The buffer for every level is reserved once.

use std::collections::{HashSet, VecDeque};

use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::platform::time::Instant;
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

/// Images smaller than this on both sides are left alone (UI pieces, tiny masks).
const MIN_SIDE: u32 = 32;

/// How long a frame may spend building mipmaps.
const BUDGET_MS: f64 = 4.0;

/// An image whose levels are being built, row by row.
pub struct Job {
    id: AssetId<Image>,
    ch: usize,
    srgb: bool,
    /// Every level so far, the full image first.
    data: Vec<u8>,
    /// Where the level being read starts in `data`, and its size.
    from: usize,
    w: u32,
    h: u32,
    /// The next row to build of the level after it.
    row: u32,
    levels: u32,
}

impl Job {
    /// A job for an image that needs mipmaps, or `None` if it doesn't take them.
    pub fn new(id: AssetId<Image>, img: &Image) -> Option<Job> {
        let d = &img.texture_descriptor;
        if d.dimension != TextureDimension::D2 || d.mip_level_count != 1 || d.size.depth_or_array_layers != 1 {
            return None;
        }
        let (ch, srgb) = channels(d.format)?;
        let (w, h) = (d.size.width, d.size.height);
        if w < MIN_SIDE && h < MIN_SIDE {
            return None;
        }
        let src = img.data.as_ref()?;
        if src.len() != (w as usize) * (h as usize) * ch {
            return None;
        }
        // Every smaller level together is under a third of the image.
        let mut data = Vec::with_capacity(src.len() + src.len() / 3 + 64);
        data.extend_from_slice(src);
        Some(Job { id, ch, srgb, data, from: 0, w, h, row: 0, levels: 1 })
    }

    /// Whether every level is built.
    pub fn done(&self) -> bool {
        self.w <= 1 && self.h <= 1
    }

    /// Builds rows of the next level until `deadline` says stop; returns whether the job is done.
    pub fn step(&mut self, tables: &Tables, mut deadline: impl FnMut() -> bool) -> bool {
        while !self.done() {
            let (w, h, ch) = (self.w, self.h, self.ch);
            let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
            let next_start = self.from + (w as usize) * (h as usize) * ch;
            while self.row < nh {
                let y = self.row;
                for x in 0..nw {
                    for c in 0..ch {
                        let colour = self.srgb && c < 3;
                        let mut sum = 0.0f32;
                        let mut n = 0.0f32;
                        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                            let i = self.from + ((sy * w + sx) as usize) * ch + c;
                            if let Some(&v) = self.data.get(i) {
                                sum += if colour { tables.linear.get(v as usize).copied().unwrap_or(0.0) } else { v as f32 };
                                n += 1.0;
                            }
                        }
                        let avg = sum / n.max(1.0);
                        let out = if colour {
                            tables.srgb.get((avg.clamp(0.0, 1.0) * 4095.0 + 0.5) as usize).copied().unwrap_or(255)
                        } else {
                            (avg + 0.5).clamp(0.0, 255.0) as u8
                        };
                        self.data.push(out);
                    }
                }
                self.row += 1;
                if deadline() {
                    return false;
                }
            }
            self.from = next_start;
            self.w = nw;
            self.h = nh;
            self.row = 0;
            self.levels += 1;
        }
        true
    }

    /// Puts the finished levels into the image, with a filtering sampler.
    pub fn finish(self, img: &mut Image) {
        img.data = Some(self.data);
        img.texture_descriptor.mip_level_count = self.levels;
        let mut s = match &img.sampler {
            ImageSampler::Descriptor(s) => s.clone(),
            ImageSampler::Default => ImageSamplerDescriptor::linear(),
        };
        s.mag_filter = ImageFilterMode::Linear;
        s.min_filter = ImageFilterMode::Linear;
        s.mipmap_filter = ImageFilterMode::Linear;
        s.anisotropy_clamp = 16;
        img.sampler = ImageSampler::Descriptor(s);
    }
}

/// sRGB to linear light, and back in 4096 steps (no powf per pixel).
pub struct Tables {
    linear: Vec<f32>,
    srgb: Vec<u8>,
}

impl Default for Tables {
    fn default() -> Self {
        Tables { linear: (0..=255u8).map(to_linear).collect(), srgb: (0..4096).map(|i| to_srgb(i as f32 / 4095.0)).collect() }
    }
}

pub fn make_mips(
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
    mut done: Local<HashSet<AssetId<Image>>>,
    mut queue: Local<VecDeque<AssetId<Image>>>,
    mut job: Local<Option<Job>>,
    tables: Local<Tables>,
) {
    for e in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = e
            && !done.contains(id)
            && assets.get_path(*id).is_some()
        {
            done.insert(*id);
            queue.push_back(*id);
        }
    }
    let start = Instant::now();
    let over = || start.elapsed().as_secs_f64() * 1000.0 > BUDGET_MS;
    loop {
        if job.is_none() {
            let Some(id) = queue.pop_front() else { return };
            *job = images.get(id).and_then(|img| Job::new(id, img));
            continue;
        }
        let Some(j) = job.as_mut() else { return };
        if j.step(&tables, over) {
            if let Some(j) = job.take()
                && let Some(mut img) = images.get_mut(j.id)
            {
                j.finish(&mut img);
            }
        } else {
            return;
        }
        if over() {
            return;
        }
    }
}

fn channels(f: TextureFormat) -> Option<(usize, bool)> {
    match f {
        TextureFormat::Rgba8UnormSrgb => Some((4, true)),
        TextureFormat::Rgba8Unorm => Some((4, false)),
        TextureFormat::Rg8Unorm => Some((2, false)),
        TextureFormat::R8Unorm => Some((1, false)),
        _ => None,
    }
}

fn to_linear(v: u8) -> f32 {
    let c = v as f32 / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

fn to_srgb(l: f32) -> u8 {
    let c = if l <= 0.003_130_8 { l * 12.92 } else { 1.055 * l.powf(1.0 / 2.4) - 0.055 };
    (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Builds the whole mip chain of a plain 2D image at once.
#[cfg(test)]
pub fn add_mips(img: &mut Image) {
    let Some(mut j) = Job::new(AssetId::default(), img) else { return };
    j.step(&Tables::default(), || false);
    j.finish(img);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::Extent3d;

    fn image(w: u32, h: u32, f: TextureFormat, px: &[u8]) -> Image {
        let n = (w * h) as usize;
        let data: Vec<u8> = px.iter().copied().cycle().take(n * px.len()).collect();
        Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, data, f, RenderAssetUsages::default())
    }

    #[test]
    fn builds_every_level() {
        let mut img = image(64, 32, TextureFormat::Rgba8UnormSrgb, &[200, 100, 50, 255]);
        add_mips(&mut img);
        assert_eq!(img.texture_descriptor.mip_level_count, 7);
        let total: usize = [64 * 32, 32 * 16, 16 * 8, 8 * 4, 4 * 2, 2, 1].iter().sum::<usize>() * 4;
        assert_eq!(img.data.as_ref().map(Vec::len), Some(total));
        // A flat colour stays the same colour at every level.
        assert_eq!(img.data.as_ref().and_then(|d| d.get(total - 4..)).map(<[u8]>::to_vec), Some(vec![200, 100, 50, 255]));
    }

    #[test]
    fn leaves_odd_images_alone() {
        let mut small = image(8, 8, TextureFormat::Rgba8Unorm, &[1, 2, 3, 4]);
        add_mips(&mut small);
        assert_eq!(small.texture_descriptor.mip_level_count, 1);
        let mut wrong = image(64, 64, TextureFormat::R8Unorm, &[9]);
        wrong.data = Some(vec![0; 10]);
        add_mips(&mut wrong);
        assert_eq!(wrong.texture_descriptor.mip_level_count, 1);
    }

    #[test]
    fn a_job_cut_into_rows_gives_the_same_levels() {
        // A gradient, so every level differs.
        let mut px = Vec::new();
        for y in 0..48u32 {
            for x in 0..80u32 {
                px.extend_from_slice(&[(x * 3) as u8, (y * 5) as u8, ((x + y) * 2) as u8, 255]);
            }
        }
        let whole = {
            let mut img = Image::new(
                Extent3d { width: 80, height: 48, depth_or_array_layers: 1 },
                TextureDimension::D2,
                px.clone(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            );
            add_mips(&mut img);
            img.data
        };
        let mut img = Image::new(
            Extent3d { width: 80, height: 48, depth_or_array_layers: 1 },
            TextureDimension::D2,
            px,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let mut job = Job::new(AssetId::default(), &img).expect("a job");
        let tables = Tables::default();
        // Stop after every single row.
        let mut steps = 0;
        while !job.step(&tables, || true) {
            steps += 1;
            assert!(steps < 1000, "never finishes");
        }
        assert!(steps > 20, "it was cut up ({steps} steps)");
        job.finish(&mut img);
        assert_eq!(img.data, whole);
        assert_eq!(img.texture_descriptor.mip_level_count, 7);
    }
}

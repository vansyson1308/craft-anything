//! Mipmaps and anisotropic filtering for every texture loaded from a file. PNG and JPEG arrive
//! with a single level, so pavements and walls shimmer and turn to noise a few metres away; each
//! one gets its smaller levels built once, on load (averaged in linear light for colour images).

use std::collections::HashSet;

use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{TextureDimension, TextureFormat};

/// Images smaller than this on both sides are left alone (UI pieces, tiny masks).
const MIN_SIDE: u32 = 32;

pub fn make_mips(
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    assets: Res<AssetServer>,
    mut done: Local<HashSet<AssetId<Image>>>,
) {
    let ids: Vec<AssetId<Image>> = events
        .read()
        .filter_map(|e| match e {
            AssetEvent::LoadedWithDependencies { id } => Some(*id),
            _ => None,
        })
        .collect();
    for id in ids {
        if done.contains(&id) || assets.get_path(id).is_none() {
            continue;
        }
        done.insert(id);
        if let Some(mut img) = images.get_mut(id) {
            add_mips(&mut img);
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

/// Builds the mip chain of a plain 2D image in place; anything else is left as it is.
pub fn add_mips(img: &mut Image) {
    let d = &img.texture_descriptor;
    if d.dimension != TextureDimension::D2 || d.mip_level_count != 1 || d.size.depth_or_array_layers != 1 {
        return;
    }
    let Some((ch, srgb)) = channels(d.format) else { return };
    let (mut w, mut h) = (d.size.width, d.size.height);
    if w < MIN_SIDE && h < MIN_SIDE {
        return;
    }
    let Some(data) = img.data.as_mut() else { return };
    if data.len() != (w as usize) * (h as usize) * ch {
        return;
    }
    let lut: Vec<f32> = (0..=255u8).map(to_linear).collect();
    let mut level: Vec<u8> = data.clone();
    let mut levels = 1u32;
    while w > 1 || h > 1 {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let mut next = vec![0u8; (nw as usize) * (nh as usize) * ch];
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..ch {
                    let colour = srgb && c < 3;
                    let mut sum = 0.0f32;
                    let mut n = 0.0f32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (sx, sy) = ((x * 2 + dx).min(w - 1), (y * 2 + dy).min(h - 1));
                        let i = ((sy * w + sx) as usize) * ch + c;
                        if let Some(&v) = level.get(i) {
                            sum += if colour { lut.get(v as usize).copied().unwrap_or(0.0) } else { v as f32 };
                            n += 1.0;
                        }
                    }
                    let avg = sum / n.max(1.0);
                    let out = if colour { to_srgb(avg) } else { (avg + 0.5).clamp(0.0, 255.0) as u8 };
                    if let Some(o) = next.get_mut(((y * nw + x) as usize) * ch + c) {
                        *o = out;
                    }
                }
            }
        }
        data.extend_from_slice(&next);
        level = next;
        w = nw;
        h = nh;
        levels += 1;
    }
    img.texture_descriptor.mip_level_count = levels;
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
}

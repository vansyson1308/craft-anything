//! How the town is lit and shown. On WebGPU (and natively): a physically based sky with the sun low
//! over the lake, the sky lighting everything it can see, soft cascaded shadows, screen-space
//! ambient occlusion, bloom and temporal anti-aliasing. On WebGL2, which can't run those passes:
//! the same sun with simpler shadows, fog and multisampling.
//!
//! Each effect can be switched off from the page's address, for slower computers and for finding
//! a misbehaving one: `?off=atmo,env,ssao,taa,bloom,shadows` (any of them).

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap, ShadowFilteringMethod};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

/// Whether this build has the full renderer.
pub const HIGH: bool = cfg!(any(feature = "webgpu", not(target_arch = "wasm32")));

/// Which effects are on.
#[derive(Resource, Clone, Copy, Debug)]
pub struct Fx {
    /// The physically based sky (and its haze over distance).
    pub atmo: bool,
    /// Sky light on everything (needs `atmo`).
    pub env: bool,
    pub ssao: bool,
    pub taa: bool,
    pub bloom: bool,
    pub shadows: bool,
}

impl Fx {
    /// Everything the build can do, minus what the address switches off.
    pub fn from_page() -> Fx {
        let off = page_query("off");
        let on = |name: &str, available: bool| available && !off.split(',').any(|o| o.trim() == name);
        let atmo = on("atmo", HIGH);
        Fx { atmo, env: atmo && on("env", HIGH), ssao: on("ssao", HIGH), taa: on("taa", HIGH), bloom: on("bloom", true), shadows: on("shadows", true) }
    }
}

/// A value from the page's query string (`?name=value`), or "" off the web.
pub fn page_query(name: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let search = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
        for pair in search.trim_start_matches('?').split('&') {
            let mut kv = pair.splitn(2, '=');
            if kv.next() == Some(name) {
                return kv.next().unwrap_or("").to_string();
            }
        }
    }
    let _ = name;
    String::new()
}

/// Light that something gives off itself (windows, lanterns, signs). Bevy adds it after the
/// camera's exposure, so it is in display units: a little dimmer under the bright WebGPU sky.
pub fn glow(r: f32, g: f32, b: f32) -> LinearRgba {
    let k = if HIGH { 0.8 } else { 1.0 };
    LinearRgba::rgb(r * k, g * k, b * k)
}

#[derive(Component)]
pub struct Sun;

/// Where the sun stands: late afternoon, low in the west-south-west, so the streets running
/// east-west catch long shadows and the lake turns gold.
fn sun_transform() -> Transform {
    Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, 1.15, -0.26, 0.0))
}

/// The sun and the global light settings.
pub fn spawn_sun(commands: &mut Commands, fx: Fx) {
    if fx.atmo {
        // Real units: the sun as it is above the atmosphere, which then filters it.
        commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
        commands.insert_resource(GlobalAmbientLight { brightness: if fx.env { 0.0 } else { 9000.0 }, color: Color::srgb(0.7, 0.8, 1.0), ..default() });
        commands.spawn((
            Sun,
            DirectionalLight { illuminance: bevy::light::light_consts::lux::RAW_SUNLIGHT, shadow_maps_enabled: fx.shadows, ..default() },
            bevy::light::SunDisk::EARTH,
            sun_transform(),
            CascadeShadowConfigBuilder {
                num_cascades: 4,
                minimum_distance: 0.1,
                first_cascade_far_bound: 14.0,
                maximum_distance: 260.0,
                overlap_proportion: 0.25,
            }
            .build(),
        ));
    } else {
        // WebGL2 gets a single cascade, so it is a large one.
        commands.insert_resource(DirectionalLightShadowMap { size: 4096 });
        commands.insert_resource(ClearColor(Color::srgb(0.56, 0.66, 0.82)));
        commands.insert_resource(GlobalAmbientLight { color: Color::srgb(0.7, 0.75, 0.95), brightness: 520.0, ..default() });
        commands.spawn((
            Sun,
            DirectionalLight { illuminance: 9000.0, color: Color::srgb(1.0, 0.82, 0.62), shadow_maps_enabled: fx.shadows, ..default() },
            sun_transform(),
            if HIGH {
                CascadeShadowConfigBuilder {
                    num_cascades: 4,
                    minimum_distance: 0.1,
                    first_cascade_far_bound: 14.0,
                    maximum_distance: 220.0,
                    overlap_proportion: 0.25,
                }
                .build()
            } else {
                CascadeShadowConfigBuilder { num_cascades: 1, first_cascade_far_bound: 70.0, maximum_distance: 70.0, ..default() }.build()
            },
        ));
    }
}

/// Adds the renderer's settings to the main camera.
pub fn setup_camera(e: &mut EntityCommands, media: &mut Assets<bevy::light::atmosphere::ScatteringMedium>, fx: Fx) {
    e.insert(Tonemapping::AgX);
    if fx.bloom {
        e.insert(Bloom::NATURAL);
    }
    if !fx.atmo {
        e.insert(DistanceFog { color: Color::srgb(0.62, 0.68, 0.8), falloff: FogFalloff::Linear { start: 140.0, end: 650.0 }, ..default() });
    }
    #[cfg(any(feature = "webgpu", not(target_arch = "wasm32")))]
    {
        use bevy::anti_alias::taa::TemporalAntiAliasing;
        use bevy::camera::Exposure;
        use bevy::pbr::{AtmosphereSettings, ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
        if fx.atmo {
            let medium = media.add(bevy::light::atmosphere::ScatteringMedium::earth(256, 256));
            e.commands().spawn(bevy::light::atmosphere::Atmosphere::earth(medium));
            e.insert((AtmosphereSettings { aerial_view_lut_max_distance: 4000.0, ..default() }, Exposure { ev100: 13.2 }));
            if fx.env {
                e.insert(bevy::light::AtmosphereEnvironmentMapLight::default());
            }
        }
        if fx.taa || fx.ssao {
            e.insert(Msaa::Off);
        } else {
            e.insert(Msaa::Sample4);
        }
        if fx.taa {
            e.insert((TemporalAntiAliasing::default(), ShadowFilteringMethod::Temporal));
        } else {
            e.insert(ShadowFilteringMethod::Gaussian);
        }
        if fx.ssao {
            e.insert(ScreenSpaceAmbientOcclusion { quality_level: ScreenSpaceAmbientOcclusionQualityLevel::Medium, ..default() });
        }
    }
    #[cfg(not(any(feature = "webgpu", not(target_arch = "wasm32"))))]
    {
        // WebGL2 can't run these passes.
        let _ = (media, fx.ssao, fx.taa);
        e.insert((Msaa::Sample4, ShadowFilteringMethod::Gaussian));
    }
}

//! The smallest Bevy scene (a cube, a light, a camera): checks that a browser's WebGPU runs Bevy
//! at all, separately from the game.
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(bevy::render::RenderPlugin { render_creation: settings().into(), ..default() })
                .set(WindowPlugin { primary_window: Some(Window { canvas: Some("#game".into()), fit_canvas_to_parent: true, ..default() }), ..default() }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, |mut q: Query<&mut Transform, With<Mesh3d>>, t: Res<Time>| {
            for mut tf in &mut q {
                tf.rotation = Quat::from_rotation_y(t.elapsed_secs());
            }
        })
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))), MeshMaterial3d(mats.add(Color::srgb(0.9, 0.4, 0.2)))));
    commands.spawn((DirectionalLight::default(), Transform::from_xyz(3.0, 5.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y)));
    commands.spawn((Camera3d::default(), Transform::from_xyz(2.0, 2.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y)));
}

/// `?prio=webgpu` asks only for WebGPU's default features and limits.
fn settings() -> bevy::render::settings::WgpuSettings {
    use bevy::render::settings::{WgpuSettings, WgpuSettingsPriority};
    #[cfg(target_arch = "wasm32")]
    let q = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
    #[cfg(not(target_arch = "wasm32"))]
    let q = String::new();
    let priority = if q.contains("prio=webgpu") { WgpuSettingsPriority::WebGPU } else { WgpuSettingsPriority::Functionality };
    WgpuSettings { priority, ..default() }
}

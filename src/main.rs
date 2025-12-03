use bevy::prelude::*;
use bevy_animations::animated_character::AnimatedCharacterPlugin;
use bevy_animations::asset_loader::AssetLoaderPlugin;
use bevy_animations::camera::CameraPlugin;
use bevy_animations::level::PlanePlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 1000.0,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(PlanePlugin)
        .add_plugins(CameraPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(AssetLoaderPlugin)
        .add_plugins(AnimatedCharacterPlugin)
        .run();
}

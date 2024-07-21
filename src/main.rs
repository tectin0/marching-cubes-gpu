mod camera;

use bevy::app::App;

use bevy::prelude::*;
use camera::camera_control;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_control)
        .run();
}

#[derive(Component)]
struct CameraMarker;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        CameraMarker,
    ));

    commands.spawn(PointLight {
        ..Default::default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.7, 0.6),
            reflectance: 0.5,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

mod camera;
mod lut;
mod voxel;

use bevy::app::App;

use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;
use camera::camera_control;
use voxel::{Chunk, VoxelsPlugin};
use wgpu::PrimitiveTopology;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(VoxelsPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 100.0,
        })
        .add_systems(Startup, (setup, spawn_voxel_sys))
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

    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, 0.6),
            reflectance: 0.5,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn spawn_voxel_sys(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_indices(Indices::U32(Vec::with_capacity(4096)));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(Vec::with_capacity(4096)),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(Vec::with_capacity(4096)),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(Vec::with_capacity(4096)),
    );
    let mesh_handle = meshes.add(mesh);
    let ground_mat_handle = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        ..default()
    });

    commands.spawn((
        Chunk::new(IVec3::ZERO),
        PbrBundle {
            mesh: mesh_handle.clone(),
            material: ground_mat_handle.clone(),
            ..default()
        },
    ));
}

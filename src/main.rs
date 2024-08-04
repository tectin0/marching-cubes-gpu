mod camera;
mod lut;
mod marching_cubes_cpu;
mod marching_cubes_gpu;

use bevy::app::App;

use bevy::log::LogPlugin;
use bevy::math::primitives;
use bevy::prelude::*;
use bevy::render::mesh::{
    Indices, PlaneMeshBuilder, RhombusMeshBuilder, SphereMeshBuilder, VertexAttributeValues,
};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use bevy_rapier3d::prelude::Collider;
use bevy_rapier3d::render::RapierDebugRenderPlugin;
use camera::camera_control;
use marching_cubes_cpu::{Bounds, MarchingCubesCpuPlugin, VoxelGrid};
use marching_cubes_gpu::{Chunk, MarchingCubesGpuPlugin};
use wgpu::PrimitiveTopology;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        }))
        // .add_plugins(MarchingCubesGpuPlugin)
        .add_plugins(MarchingCubesCpuPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
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
    mut ambient_light: ResMut<AmbientLight>,
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

    ambient_light.brightness = 100.0;

    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    //     material: materials.add(StandardMaterial {
    //         base_color: Color::srgb(0.1, 0.1, 0.6),
    //         reflectance: 0.5,
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     ..Default::default()
    // });
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

    let mesh = SphereMeshBuilder {
        sphere: Sphere::new(2.0),
        kind: bevy::render::mesh::SphereKind::Uv {
            sectors: 32,
            stacks: 32,
        },
    }
    .build();

    // let mesh = Cuboid::new(2.0, 2.0, 2.0).mesh().build();

    let collider = Collider::from_bevy_mesh(
        &mesh,
        &bevy_rapier3d::prelude::ComputedColliderShape::TriMesh,
    )
    .unwrap();

    let voxel_grid = VoxelGrid::from_mesh(&mesh, [32, 32, 32]);
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
        voxel_grid,
        collider.clone(),
    ));
}

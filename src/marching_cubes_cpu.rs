use bevy::{
    app::{App, Plugin, PreUpdate},
    asset::{Assets, Handle},
    color::palettes::css::WHITE,
    input::ButtonInput,
    log::{debug, info},
    math::{IVec3, Quat, Vec3, Vec4, Vec4Swizzles},
    pbr::{PbrBundle, StandardMaterial},
    prelude::{
        default, Commands, Component, Condition, Entity, GlobalTransform, IntoSystemConfigs,
        KeyCode, Mesh, Query, Res, ResMut, Sphere, Transform,
    },
    render::mesh::{Indices, VertexAttributeValues},
    time::Time,
};
use bevy_rapier3d::prelude::{Collider, ComputedColliderShape};

use crate::{
    lut::{EDGE_TABLE, TRI_TABLE},
    marching_cubes_gpu::Chunk,
};

pub struct MarchingCubesCpuPlugin;

impl Plugin for MarchingCubesCpuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, marching_cubes_system);
    }
}

const OFFSETS: [[usize; 3]; 8] = [
    [0, 0, 1],
    [1, 0, 1],
    [1, 0, 0],
    [0, 0, 0],
    [0, 1, 1],
    [1, 1, 1],
    [1, 1, 0],
    [0, 1, 0],
];

const VERTICES_COMB: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Component)]
pub struct VoxelGrid {
    pub resolution: [usize; 3],
    pub data: Vec<f32>,
    pub bounds: Bounds,
}

impl VoxelGrid {
    pub fn from_mesh(mesh: &Mesh, resolution: [usize; 3]) -> Self {
        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;

        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        let mut z_min = f32::MAX;
        let mut z_max = f32::MIN;

        if let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            for vertex in vertices.iter() {
                x_min = x_min.min(vertex[0]);
                x_max = x_max.max(vertex[0]);

                y_min = y_min.min(vertex[1]);
                y_max = y_max.max(vertex[1]);

                z_min = z_min.min(vertex[2]);
                z_max = z_max.max(vertex[2]);
            }
        }

        x_min *= 1.1;
        x_max *= 1.1;

        y_min *= 1.1;
        y_max *= 1.1;

        z_min *= 1.1;
        z_max *= 1.1;

        let x_step = (x_max - x_min) / resolution[0] as f32;
        let y_step = (y_max - y_min) / resolution[1] as f32;
        let z_step = (z_max - z_min) / resolution[2] as f32;

        let x_steps = resolution[0];
        let y_steps = resolution[1];
        let z_steps = resolution[2];

        let collider = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap();

        let mut data = vec![0.0; resolution[0] * resolution[1] * resolution[2]];

        for zi in 0..z_steps - 1 {
            for yi in 0..y_steps - 1 {
                for xi in 0..x_steps - 1 {
                    let x = x_min + xi as f32 * x_step;
                    let y = y_min + yi as f32 * y_step;
                    let z = z_min + zi as f32 * z_step;

                    let point = Vec3::new(x, y, z);

                    let check_directions =
                        [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z];

                    let mut is_check_num = 0;

                    for direction in check_directions.iter() {
                        if collider.intersects_local_ray(point, *direction, 100.0) {
                            is_check_num += 1;
                        }
                    }

                    let criterion = is_check_num == 6;
                    if criterion {
                        data[zi * y_steps * x_steps + yi * x_steps + xi] = 1.0;
                    }
                }
            }
        }

        let bounds = Bounds {
            min: Vec3::new(x_min, y_min, z_min),
            max: Vec3::new(x_max, y_max, z_max),
        };

        VoxelGrid {
            resolution,
            data,
            bounds,
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> f32 {
        self.data[z * self.resolution[1] * self.resolution[0] + y * self.resolution[0] + x]
    }
}

pub fn marching_cubes_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Handle<StandardMaterial>,
        &Handle<Mesh>,
        &GlobalTransform,
        &mut Transform,
        &VoxelGrid,
        &Collider,
        &mut Chunk,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if !keyboard_input.just_pressed(KeyCode::Enter) {
        return;
    }

    let points_sphere = meshes.add(Mesh::from(Sphere::new(0.05)));

    debug!("Running marching cubes");

    for (
        entity,
        material_handle,
        mesh_handle,
        global_transform,
        transform,
        voxel_grid,
        collider,
        _,
    ) in query.iter_mut()
    {
        debug!("Running marching cubes for entity {:?}", entity);

        let mesh = meshes.get_mut(mesh_handle).unwrap();

        let Bounds { min, max } = voxel_grid.bounds;

        let x_steps = voxel_grid.resolution[0];
        let y_steps = voxel_grid.resolution[1];
        let z_steps = voxel_grid.resolution[2];

        let x_step = (max[0] - min[0]) / x_steps as f32;
        let y_step = (max[1] - min[1]) / y_steps as f32;
        let z_step = (max[2] - min[2]) / z_steps as f32;

        for xi in 0..voxel_grid.resolution[0] {
            for yi in 0..voxel_grid.resolution[1] {
                for zi in 0..voxel_grid.resolution[2] {
                    let x = min[0] + xi as f32 * x_step;
                    let y = min[1] + yi as f32 * y_step;
                    let z = min[2] + zi as f32 * z_step;

                    let value = voxel_grid.get(xi, yi, zi);

                    if value > 0.0 {}
                }
            }
        }

        let mut vert_counter = 0;

        let mut new_vertices = Vec::new();
        let mut new_indices = Vec::new();
        let mut new_normals = Vec::new();
        let mut new_uvs = Vec::new();

        let mut points_inside = 0;

        for zi in 0..z_steps - 1 {
            for yi in 0..y_steps - 1 {
                for xi in 0..x_steps - 1 {
                    let mut cube_index = 0b0000_0000;

                    let position_values = OFFSETS
                        .iter()
                        .map(|offset| {
                            let x = min[0] + (xi + offset[0]) as f32 * x_step;
                            let y = min[1] + (yi + offset[1]) as f32 * y_step;
                            let z = min[2] + (zi + offset[2]) as f32 * z_step;
                            let value =
                                voxel_grid.get(xi + offset[0], yi + offset[1], zi + offset[2]);

                            Vec4::new(x, y, z, value)
                        })
                        .collect::<Vec<Vec4>>();

                    for (index, position) in position_values.iter().enumerate() {
                        // let criterion = position.x * position.x
                        //     + position.y * position.y
                        //     + position.z * position.z
                        //     - 1.0
                        //     > 0.0;

                        let criterion = position.w > 0.0;

                        if criterion {
                            points_inside += 1;
                        }

                        cube_index = cube_index | (criterion as u32) * (1 << index);
                    }

                    if cube_index == 0x00 || cube_index == 0xff {
                        continue;
                    }

                    let triangulation = TRI_TABLE[cube_index as usize];

                    let vertices = (0..12)
                        .map(|index| {
                            let edge = ((EDGE_TABLE[cube_index as usize] & (1 << index)) != 0)
                                as i32 as f32;

                            edge * interp_vertex(
                                position_values[VERTICES_COMB[index][0]].xyz(),
                                position_values[VERTICES_COMB[index][1]].xyz(),
                                0.5,
                                -0.5,
                            )
                        })
                        .collect::<Vec<Vec3>>();

                    for tri_idx in (0..triangulation.len()).step_by(3) {
                        if triangulation[tri_idx] == -1 {
                            break;
                        }

                        let v0 = vertices[triangulation[tri_idx] as usize];
                        let v1 = vertices[triangulation[tri_idx + 1] as usize];
                        let v2 = vertices[triangulation[tri_idx + 2] as usize];

                        let normal = (v1 - v0).cross(v2 - v0).normalize();

                        new_vertices.push([v0.x, v0.y, v0.z]);
                        new_vertices.push([v1.x, v1.y, v1.z]);
                        new_vertices.push([v2.x, v2.y, v2.z]);

                        new_indices.extend([vert_counter, vert_counter + 1, vert_counter + 2]);

                        new_normals.push([normal.x, normal.y, normal.z]);
                        new_normals.push([normal.x, normal.y, normal.z]);
                        new_normals.push([normal.x, normal.y, normal.z]);

                        new_uvs.push([0.0, 0.0]);
                        new_uvs.push([1.0, 0.0]);
                        new_uvs.push([0.0, 1.0]);

                        vert_counter += 3;
                    }
                }
            }
        }

        let vertex_count = new_vertices.len();

        debug!("Calculated vertices: {}", vertex_count);
        debug!("Points inside: {}", points_inside);

        if let Some(VertexAttributeValues::Float32x3(vertices)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            vertices.clear();
            vertices.extend(new_vertices);
        }

        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            indices.clear();
            indices.extend(new_indices);
        }

        if let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
        {
            normals.clear();
            normals.reserve(vertex_count);
            normals.extend(new_normals);
        }
        if let Some(VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            uvs.clear();
            uvs.reserve(vertex_count);
            uvs.extend(new_uvs);
        }

        commands
            .entity(entity)
            .insert(Collider::from_bevy_mesh(mesh, &ComputedColliderShape::TriMesh).unwrap());

        debug!("Marching cubes done");
    }
}

fn interp_vertex(p1: Vec3, p2: Vec3, val1: f32, val2: f32) -> Vec3 {
    let t = (0.0 - val1) / (val2 - val1);
    p1 + (p2 - p1) * t
}

use bevy::ecs::ResMut;
use bevy::prelude::Plugin;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::{pipeline::PrimitiveTopology, render_graph::base::MainPass};
use bevy::{asset::Assets, ecs::Query, prelude::Handle};
use bevy_mod_picking::InteractableMesh;
use bevy_mod_picking::PickableMesh;
use bevy_rapier3d::{
    physics::{ColliderHandleComponent, RigidBodyHandleComponent},
    rapier::{
        dynamics::RigidBodySet,
        geometry::{ColliderBuilder, ColliderSet},
        math::{Point, Real},
    },
};
use pipeline::setup_marching_mesh_pipeline;
use pipeline::MarchMeshMaterial;
use pipeline::ATTRIBUTE_POINT_DATA;
use stage::POST_UPDATE;

use crate::triangulation::{self, triangulation};

pub mod pipeline;

#[derive(Clone)]
pub struct Chunk {
    pub data: Box<Vec<Vec<Vec<f32>>>>,
}

impl Default for Chunk {
    fn default() -> Self {
        Chunk {
            data: Box::new(vec![vec![Vec::new()]]),
        }
    }
}

#[derive(Default)]
pub struct ChunkSettings {
    pub length: usize,
    pub width: usize,
    pub height: usize,
    pub threshold: f32,
}

#[derive(Default, Bundle)]
pub struct MarchingChunkBundle {
    pub chunk: Chunk,
    pub mesh: Handle<Mesh>,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub main_pass: MainPass,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

fn regen_mesh(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut collider_set: ResMut<ColliderSet>,
    mut bodies: ResMut<RigidBodySet>,
    chunk_settings: Res<ChunkSettings>,
    mesh_query: Query<
        (
            &Chunk,
            &Handle<Mesh>,
            &ColliderHandleComponent,
            &RigidBodyHandleComponent,
            Entity,
        ),
        Changed<Chunk>,
    >,
    interactable_query: Query<(&PickableMesh, &InteractableMesh)>,
) {
    for (chunk, mesh_handle, collider_handle, rigid_body_handle, entity) in mesh_query.iter() {
        let mesh = meshes.get_mut(mesh_handle).unwrap();
        let (v_pos, normals, p_data, indices) = generate_mesh(&chunk_settings, &chunk);
        let mut collider_verts: Vec<Point<Real>> = Vec::new();
        let mut collider_indicies: Vec<[u32; 3]> = Vec::new();

        for pos in v_pos.iter() {
            collider_verts.push(Point::new(pos[0], pos[1], pos[2]));
        }

        for i in (0..indices.len()).step_by(3) {
            collider_indicies.push([indices[i], indices[i + 1], indices[i + 2]]);
        }

        collider_set.remove(collider_handle.handle(), &mut bodies, false);
        let new_handle = collider_set.insert(
            ColliderBuilder::trimesh(
                collider_verts, 
                collider_indicies
            ).build(),
            rigid_body_handle.handle(),
            &mut bodies,
        );
        commands.remove_one::<ColliderHandleComponent>(entity);
        commands.set_current_entity(entity);
        commands.with(ColliderHandleComponent::from(new_handle));

        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float3(v_pos),
        );

        mesh.set_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float3(normals),
        );

        mesh.set_attribute(ATTRIBUTE_POINT_DATA, VertexAttributeValues::Float(p_data));

        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

        let interactable_query_result = interactable_query.get(entity);

        match interactable_query_result {
            Ok(_) => {}
            Err(query_error) => match query_error {
                bevy::ecs::QueryError::NoSuchEntity => {
                    commands.insert_one(entity, PickableMesh::default());
                    commands.insert_one(entity, InteractableMesh::default());
                }
                _ => {}
            },
        }
    }
}

pub fn normalize_f32(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

pub fn generate_mesh(
    chunk_settings: &ChunkSettings,
    chunk: &Chunk,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<f32>, Vec<u32>) {
    let mut v_pos: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut p_data: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..chunk_settings.height - 1 {
        for x in 0..chunk_settings.width - 1 {
            for z in 0..chunk_settings.length - 1 {
                let temp_x = x as f32;
                let temp_y = y as f32;
                let temp_z = z as f32;

                let point_pos = [
                    [temp_x, temp_y, temp_z + 1f32],
                    [temp_x + 1f32, temp_y, temp_z + 1f32],
                    [temp_x + 1f32, temp_y, temp_z],
                    [temp_x, temp_y, temp_z],
                    [temp_x, temp_y + 1f32, temp_z + 1f32],
                    [temp_x + 1f32, temp_y + 1f32, temp_z + 1f32],
                    [temp_x + 1f32, temp_y + 1f32, temp_z],
                    [temp_x, temp_y + 1f32, temp_z],
                ];

                let point_data = [
                    chunk.data[x][y][z + 1],         // 0
                    chunk.data[x + 1][y][z + 1],     // 1
                    chunk.data[x + 1][y][z],         // 2
                    chunk.data[x][y][z],             // 3
                    chunk.data[x][y + 1][z + 1],     // 4
                    chunk.data[x + 1][y + 1][z + 1], // 5
                    chunk.data[x + 1][y + 1][z],     // 6
                    chunk.data[x][y + 1][z],         // 7
                ];

                let mut cube_ndex: usize = 0;
                for i in 0..8 {
                    if point_data[i] > chunk_settings.threshold {
                        cube_ndex |= 1 << (i as usize);
                    }
                }

                let triang = triangulation::triangulation[cube_ndex];

                let mut tri_verts: Vec<Vec3> = Vec::new();

                for edge_index in triang.iter() {
                    if *edge_index != 10000 {
                        let index_a = triangulation::cornerIndexAFromEdge[*edge_index];
                        let index_b = triangulation::cornerIndexBFromEdge[*edge_index];

                        let point_a_pos = point_pos[index_a];
                        let point_b_pos = point_pos[index_b];

                        let point_a_data = point_data[index_a];
                        let point_b_data = point_data[index_b];

                        let vec_a = Vec3::new(point_a_pos[0], point_a_pos[1], point_a_pos[2]);
                        let vec_b = Vec3::new(point_b_pos[0], point_b_pos[1], point_b_pos[2]);

                        let pos = if point_a_data > point_b_data {
                            let lerp_val = normalize_f32(
                                chunk_settings.threshold,
                                point_a_data as f32,
                                point_b_data as f32,
                            );
                            vec_a.lerp(vec_b, lerp_val)
                        } else {
                            let lerp_val = normalize_f32(
                                chunk_settings.threshold,
                                point_b_data as f32,
                                point_a_data as f32,
                            );
                            vec_b.lerp(vec_a, lerp_val)
                        };
                        // v_pos.push(pos);
                        tri_verts.push(pos);
                        p_data.push(point_a_data);
                        indices.push(p_data.len() as u32 - 1);
                    }
                }

                for i in (0..tri_verts.len()).step_by(3) {
                    let a: Vec3 = tri_verts[i + 1] - tri_verts[i];
                    let b: Vec3 = tri_verts[i + 2] - tri_verts[i];

                    let normal = a.cross(b);

                    v_pos.push(tri_verts[i].into());
                    v_pos.push(tri_verts[i + 1].into());
                    v_pos.push(tri_verts[i + 2].into());

                    normals.push(normal.into());
                    normals.push(normal.into());
                    normals.push(normal.into());
                }
            }
        }
    }

    return (v_pos, normals, p_data, indices);
}

pub struct MarchingCubesPlugin;
impl Plugin for MarchingCubesPlugin {
    fn build(&self, app: &mut bevy::prelude::AppBuilder) {
        app.add_resource(ChunkSettings {
            length: 16,
            width: 16,
            height: 40,
            threshold: 0.0,
            ..Default::default()
        })
        .add_startup_system(setup_marching_mesh_pipeline.system())
        .add_asset::<MarchMeshMaterial>()
        .add_system_to_stage(POST_UPDATE, regen_mesh.system());
    }
}

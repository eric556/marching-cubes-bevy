use crate::chunk::generate_mesh;
use crate::chunk::pipeline::MarchMeshMaterial;
use crate::chunk::pipeline::MARCHING_MESH_MAT;
use crate::chunk::Chunk;
use crate::chunk::MarchingCubesPlugin;
use bevy_4x_camera::FourXCameraPlugin;
use chunk::{pipeline::default_marching_mesh_pipeline, ChunkSettings, MarchingChunkBundle};

use bevy::{
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
    },
};
use bevy_4x_camera::CameraRigBundle;
use bevy_mod_picking::{
    DebugPickingPlugin, Group, InteractableMesh, InteractablePickingPlugin, PickSource, PickState,
    PickableMesh, PickingPlugin,
};
use triangulation::{edges, triangulation};

const WIDTH: usize = 60;
const HEIGHT: usize = 60;
const LENGTH: usize = 60;
const thershold: f32 = 0.0f32;

pub mod chunk;
pub mod triangulation;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            width: 1920f32,
            height: 1080f32,
            title: "Marching Cubes".to_string(),
            vsync: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FourXCameraPlugin)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_plugin(MarchingCubesPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(setup_march_mesh.system())
        .add_system(select_terrain.system())
        .run();
}

/// set up a simple 3D scene
fn setup(commands: &mut Commands) {
    commands
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        // camera
        .spawn(CameraRigBundle::default())
        .with_children(|cb| {
            cb.spawn(Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(-20.0, 20., 0.0))
                    .looking_at(Vec3::zero(), Vec3::unit_y()),
                ..Default::default()
            })
            .with(PickSource::default());
        });
}

fn setup_march_mesh(
    commands: &mut Commands,
    chunk_settings: Res<ChunkSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MarchMeshMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    // asset_server.watch_for_changes().unwrap();
    let pipeline_handle = pipelines.add(default_marching_mesh_pipeline(shaders));

    render_graph.add_system_node(
        MARCHING_MESH_MAT,
        AssetRenderResourcesNode::<MarchMeshMaterial>::new(true),
    );

    render_graph
        .add_node_edge(MARCHING_MESH_MAT, base::node::MAIN_PASS)
        .unwrap();

    let mut chunk = Chunk::default();
    chunk.data = Box::new(vec![
        vec![
            vec![0.0; chunk_settings.length];
            chunk_settings.height
        ];
        chunk_settings.width
    ]);

    for y in 0..chunk_settings.height {
        for x in 0..chunk_settings.width {
            for z in 0..chunk_settings.length {
                chunk.data[x][y][z] = plane(5f32, y as f32);
            }
        }
    }

    let mesh_material_handle = materials.add(MarchMeshMaterial {
        lightPos: Vec3::new(4.0, 8.0, 4.0),
        lightColor: Vec3::new(1f32, 1f32, 1f32),
        objectColor: Vec3::new(0.88, 0.32, 0.39),
    });

    // let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // let (v_pos, normals, p_data, indices) = generate_mesh(&chunk_settings, &chunk);

    // mesh.set_attribute(
    //     Mesh::ATTRIBUTE_POSITION,
    //     VertexAttributeValues::Float3(v_pos),
    // );

    // mesh.set_attribute(
    //     Mesh::ATTRIBUTE_NORMAL,
    //     VertexAttributeValues::Float3(normals),
    // );

    // mesh.set_attribute(
    //     ATTRIBUTE_POINT_DATA,
    //     VertexAttributeValues::Float(p_data),
    // );

    // mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    // let mesh_handle = meshes.add(mesh);

    commands
        .spawn(MarchingChunkBundle {
            // mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
            // mesh: mesh_handle,
            chunk: chunk,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle.clone_weak(),
            )]),
            ..Default::default()
        })
        .with(mesh_material_handle.clone_weak())
        .with(PickableMesh::default())
        .with(InteractableMesh::default());
    println!("Commanded the add");
}

fn inside_sphere(sphere_pos: Vec3, radius: f32, point: Vec3) -> bool {
    (point.x - sphere_pos.x).powf(2f32)
        + (point.y - sphere_pos.y).powf(2f32)
        + (point.z - sphere_pos.z).powf(2f32)
        < (radius * radius)
}

fn select_terrain(
    pick_state: Res<PickState>,
    mut corner_query: Query<(&InteractableMesh, &mut Chunk, Entity)>,
) {
    for (interactable, mut chunk, entity) in &mut corner_query.iter_mut() {
        let increment_event = interactable
            .mouse_down_event(&Group::default(), MouseButton::Left)
            .unwrap();

        let decrement_event = interactable
            .mouse_down_event(&Group::default(), MouseButton::Right)
            .unwrap();

        if increment_event.is_none() && decrement_event.is_none() {
            continue;
        }

        let (_, intersection) = pick_state.top(Group::default()).unwrap();
        let sphere_center = intersection.position();

        // Gen a sphere and capture chunk data in that sphere
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                for z in 0..LENGTH {
                    if inside_sphere(
                        *sphere_center,
                        3f32,
                        Vec3::new(x as f32, y as f32, z as f32),
                    ) {
                        if !increment_event.is_none() {
                            chunk.data[x][y][z] += 0.1;
                        }

                        if !decrement_event.is_none() {
                            chunk.data[x][y][z] -= 0.1;
                        }
                    }
                }
            }
        }
    }
}

fn plane(plane_y: f32, y: f32) -> f32 {
    return if y > plane_y {
        thershold + 1f32
    } else {
        thershold - 1f32
    };
}

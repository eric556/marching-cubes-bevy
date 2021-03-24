use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use bevy_rapier3d::{physics::RapierPhysicsPlugin, rapier::{dynamics::RigidBodyBuilder, geometry::ColliderBuilder, math::Real}};
use camera::third_person_camera::{FollowTarget, ThirdPerson3DCameraBundle, ThirdPersonCamera, ThirdPersonCameraPlugin};
use settings::SettingsPlugin;
use crate::chunk::pipeline::MarchMeshMaterial;
use crate::chunk::pipeline::MARCHING_MESH_MAT;
use crate::chunk::Chunk;
use crate::chunk::MarchingCubesPlugin;
use bevy_4x_camera::FourXCameraPlugin;
use chunk::{pipeline::default_marching_mesh_pipeline, ChunkSettings, MarchingChunkBundle};

use bevy::{
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
    },
};
use bevy_4x_camera::CameraRigBundle;
use bevy_mod_picking::{
    DebugPickingPlugin, Group, InteractableMesh, InteractablePickingPlugin, PickSource, PickState,
    PickingPlugin,
};
use triangulation::{edges, triangulation};

const WIDTH: usize = 60;
const HEIGHT: usize = 60;
const LENGTH: usize = 60;
const thershold: f32 = 0.0f32;

pub mod chunk;
pub mod triangulation;
pub mod camera;
pub mod settings;

struct ModifyLand{
    incrementing: bool,
    decrementing: bool,
    change_value: f32
}

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
        .add_resource(ModifyLand {
            incrementing: false,
            decrementing: false,
            change_value: 0.0
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FourXCameraPlugin)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_plugin(MarchingCubesPlugin)
        .add_plugin(RapierPhysicsPlugin)
        .add_plugin(SettingsPlugin)
        .add_plugin(ThirdPersonCameraPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(setup_march_mesh.system())
        .add_startup_system(setup_test_object.system())
        .add_system(select_terrain.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        // light
        .spawn(LightBundle {
            transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
            ..Default::default()
        })
        // camera
        // .spawn(CameraRigBundle::default())
        // .with_children(|cb| {
        //     cb.spawn(Camera3dBundle {
        //         transform: Transform::from_translation(Vec3::new(-20.0, 20., 0.0))
        //             .looking_at(Vec3::zero(), Vec3::unit_y()),
        //         ..Default::default()
        //     })
        //     .with(PickSource::default());
        // });
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere{radius: 1.0, ..Default::default()})),
            material: materials.add(Color::rgb(0.5, 0.4, 0.3).into()),
            transform: Transform::from_translation(Vec3::new(10.0, 0.0, 10.0)),
            ..Default::default()
        })
        .with(RigidBodyBuilder::new_dynamic().translation(10.0, 50.0, 10.0))
        .with(ColliderBuilder::cylinder(1.0, 1.0))
        .with(FollowTarget)
        .with_children(|parent|{
            parent.spawn(ThirdPerson3DCameraBundle {
                third_person_camera: ThirdPersonCamera {
                    distance: 40.0,
                    ..Default::default()
                },
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

    for z in 0..10 {
        for x in 0..10 {
            commands
                .spawn(MarchingChunkBundle {
                    mesh: meshes.add(Mesh::new(PrimitiveTopology::TriangleList)),
                    chunk: chunk.clone(),
                    render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                        pipeline_handle.clone_weak(),
                    )]),
                    ..Default::default()
                })
                .with(mesh_material_handle.clone_weak())
                .with(RigidBodyBuilder::new_static().translation(((chunk_settings.width - 1) * x) as Real, 0.0, ((chunk_settings.length - 1) * z) as Real))
                .with(ColliderBuilder::cuboid(1.0, 1.0, 1.0));
        }
    }
}

fn setup_test_object(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle{
        mesh: meshes.add(Mesh::from(shape::Icosphere{radius:1.0, ..Default::default()})),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        ..Default::default()
    })
    .with(RigidBodyBuilder::new_dynamic().translation(10.0, 20.0, 10.0).gravity_scale(0.5).linvel(0.0, 0.0, 10.0))
    .with(ColliderBuilder::cylinder(1.0, 1.0));
}

fn inside_sphere(sphere_pos: Vec3, radius: f32, point: Vec3) -> bool {
    (point.x - sphere_pos.x).powf(2f32)
        + (point.y - sphere_pos.y).powf(2f32)
        + (point.z - sphere_pos.z).powf(2f32)
        < (radius * radius)
}

fn select_terrain(
    pick_state: Res<PickState>,
    chunk_setting: Res<ChunkSettings>,
    mut corner_query: Query<(&InteractableMesh, &mut Chunk, &Transform, Entity)>,
) {
    for (interactable, mut chunk, transform, entity) in &mut corner_query.iter_mut() {
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
        let chunk_position = transform.translation;

        // Gen a sphere and capture chunk data in that sphere
        for y in 0..chunk_setting.height {
            for x in 0..chunk_setting.width {
                for z in 0..chunk_setting.length {
                    if inside_sphere(
                        *sphere_center,
                        3f32,
                        Vec3::new(
                            chunk_position.x + x as f32,
                            chunk_position.y + y as f32,
                            chunk_position.z + z as f32,
                        ),
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

use std::fmt::Debug;

use bevy::{
    pbr::render_graph::FORWARD_PIPELINE_HANDLE,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::VertexAttributeValues,
        pipeline::{
            BlendDescriptor, BlendFactor, BlendOperation, ColorStateDescriptor, ColorWrite,
            CompareFunction, CullMode, DepthStencilStateDescriptor, FrontFace, IndexFormat,
            PipelineDescriptor, PrimitiveTopology, RasterizationStateDescriptor, RenderPipeline,
            StencilStateDescriptor, StencilStateFaceDescriptor,
        },
        render_graph::{
            base::{self, node::MAIN_PASS},
            AssetRenderResourcesNode, RenderGraph,
        },
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
};
use bevy_mod_picking::{DebugPickingPlugin, Group, InteractableMesh, InteractablePickingPlugin, PickSource, PickState, PickableMesh, PickingPlugin};
use triangulation::{edges, triangulation};

const WIDTH: usize = 60;
const HEIGHT: usize = 60;
const LENGTH: usize = 60;
const thershold: f32 = 0.0f32;

const DEBUG_CORNER_MAT: &str = "debug_corner_mat";
const MARCHING_MESH_MAT: &str = "marching_mesh_mat";
const ATTRIBUTE_POINT_DATA: &str = "Vertex_Data";

pub mod triangulation;

struct Chunk {
    pub data: Box<[[[f32; LENGTH]; HEIGHT]; WIDTH]>,
}

struct Corner {
    x: usize,
    y: usize,
    z: usize,
}

impl Debug for Corner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        f.debug_struct("Corner")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .finish()
    }
}

#[derive(Default)]
struct MarchMeshResource {
    pub pipeline_handle: Handle<PipelineDescriptor>,
    pub mesh_handle: Handle<Mesh>,
    pub mesh_material_handle: Handle<MarchMeshMaterial>,
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "3bf9e364-f29d-4d6c-92cf-93298466c620"]
struct DebugCornerMaterial {
    pub value: f32,
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "3bf9e364-f29d-4d6c-92cf-93298466c500"]
struct MarchMeshMaterial {
    pub lightPos: Vec3,
    pub lightColor: Vec3,
    pub objectColor: Vec3
}

fn main() {
        App::build()
        .add_resource(Chunk {
            data: Box::new([[[0.0f32; LENGTH]; HEIGHT]; WIDTH]),
        })
        .add_resource(Msaa { samples: 4 })
        .add_resource(WindowDescriptor {
            title: "Marching Cubes".to_string(),
            vsync: true,
            ..Default::default()
        })
        .init_resource::<MarchMeshResource>()
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(DebugPickingPlugin)
        .add_asset::<DebugCornerMaterial>()
        .add_asset::<MarchMeshMaterial>()
        .add_startup_system(setup.system())
        // .add_startup_system(spawn_debug_points.system())
        .add_startup_system(setup_march_mesh.system())
        // .add_system(select_corner.system())
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
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(-20.0, 20., 0.0))
                .looking_at(Vec3::new(WIDTH as f32/2.0, 0.0, LENGTH as f32/2.0), Vec3::unit_y()),
            ..Default::default()
        })
        .with(PickSource::default());
}

fn setup_march_mesh(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MarchMeshMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
    mut marching_mesh_res: ResMut<MarchMeshResource>,
    mut chunk: ResMut<Chunk>,
) {
    // asset_server.watch_for_changes().unwrap();

    let descriptor = PipelineDescriptor {
        name: None,
        primitive_topology: PrimitiveTopology::TriangleList,
        layout: None,
        index_format: IndexFormat::Uint32,
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
        rasterization_state: Some(RasterizationStateDescriptor {
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        depth_stencil_state: Some(DepthStencilStateDescriptor {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilStateDescriptor {
                front: StencilStateFaceDescriptor::IGNORE,
                back: StencilStateFaceDescriptor::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
        }),
        color_states: vec![ColorStateDescriptor {
            format: TextureFormat::default(),
            color_blend: BlendDescriptor {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        shader_stages: ShaderStages {
            vertex: asset_server.load::<Shader, _>("shaders/marching_mesh.vert"),
            fragment: Some(asset_server.load::<Shader, _>("shaders/marching_mesh.frag")),
        },
    };

    marching_mesh_res.pipeline_handle = pipelines.add(descriptor);

    render_graph.add_system_node(
        MARCHING_MESH_MAT,
        AssetRenderResourcesNode::<MarchMeshMaterial>::new(true),
    );

    render_graph
        .add_node_edge(MARCHING_MESH_MAT, base::node::MAIN_PASS)
        .unwrap();

    let mut mesh = Mesh::new(bevy::render::pipeline::PrimitiveTopology::TriangleList);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            for z in 0..LENGTH {
                // chunk.data[x][y][z] = if noise.get_value(x, y) as f32 > y as f32 { thershold + 1f32} else {thershold - 1f32};
                // chunk.data[x][y][z] = noise.get([x as f64, y as f64, z as f64]) as f32;
                // chunk.data[x][y][z] = other(x as f32, y as f32, z as f32);
                chunk.data[x][y][z] = plane(5f32, y as f32);
                // chunk.data[x][y][z] = sphere(x as f32, y as f32, z as f32);

            }
        }
    }

    let (v_pos, normals, p_data, indices) = generate_mesh(&chunk);

    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(v_pos),
    );

    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float3(normals),
    );

    mesh.set_attribute(
        ATTRIBUTE_POINT_DATA,
        VertexAttributeValues::Float(p_data),
    );

    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));

    marching_mesh_res.mesh_handle = meshes.add(mesh);

    marching_mesh_res.mesh_material_handle = materials.add(MarchMeshMaterial {
        lightPos: Vec3::new(4.0, 8.0, 4.0),
        lightColor: Vec3::new(1f32, 1f32, 1f32),
        objectColor: Vec3::new(0.88, 0.32, 0.39)
    });

    commands
        .spawn(MeshBundle {
            mesh: marching_mesh_res.mesh_handle.clone_weak(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                marching_mesh_res.pipeline_handle.clone_weak(),
            )]),
            ..Default::default()
        })
        .with(marching_mesh_res.mesh_material_handle.clone_weak())
        .with(PickableMesh::default())
        .with(InteractableMesh::default());
}


fn inside_sphere(sphere_pos: Vec3, radius: f32, point: Vec3) -> bool{
    (point.x - sphere_pos.x).powf(2f32) + (point.y - sphere_pos.y).powf(2f32) + (point.z - sphere_pos.z).powf(2f32) < (radius * radius)
}

fn select_terrain(
    mut chunk: ResMut<Chunk>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_corner_materials: ResMut<Assets<DebugCornerMaterial>>,
    pick_state: Res<PickState>,
    marching_mesh_res: Res<MarchMeshResource>,
    corner_query: Query<&InteractableMesh>,
) {
    for interactable in &mut corner_query.iter() {
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
                    if inside_sphere(*sphere_center, 3f32, Vec3::new(x as f32, y as f32, z as f32))
                    {
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

        let (v_pos, normals, p_data, indices) = generate_mesh(&chunk);
        let mesh_option = meshes.get_mut(marching_mesh_res.mesh_handle.clone_weak());
        match mesh_option {
            Some(mesh) => {
                let len = indices.len();

                mesh.set_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    VertexAttributeValues::Float3(v_pos),
                );

                mesh.set_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    VertexAttributeValues::Float3(normals),
                );

                mesh.set_attribute(
                    ATTRIBUTE_POINT_DATA,
                    VertexAttributeValues::Float(p_data),
                );

                mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
            }
            None => {}
        }
    }

}

fn spawn_debug_points(
    commands: &mut Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DebugCornerMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
    mut chunk: ResMut<Chunk>,
) {
    // asset_server.watch_for_changes().unwrap();

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/debug_corner.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/debug_corner.frag")),
    }));

    render_graph.add_system_node(
        DEBUG_CORNER_MAT,
        AssetRenderResourcesNode::<DebugCornerMaterial>::new(true),
    );

    render_graph
        .add_node_edge(DEBUG_CORNER_MAT, base::node::MAIN_PASS)
        .unwrap();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            for z in 0..LENGTH {
                chunk.data[x][y][z] = sphere(x as f32, y as f32, z as f32).clamp(-10f32, 10f32);

                let material = materials.add(DebugCornerMaterial {
                    value: chunk.data[x][y][z],
                });

                commands
                    .spawn(MeshBundle {
                        mesh: meshes.add(Mesh::from(shape::Icosphere {
                            radius: 0.06f32,
                            ..Default::default()
                        })),
                        render_pipelines: RenderPipelines::from_pipelines(vec![
                            RenderPipeline::new(pipeline_handle.clone_weak()),
                        ]),
                        transform: Transform::from_translation(Vec3::new(
                            x as f32, y as f32, z as f32,
                        )),
                        ..Default::default()
                    })
                    .with(material)
                    .with(PickableMesh::default())
                    .with(InteractableMesh::default())
                    .with(Corner { x, y, z });
            }
        }
    }
}

fn select_corner(
    mut chunk: ResMut<Chunk>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut debug_corner_materials: ResMut<Assets<DebugCornerMaterial>>,
    marching_mesh_res: Res<MarchMeshResource>,
    corner_query: Query<(&InteractableMesh, &Corner, &Handle<DebugCornerMaterial>)>,
) {
    for (interactable, corner, debug_corner_mat_handle) in &mut corner_query.iter() {
        let increment_corner_event = interactable
            .mouse_down_event(&Group::default(), MouseButton::Left)
            .unwrap();

        let decrement_corner_event = interactable
            .mouse_down_event(&Group::default(), MouseButton::Right)
            .unwrap();

        if increment_corner_event.is_none() && decrement_corner_event.is_none() {
            continue;
        }

        match increment_corner_event {
            bevy_mod_picking::MouseDownEvents::MouseJustPressed => {
                chunk.data[corner.x][corner.y][corner.z] += 0.1;
            }
            _ => {}
        }

        match decrement_corner_event {
            bevy_mod_picking::MouseDownEvents::MouseJustPressed => {
                chunk.data[corner.x][corner.y][corner.z] -= 0.1;
            }
            _ => {}
        }

        let mat_opt = debug_corner_materials.get_mut(debug_corner_mat_handle);

        match mat_opt {
            Some(mat) => {
                mat.value = chunk.data[corner.x][corner.y][corner.z];
            }
            None => {}
        }

        let (v_pos, normals, p_data, indices) = generate_mesh(&chunk);
        let uvs = v_pos.clone();

        let mesh_option = meshes.get_mut(marching_mesh_res.mesh_handle.clone_weak());
        match mesh_option {
            Some(mesh) => {
                let len = indices.len();

                mesh.set_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    VertexAttributeValues::Float3(v_pos),
                );

                mesh.set_attribute(
                    Mesh::ATTRIBUTE_NORMAL,
                    VertexAttributeValues::Float3(normals),
                );

                mesh.set_attribute(
                    ATTRIBUTE_POINT_DATA,
                    VertexAttributeValues::Float(p_data),
                );

                mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
            }
            None => {}
        }

        // println!("DATA: {}",  chunk.data[corner.x][corner.y][corner.z]);
    }
}

fn sphere(x: f32, y: f32, z: f32) -> f32 {
    return x * x + y * y + z * z - 100.0f32;
}

fn other(x: f32, y: f32, z: f32) -> f32 {
    return (x * y + x * z + y * z).sin() + (x * y).sin() + (y * z).sin() + (x * z).sin() - 1.0f32;
}

fn plane(plane_y: f32, y: f32) -> f32 {
    return if y > plane_y { thershold + 1f32 } else {thershold - 1f32};
}


pub fn normalize_f32(value: f32, min: f32, max: f32) -> f32 {
    return (value - min) / (max - min);
}

fn generate_mesh(chunk: &Chunk) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<f32>, Vec<u32>) {
    let mut v_pos: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut p_data: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..HEIGHT - 1 {
        for x in 0..WIDTH - 1 {
            for z in 0..LENGTH - 1 {
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
                    if point_data[i] > thershold {
                        cube_ndex |= 1 << (i as usize);
                    }
                }

                let triang = triangulation::triangulation[cube_ndex];

                let mut tri_verts: Vec<Vec3> = Vec::new();
                let tri_data_points: Vec<f32> = Vec::new();

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
                                thershold as f32,
                                point_a_data as f32,
                                point_b_data as f32,
                            );
                            vec_a.lerp(vec_b, lerp_val)
                        } else {
                            let lerp_val = normalize_f32(
                                thershold as f32,
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

                if tri_verts.len() != 0 && tri_verts.len() % 3 != 0 {
                    println!("Tri verts: {}", tri_verts.len());
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

use bevy::render::render_graph::{AssetRenderResourcesNode, RenderGraph, base};
use bevy::render::mesh::Mesh;
use bevy::render::pipeline::ColorStateDescriptor;
use bevy::render::pipeline::{CullMode, FrontFace, RasterizationStateDescriptor};
use bevy::render::pipeline::{IndexFormat, PipelineDescriptor, PrimitiveTopology};
use bevy::render::pipeline::{StencilStateDescriptor, StencilStateFaceDescriptor};
use bevy::render::{
    pipeline::{CompareFunction, DepthStencilStateDescriptor},
    renderer::RenderResources,
    texture::TextureFormat,
};
use bevy::{
    math::Vec3,
    ecs::ResMut,
    prelude::{Assets, Shader},
    reflect::TypeUuid,
    render::{
        pipeline::{BlendDescriptor, BlendFactor, BlendOperation, ColorWrite},
        shader::{ShaderStage, ShaderStages},
    },
};

use super::ChunkSettings;

pub const MARCHING_MESH_MAT: &str = "marching_mesh_mat";
pub const ATTRIBUTE_POINT_DATA: &str = "Vertex_Data";

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "3bf9e364-f29d-4d6c-92cf-93298466c500"]
pub struct MarchMeshMaterial {
    pub lightPos: Vec3,
    pub lightColor: Vec3,
    pub objectColor: Vec3,
}

pub fn default_marching_mesh_pipeline(mut shaders: ResMut<Assets<Shader>>) -> PipelineDescriptor {
    PipelineDescriptor {
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
            vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
            fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
        },
    }
}

pub fn setup_marching_mesh_pipeline(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
    mut chunk_settings: ResMut<ChunkSettings>,
    mut shaders: ResMut<Assets<Shader>>
) {
    // chunk_settings.pipeline_handle = pipelines.add(default_marching_mesh_pipeline(shaders));

    // render_graph.add_system_node(
    //     MARCHING_MESH_MAT,
    //     AssetRenderResourcesNode::<MarchMeshMaterial>::new(true),
    // );

    // render_graph
    //     .add_node_edge(MARCHING_MESH_MAT, base::node::MAIN_PASS)
    //     .unwrap();
}

const VERTEX_SHADER: &str = r#"
#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in float Vertex_Data;

layout(location = 0) out vec3 normal;
layout(location = 1) out vec3 frag_pos;
layout(location = 2) out float data;


layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    data = Vertex_Data;
    normal = mat3(transpose(inverse(Model))) * Vertex_Normal;  
    frag_pos = vec3(Model * vec4(Vertex_Position, 1.0));
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 450

layout(location = 0) out vec4 o_Target;

layout(location = 0) in vec3 normal;
layout(location = 1) in vec3 frag_pos;
layout(location = 2) out float data;


layout(set = 2, binding = 0) uniform MarchMeshMaterial_lightColor {
    vec3 lightColor;
};

layout(set = 2, binding = 1) uniform MarchMeshMaterial_objectColor {
    vec3 objectColor;
};

layout(set = 2, binding = 3) uniform MarchMeshMaterial_lightPos {
    vec3 lightPos;
};

void main() {
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
  	
    // diffuse 
    vec3 norm = normalize(normal);
    vec3 lightDir = normalize(lightPos);
    float diff = max(dot(norm, lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
            
    vec3 result = (ambient + diffuse) * normalize(frag_pos);
    o_Target = vec4(result, 1.0);
}
"#;

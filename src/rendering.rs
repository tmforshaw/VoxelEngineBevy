use bevy::{
    prelude::*,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

const CHUNK_VERTEX_SHADER: &str = "shaders/chunk.wgsl";
const CHUNK_FRAGMENT_SHADER: &str = "shaders/chunk.wgsl";

// A "high" random id should be used for custom attributes to ensure consistent sorting and avoid collisions with other attributes.
// See the MeshVertexAttribute docs for more info.
pub const ATTRIBUTE_VOXEL: MeshVertexAttribute =
    MeshVertexAttribute::new("Voxel", 696969696, VertexFormat::Uint32);

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<ChunkMaterial>::default());
    }
}

#[derive(Resource, Reflect)]
pub struct GlobalChunkMaterial(pub Handle<ChunkMaterial>);

#[derive(Asset, Reflect, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub reflectance: f32,
    #[uniform(0)]
    pub perceptual_roughness: f32,
    #[uniform(0)]
    pub metallic: f32,
}

impl Material for ChunkMaterial {
    fn vertex_shader() -> ShaderRef {
        CHUNK_VERTEX_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        CHUNK_FRAGMENT_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[ATTRIBUTE_VOXEL.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

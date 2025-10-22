use std::sync::Arc;

use crate::rhi::{device::Device, queue::Queue};

pub struct TextureHandle;

pub struct MeshHandle;

pub struct SceneHandle;

pub struct RenderPassHandle;

pub struct PipelineHandle;

pub struct ShaderHandle;

pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8Srgb,
    Rgba16Unorm,
    Rgba16Srgb,
}

pub struct TextureDescriptor {
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
}

pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub struct MeshDescriptor {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

pub struct SceneDescriptor {
    pub meshes: Vec<MeshHandle>,
}

pub struct RenderPassDescriptor {
    pub color_attachments: Vec<TextureHandle>,
}

pub trait RenderContext: Send + Sync {
    fn create_texture(&self, descriptor: TextureDescriptor) -> TextureHandle;
    fn create_mesh(&self, descriptor: MeshDescriptor) -> MeshHandle;
    fn create_scene(&self) -> SceneHandle;
}



// pub struct RenderScene {
//     #[input]
//     pub 
// }
use std::sync::Arc;

use vislum_math::Vector3;
use wgpu::PipelineCache;

use crate::{device::RenderDevice, pipeline::RenderPipeline, resource::ResourceId};

pub struct Color;

pub enum MaterialAttachment {
    Vector3(String, Vector3),
    Texture(String, ResourceId<()>)
}

pub struct Material {
    attachments: Vec<MaterialAttachment>,
}

pub struct MaterialManager {
    device: RenderDevice,
}

pub struct MaterialDescriptor {
    
}

impl MaterialManager {
    pub fn new(device: RenderDevice) -> Self {
        Self { device }
    }

    pub fn create(
        &mut self, 
        render_pipeline_manager: &mut PipelineCache,
        descriptor: &MaterialDescriptor,
    ) -> Arc<Material> {
        todo!()
    }
}
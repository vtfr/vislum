use std::collections::HashMap;
use std::sync::Arc;

use vislum_system::Resource;

use crate::cache::storage::ResourceId;
use crate::resource::shader::ShaderModule;
use crate::cache::types::RenderDevice;

crate::create_atomic_id! {
    pub struct RenderPipelineId;
}

struct RenderPipelineInner {
    /// The key of the render pipeline.
    key: RenderPipelineKey,

    /// The inner wgpu render pipeline.
    pipeline: wgpu::RenderPipeline,
}

/// A render pipeline.
pub struct RenderPipeline(Arc<RenderPipelineInner>);

impl RenderPipeline {
    /// Gets the inner wgpu render pipeline.
    pub fn wgpu_pipeline(&self) -> &wgpu::RenderPipeline {
        &self.0.pipeline
    }
}

/// A key for a render pipeline.
///
/// This key is used to create a render pipeline.
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct RenderPipelineKey {
    pub vertex_shader: ResourceId<ShaderModule>,
    pub fragment_shader: ResourceId<ShaderModule>,
}

/// A cache for render pipelines.
/// 
/// This cache is used to store render pipelines that are created.
/// 
/// It is used to avoid creating the same render pipeline multiple times.
#[derive(Resource)]
pub struct RenderPipelineCache {
    device: RenderDevice,
    pipelines: HashMap<RenderPipelineKey, RenderPipeline>,
}

impl RenderPipelineCache {
    pub fn new(device: RenderDevice) -> Self {
        Self {
            device,
            pipelines: Default::default(),
        }
    }

    pub fn get(&self, key: RenderPipelineKey) -> Option<&RenderPipeline> {
        self.pipelines.get(&key)
    }
}

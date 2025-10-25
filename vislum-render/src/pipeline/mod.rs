use std::sync::Arc;

use crate::pipeline::shader::ShaderCache;

pub mod shader;

pub struct PipelineManager {
    shader_cache: Arc<ShaderCache>,
}

impl PipelineManager {
    pub fn new(shader_cache: Arc<ShaderCache>) -> Self {
        Self { shader_cache }
    }
}

static_assertions::assert_impl_all!(PipelineManager: Send, Sync);
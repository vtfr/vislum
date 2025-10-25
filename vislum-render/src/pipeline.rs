use std::{collections::HashMap, sync::{Arc, RwLock}};

use serde::{Deserialize, Serialize};
use vulkano::{device::Device, pipeline::graphics::GraphicsPipelineCreateInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineId(u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shader {
    pub source: Source,
    pub entry_point: String,
}

/// The source code of a shader.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    Hlsl(String),
}

/// The vertex shader of a graphics pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexShaderStage {
    pub shader: Shader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentShaderStage {
    pub shader: Shader,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsPipelineDescription {
    pub vertex_shader: VertexShaderStage,
    pub fragment_shader: Option<FragmentShaderStage>,
}

pub struct GraphicsPipeline {
    pipeline: vulkano::pipeline::GraphicsPipeline,
}

pub(crate) struct PipelineManager {
    device: Arc<Device>,
    pipelines: HashMap<PipelineId, Arc<GraphicsPipeline>>,
}

impl PipelineManager {
    pub fn create(&mut self, descriptor: GraphicsPipelineDescription) -> PipelineId {
        let pipeline_create_info = GraphicsPipelineCreateInfo {
            ..Default::default()
        };

        let pipeline = vulkano::pipeline::GraphicsPipeline::new(self.device, None, pipeline_create_info);
        let id = PipelineId(self.pipelines.len() as u32);
        self.pipelines.insert(id, pipeline);
        id
    }

    /// Warmup the pipeline manager by creating all the pipelines based on their descriptions.
    pub fn warmup(&mut self, descriptions: impl Iterator<Item=GraphicsPipelineDescription>) {
        for description in descriptions {
            self.create(description);
        }
    }

}
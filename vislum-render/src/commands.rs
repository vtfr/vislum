use std::{ops::Range, sync::Arc};

use ash::vk;

use crate::rhi;

pub struct CommandBufferPool {
    inner: Arc<rhi::command::CommandPool>,
}

/// A command to be executed by the GPU
enum Command {
    BindGraphicsPipeline {
        pipeline: vk::Pipeline,
    },
    Draw {
        vertices: Range<u32>,
        instances: Range<u32>,
    },
    BeginRendering {
        rendering_info: Box<vk::RenderingInfo<'static>>,
    },
    EndRendering,
    SetCullMode {
        cull_mode: vk::CullModeFlags,
    },
    SetFrontFace {
        front_face: vk::FrontFace,
    },
    SetPrimitiveTopology {
        topology: vk::PrimitiveTopology,
    },
    SetViewportWithCount {
        viewports: Box<[vk::Viewport]>,
    },
}

/// A command buffer being recorded.
struct RecordingCommandBuffer {
    commands: Vec<Command>,
}

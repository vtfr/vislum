use std::{borrow::Cow, fmt::Debug, sync::Arc};

use smallvec::SmallVec;
use ash::vk;
use vislum_render_rhi::{
    command::{CommandPool, AutoCommandBuffer},
    device::Device,
    queue::Queue,
    image::Image,
    memory::MemoryAllocator,
    sync::{Fence, Semaphore},
};

use crate::{
    graph::{CommandEncoder, ResourceStateTracker},
    resource::{ResourceManager, mesh::Mesh, pool::ResourceId, texture::Texture},
};

#[derive(Debug)]
pub enum FramePassResource {
    Texture(ResourceId<Texture>),
    Mesh(ResourceId<Mesh>),
    Surface,
}

/// Context for preparing a frame graph node.
///
/// Contains all the collected resources.
pub struct PrepareContext<'a> {
    resource_manager: &'a ResourceManager,
    write: SmallVec<[FramePassResource; 16]>,
    read: SmallVec<[FramePassResource; 16]>,
}

impl<'a> PrepareContext<'a> {
    pub fn new(resource_manager: &'a ResourceManager) -> Self {
        Self {
            resource_manager,
            write: Default::default(),
            read: Default::default(),
        }
    }

    pub fn read_texture(&mut self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.read.push(FramePassResource::Texture(id));
        self.resource_manager.resolve_texture_image(id)
    }
    
    pub fn write_texture(&mut self, id: ResourceId<Texture>) -> Option<Arc<Image>> {
        self.write.push(FramePassResource::Texture(id));
        self.resource_manager.resolve_texture_image(id)
    }
}

/// The conte
pub struct ExecuteContext<'g> {
    /// The command encoder to use for executing the pass.
    pub command_encoder: CommandEncoder<'g>,
}

type ExecuteFn = Box<dyn FnMut(&mut ExecuteContext<'_>) + 'static>;

pub trait FrameNode {
    /// The name of the node.
    fn name(&self) -> Cow<'static, str>;

    /// Prepares the node for execution.
    fn prepare(&self, context: &mut PrepareContext) -> ExecuteFn;
}

pub struct PreparedFrameNode {
    name: Cow<'static, str>,
    execute: ExecuteFn,
    write: SmallVec<[FramePassResource; 16]>,
    read: SmallVec<[FramePassResource; 16]>,
}

impl PreparedFrameNode {
    /// Returns the name of the node.
    #[inline]
    pub fn name(&self) -> &str {
        &*self.name
    }

    /// Executes the node.
    #[inline]
    pub fn execute(&mut self, ctx: &mut ExecuteContext) {
        (self.execute)(ctx);
    }

    /// Returns an iterator over the write resources.
    pub fn write(&self) -> impl ExactSizeIterator<Item = &FramePassResource> {
        self.write.iter()
    }

    /// Returns an iterator over the read resources.
    pub fn read(&self) -> impl ExactSizeIterator<Item = &FramePassResource> {
        self.read.iter()
    }
}

impl Debug for PreparedFrameNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameGraphNode")
            .field("name", &self.name)
            .finish()
    }
}

pub struct FrameGraph {
    device: Arc<Device>,
    queue: Arc<Queue>,
    command_pool: Arc<CommandPool>,
    resource_state_traker: ResourceStateTracker,
    nodes: Vec<Box<dyn FrameNode + 'static>>,
    queue_family_index: u32,
}

pub struct FrameGraphSubmitInfo {
    pub wait_semaphores: Vec<Arc<Semaphore>>,
    pub signal_semaphores: Vec<Arc<Semaphore>>,
    pub signal_fence: Option<Arc<Fence>>,
}

impl FrameGraph {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, allocator: Arc<MemoryAllocator>) -> Self {
        // TODO: Get actual queue family index
        let queue_family_index = 0;
        let command_pool = CommandPool::new(device.clone(), queue_family_index);
        
        Self {
            device,
            queue,
            command_pool,
            nodes: Default::default(),
            resource_state_traker: Default::default(),
            queue_family_index,
        }
    }

    /// Adds a new pass to the frame graph.
    pub fn add_pass<F>(&mut self, node: F) 
    where 
        F: FrameNode + 'static,
    {
        self.nodes.push(Box::new(node));
    }

    pub fn execute(&mut self, resource_manager: &ResourceManager, submit_info: FrameGraphSubmitInfo) {
        // Prepare the nodes
        let prepared: SmallVec<[PreparedFrameNode; 8]> = self.nodes
            .drain(..)
            .map(|node| {
                let mut prepare_context = PrepareContext::new(resource_manager);
                let execute = node.prepare(&mut prepare_context);

                PreparedFrameNode {
                    name: node.name(),
                    write: prepare_context.write,
                    read: prepare_context.read,
                    execute,
                }
            })
            .collect();

        // Allocate and begin recording the command buffer
        use vislum_render_rhi::command::{CommandBufferLevel, CommandBufferUsageFlags};
        let mut raw_command_buffer = self.command_pool.allocate(CommandBufferLevel::PRIMARY);
        raw_command_buffer.begin(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        let auto_command_buffer = AutoCommandBuffer::new(raw_command_buffer);
        
        // Create command encoder with our owned resources
        let command_encoder =
            CommandEncoder::new(auto_command_buffer, &mut self.resource_state_traker);

        // Prepare the execute context
        let mut execute_context = ExecuteContext { command_encoder };

        // Execute the prepared nodes
        for mut node in prepared {
            node.execute(&mut execute_context);
        }

        // Get the command buffer back and end recording
        let mut auto_command_buffer = execute_context.command_encoder.auto_command_buffer();
        auto_command_buffer.command_buffer_mut().end();
        let raw_command_buffer = auto_command_buffer.into_command_buffer();

        self.submit(&raw_command_buffer, submit_info);
    }

    fn submit(&self, command_buffer: &vislum_render_rhi::command::CommandBuffer, submit_info: FrameGraphSubmitInfo) {
        command_buffer.submit(
            self.queue.clone(),
            submit_info.wait_semaphores,
            submit_info.signal_semaphores,
            submit_info.signal_fence,
        );
    }
}

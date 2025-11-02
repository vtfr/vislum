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

pub struct FrameNode {
    name: Cow<'static, str>,
    execute: Box<dyn for<'g> FnMut(&mut ExecuteContext<'g>) + 'static>,
    write: SmallVec<[FramePassResource; 16]>,
    read: SmallVec<[FramePassResource; 16]>,
}

impl FrameNode {
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

impl Debug for FrameNode {
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
    nodes: Vec<FrameNode>,
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
    pub fn add_pass<S, E, P>(
        &mut self,
        resource_manager: &ResourceManager,
        name: impl Into<Cow<'static, str>>,
        prepare: P,
        execute: E,
    ) where
        S: 'static,
        P: Fn(&mut PrepareContext) -> S,
        E: for<'g, 's> Fn(&mut ExecuteContext<'g>, &'s mut S) + 'static,
    {
        let mut prepare_context = PrepareContext {
            resource_manager,
            write: Default::default(),
            read: Default::default(),
        };

        let mut state = prepare(&mut prepare_context);

        // Erase the execution function and bind it to the state.
        let execute = Box::new(move |ctx: &mut ExecuteContext| {
            execute(ctx, &mut state);
        }) as Box<dyn for<'g> FnMut(&mut ExecuteContext<'g>) + 'static>;

        // Deconstruct the prepare context, freeing the resource manager lock.
        let PrepareContext { write, read, .. } = prepare_context;

        // Construct the node.
        let node = FrameNode {
            name: name.into(),
            write,
            read,
            execute,
        };

        self.nodes.push(node);
    }

    pub fn execute_and_submit(&mut self, submit_info: FrameGraphSubmitInfo) {
        // Allocate and begin recording the command buffer
        let mut raw_command_buffer = self.command_pool.allocate(vk::CommandBufferLevel::PRIMARY);
        raw_command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        
        let auto_command_buffer = AutoCommandBuffer::new(raw_command_buffer);
        
        // Create command encoder with our owned resources
        let command_encoder =
            CommandEncoder::new(auto_command_buffer, &mut self.resource_state_traker);

        // Prepare the execute context
        let mut execute_context = ExecuteContext { command_encoder };

        // Execute the nodes
        for mut node in self.nodes.drain(..) {
            node.execute(&mut execute_context);
        }

        // Get the command buffer back and end recording
        let mut auto_command_buffer = execute_context.command_encoder.auto_command_buffer();
        auto_command_buffer.command_buffer_mut().end();
        let raw_command_buffer = auto_command_buffer.into_command_buffer();

        self.submit(&raw_command_buffer, submit_info);
    }

    fn submit(&self, command_buffer: &vislum_render_rhi::command::CommandBuffer, submit_info: FrameGraphSubmitInfo) {
        let wait_semaphores: Vec<_> = submit_info.wait_semaphores.iter().map(|s| s.as_ref()).collect();
        let signal_semaphores: Vec<_> = submit_info.signal_semaphores.iter().map(|s| s.as_ref()).collect();
        let fence = submit_info.signal_fence.as_ref().map(|f| f.as_ref());
        
        command_buffer.submit(
            &self.queue,
            &wait_semaphores,
            &signal_semaphores,
            fence,
        );
    }
}

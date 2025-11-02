use std::{borrow::Cow, fmt::Debug, sync::{Arc, PoisonError, RwLock, RwLockReadGuard}};

use smallvec::SmallVec;
use vulkano::{
    VulkanObject, command_buffer::{
        CommandBuffer, CommandBufferBeginInfo, CommandBufferLevel, CommandBufferUsage, RecordingCommandBuffer, allocator::{CommandBufferAllocator, StandardCommandBufferAllocator}
    }, device::{Device, Queue}, image::Image, sync::{fence::Fence, semaphore::Semaphore}
};

use crate::{
    graph::{CommandEncoder, ResourceStateTracker},
    resource::{mesh::Mesh, pool::ResourceId, texture::Texture, ResourceManager},
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
    resource_manager: RwLockReadGuard<'a, ResourceManager>,
    write: SmallVec<[FramePassResource; 16]>,
    read: SmallVec<[FramePassResource; 16]>,
}

impl<'a> PrepareContext<'a> {
    pub fn new(resource_manager: RwLockReadGuard<'a, ResourceManager>) -> Self {
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
    resource_manager: Arc<RwLock<ResourceManager>>,
    command_buffer_allocator: Arc<dyn CommandBufferAllocator>,
    resource_state_traker: ResourceStateTracker,
    nodes: Vec<FrameNode>,
}

pub struct FrameGraphSubmitInfo {
    pub wait_semaphores: Vec<Arc<Semaphore>>,
    pub signal_semaphores: Vec<Arc<Semaphore>>,
    pub signal_fence: Option<Arc<Fence>>,
}

impl FrameGraph {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>, resource_manager: Arc<RwLock<ResourceManager>>) -> Self {
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), Default::default()));
        Self {
            device,
            queue,
            resource_manager,
            nodes: Default::default(),
            command_buffer_allocator,
            resource_state_traker: Default::default(),
        }
    }

    /// Adds a new pass to the frame graph.
    pub fn add_pass<S, E, P>(&mut self, name: impl Into<Cow<'static, str>>, prepare: P, execute: E)
    where
        S: 'static,
        P: Fn(&mut PrepareContext) -> S,
        E: for<'g, 's> Fn(&mut ExecuteContext<'g>, &'s mut S) + 'static,
    {
        let resource_manager = self
            .resource_manager
            .read()
            .unwrap_or_else(PoisonError::into_inner);

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
        let queue_family_index = self.queue.queue_family_index();

        // Create the command buffer
        let mut command_buffer = RecordingCommandBuffer::new(
            self.command_buffer_allocator.clone(),
            queue_family_index,
            CommandBufferLevel::Primary,
            CommandBufferBeginInfo {
                usage: CommandBufferUsage::OneTimeSubmit,
                ..Default::default()
            },
        )
        .unwrap();

        // Create command encoder with our owned resources
        let command_encoder = CommandEncoder::new(&mut command_buffer, &mut self.resource_state_traker);

        // Prepare the execute context
        let mut execute_context = ExecuteContext {
            command_encoder,
        };

        // Execute the nodes
        for mut node in self.nodes.drain(..) {
            node.execute(&mut execute_context);
        }

        // Drop the execute context to free the encoder
        drop(execute_context);

        // Finish recording the command buffer
        let command_buffer = unsafe { command_buffer.end().unwrap() };

        self.submit(command_buffer, submit_info);
    }

    // Pretty unsafe, but it's the only way to submit a raw command buffer to a queue.
    fn submit(&self, command_buffer: CommandBuffer, submit_info: FrameGraphSubmitInfo) {
        use vulkano::VulkanObject;

        let device = self.queue.device();
        let fns = device.fns();
        let queue = self.queue.handle();

        let command_buffers = [command_buffer.handle()];

        let wait_semaphores: SmallVec<[ash::vk::Semaphore; 2]> = submit_info
            .wait_semaphores
            .into_iter()
            .map(|semaphore| semaphore.handle())
            .collect();

        let signal_semaphores: SmallVec<[ash::vk::Semaphore; 2]> = submit_info
            .signal_semaphores
            .into_iter()
            .map(|semaphore| semaphore.handle())
            .collect();

        let signal_fence = submit_info.signal_fence
            .map(|fence| fence.handle())
            .unwrap_or(ash::vk::Fence::null());

        let submit_info = ash::vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .wait_semaphores(&*wait_semaphores)
            .signal_semaphores(&*signal_semaphores);

        unsafe {
            (fns.v1_0.queue_submit)(
                queue,
                1,
                &submit_info,
                signal_fence,
            );
        }
    }
}

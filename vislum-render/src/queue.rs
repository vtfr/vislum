use std::{
    fmt::Debug, ops::Deref, sync::Arc
};

/// A render queue.
/// 
/// This is a wrapper around the wgpu::Queue that provides shared ownership of the queue.
#[derive(Clone)]
pub struct RenderQueue(Arc<wgpu::Queue>);

static_assertions::assert_impl_all!(RenderQueue: Send, Sync);

impl Debug for RenderQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RenderQueue")
    }
}

impl From<wgpu::Queue> for RenderQueue {
    fn from(queue: wgpu::Queue) -> Self {
        Self(Arc::new(queue))
    }
}

impl From<Arc<wgpu::Queue>> for RenderQueue {
    fn from(queue: Arc<wgpu::Queue>) -> Self {
        Self(queue)
    }
}

impl Deref for RenderQueue {
    type Target = wgpu::Queue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

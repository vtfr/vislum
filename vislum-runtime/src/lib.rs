use vislum_op::system::NodeGraphSystem;
use vislum_render::types::{RenderDevice, RenderQueue};
use vislum_render::system::RenderSystem;
use vislum_system::{ResMut, Res, Resource, Resources};

/// The runtime for the vislum engine.
pub struct Runtime {
    resources: Resources,
}

impl Runtime {
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        let mut resources = Resources::new();
        
        // Insert the default resources.
        resources.insert(NodeGraphSystem::default());
        resources.insert(RenderSystem::new(device, queue));

        Self {
            resources,
        }
    }

    /// Gets a system by type.
    pub fn get_resource<T>(&self) -> Res<'_, T>
    where
        T: Resource, 
    {
        self.resources.get::<T>()
    }

    /// Gets a mutable system by type.
    pub fn get_resource_mut<T>(&self) -> ResMut<'_, T>
    where
        T: Resource,
    {
        self.resources.get_mut::<T>()
    }
}

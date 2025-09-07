use vislum_op::system::NodeGraphSystem;
use vislum_render::{MeshManager, RenderPassCollector, SceneManager, ShaderManager, TextureManager};
use vislum_render::types::{RenderDevice, RenderQueue};
use vislum_system::{ResMut, Res, Resource, Resources};

/// The runtime for the vislum engine.
pub struct Runtime {
    pub resources: Resources,
}

impl Runtime {
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        let mut resources = Resources::new();
        
        // Insert the default resources.
        resources.insert(NodeGraphSystem::default());
        resources.insert(TextureManager::new(device.clone(), queue.clone()));
        resources.insert(MeshManager::new(device.clone()));
        resources.insert(ShaderManager::new(device.clone()));
        resources.insert(SceneManager::new());
        resources.insert(RenderPassCollector::new());
        resources.insert(device);
        resources.insert(queue);


        Self {
            resources,
        }
    }

    /// Gets a resource by type.
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

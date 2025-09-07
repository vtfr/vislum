use vislum_op::system::NodeGraphSystem;
use vislum_render::types::{RenderDevice, RenderQueue};
use vislum_render::system::RenderSystem;
use vislum_system::{SysMut, SysRef, System, Systems};

/// The runtime for the vislum engine.
pub struct Runtime {
    systems: Systems,
}

impl Runtime {
    pub fn new(device: RenderDevice, queue: RenderQueue) -> Self {
        let mut systems = Systems::new();
        
        // Insert the default systems.
        systems.insert(NodeGraphSystem::default());
        systems.insert(RenderSystem::new(device, queue));

        Self {
            systems,
        }
    }

    /// Gets a system by type.
    pub fn get_system<T>(&self) -> SysRef<'_, T>
    where
        T: System + 'static,
    {
        self.systems.must_get::<T>()
    }

    /// Gets a mutable system by type.
    pub fn get_system_mut<T>(&self) -> SysMut<'_, T>
    where
        T: System + 'static,
    {
        self.systems.must_get_mut::<T>()
    }
}

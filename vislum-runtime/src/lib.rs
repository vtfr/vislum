use vislum_op::system::NodeGraphSystem;
use vislum_system::{SysMut, SysRef, System, Systems};

/// The runtime for the vislum engine.
pub struct Runtime {
    systems: Systems,
}

impl Runtime {
    pub fn new() -> Self {
        let mut systems = Systems::new();
        systems.insert(NodeGraphSystem::default());

        Self {
            systems,
        }
    }

    /// Gets a system by type.
    pub fn get_system<T>(&self) -> SysRef<T>
    where
        T: System + 'static,
    {
        self.systems.must_get::<T>()
    }

    /// Gets a mutable system by type.
    pub fn get_system_mut<T>(&self) -> SysMut<T>
    where
        T: System + 'static,
    {
        self.systems.must_get_mut::<T>()
    }
}

use std::sync::Arc;

use vulkano::buffer::BufferContents;

use crate::resources::id::HandleInner;

#[derive(Debug, Clone, BufferContents)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

pub struct Mesh {
    pub buffer: Arc<vulkano::buffer::Buffer>,
}

impl Resource for Mesh {
    const TYPE: ResourceType = ResourceType::Mesh;
}

#[derive(Debug)]
pub struct MeshDescriptor<'a> {
    pub vertices: &'a [Vertex],
    pub indices: &'a [u32],
}

struct MeshHandleUserData {
    indices: u32,
}

pub struct MeshHandle(Arc<HandleInner<Mesh, MeshHandleUserData>>);

impl MeshHandle {
    pub(crate) fn new(
        id: ResourceId<Mesh>,
        indices: u32,
        resource_drop_tx: crossbeam_channel::Sender<ErasedResourceId>,
    ) -> Self {
        let user_data = MeshHandleUserData { indices };
        Self(Arc::new(HandleInner::new(id, user_data, resource_drop_tx)))
    }

    pub fn indices(&self) -> u32 {
        self.0.user_data().indices
    }
}

impl Debug for MeshHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshHandle")
            .field("id", &self.0.id())
            .field("indices", &self.indices())
            .finish()
    }
}

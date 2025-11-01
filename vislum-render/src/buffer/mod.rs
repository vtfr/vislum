use std::sync::{Arc, RwLock};

use vulkano::{buffer::{Buffer, Subbuffer}, device::Device, memory::allocator::MemoryAllocator};

use crate::context::RenderContext;

/// Manages the pending uploads to the device.
pub struct PendingUploadsManager {
}

struct PendingUniformUpload<T> {
    data: T,
    staging_buffer: Arc<Subbuffer<T>>,
}

/// A uniform buffer.
pub struct Uniform<T> {
    context: Arc<RenderContext>,
    pending: RwLock<Option<PendingUniformUpload<T>>>,
    buffer: Arc<Subbuffer<T>>,
}

impl<T> Uniform<T> {
    pub fn new(context: Arc<RenderContext>, data: T) -> Self {
        todo!()

    //     Self {
    //         context,
    //         pending: RwLock::new(None),
    //         buffer: Arc::new(Subbuffer::new(device.clone(), data)),
    //     }
    }
}


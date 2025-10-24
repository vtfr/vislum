use std::{collections::HashMap, sync::Arc};

use crate::rhi::{device::Device, image::{Image, ImageDescription}, queue::Queue};

pub struct ImageHandle(pub u32);

pub struct RenderContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    images: HashMap<ImageHandle, Image>,
}

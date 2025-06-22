use std::sync::Arc;

use derive_more::{Deref, From};

#[derive(Debug, Clone, From, Deref)]
#[from(forward)]
pub struct Device(Arc<wgpu::Device>);

#[derive(Debug, Clone, From, Deref)]
#[from(forward)]
pub struct Queue(Arc<wgpu::Queue>);

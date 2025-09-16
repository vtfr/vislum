use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use vislum_system::Resource;

use crate::device::RenderDevice;

pub mod bind_group;

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Visibility {
    Vertex,
    Fragment,
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum BindingType {
    StorageBuffer,
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct BindGroupLayoutEntry {
    pub visibility: Visibility,
    pub ty: BindingType,
}

#[derive(Clone)]
pub struct BindGroupLayout {
    inner: Arc<wgpu::BindGroupLayout>,
}

impl BindGroupLayout {
    /// Gets the inner wgpu bind group layout.
    pub fn inner(&self) -> &wgpu::BindGroupLayout {
        &self.inner
    }
}

#[derive(Resource)]
pub struct BindGroupLayoutCache {
    device: RenderDevice,
    entries: HashMap<Vec<BindGroupLayoutEntry>, BindGroupLayout>,
}

impl BindGroupLayoutCache {
    /// Creates a new bind group layout cache.
    pub fn new(device: RenderDevice) -> Self {
        Self { device, entries: Default::default() }
    }

    /// Gets a bind group layout from the cache.
    /// 
    /// If the layout is not found, it will be created and added to the cache.
    pub fn get(&mut self, entries: &[BindGroupLayoutEntry]) -> BindGroupLayout {
        if let Some(layout) = self.entries.get(entries) {
            return layout.clone();
        }

        let wgpu_entries = entries.iter()
            .enumerate()
            .map(|(i, entry)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: match entry.visibility {
                    Visibility::Vertex => wgpu::ShaderStages::VERTEX,
                    Visibility::Fragment => wgpu::ShaderStages::FRAGMENT,
                },
                ty: match entry.ty {
                    BindingType::StorageBuffer => wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: true,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                },
                count: None,
            })
            .collect::<Vec<_>>();

        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: None,
            entries: &wgpu_entries,
        });

        let layout = BindGroupLayout {
            inner: Arc::new(layout),
        };

        self.entries.insert(entries.to_vec(), layout.clone());

        layout
    }
}
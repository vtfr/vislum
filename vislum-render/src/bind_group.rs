use std::{borrow::Cow, collections::HashMap};

use vislum_system::System;

use crate::{types::RenderDevice, wrap_wgpu_with_atomic_id};

wrap_wgpu_with_atomic_id! {
    /// A bind group layout.
    pub struct BindGroupLayout(BindGroupLayoutId): wgpu::BindGroupLayout;
}

#[derive(System)]
pub struct BindGroupLayoutCache {
    device: RenderDevice,
    descriptors: HashMap<Vec<wgpu::BindGroupLayoutEntry>, BindGroupLayout>,
}

impl BindGroupLayoutCache {
    pub fn new(device: RenderDevice) -> Self {
        Self { device, descriptors: Default::default() }
    }

    /// Creates a new bind group layout, or returns an existing one.
    pub fn create<'a>(
        &'a mut self, 
        entries: impl Into<Cow<'a, [wgpu::BindGroupLayoutEntry]>>,
    ) -> BindGroupLayout {
        let entries = entries.into();
        if let Some(bind_group_layout) = self.descriptors.get(&*entries) {
            return bind_group_layout.clone();
        }

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { 
            label: None, 
            entries: &*entries,
        });

        let key = entries.into_owned();
        let bind_group_layout = BindGroupLayout::new(bind_group_layout);

        self.descriptors.insert(key, bind_group_layout.clone());

        bind_group_layout
    }
}

wrap_wgpu_with_atomic_id! {
    /// A bind group.
    pub struct BindGroup(BindGroupId): wgpu::BindGroup;
}

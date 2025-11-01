use std::sync::Arc;

use vulkano::{
    descriptor_set::allocator::{
        DescriptorSetAllocator, StandardDescriptorSetAllocator,
        StandardDescriptorSetAllocatorCreateInfo,
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags, physical::PhysicalDevice,
    },
    instance::{Instance, InstanceExtensions},
    library::VulkanLibrary,
    memory::allocator::{MemoryAllocator, StandardMemoryAllocator},
    swapchain::Surface,
};

use crate::resource::ResourceStorage;

// use crate::resources::ResourceManager;

bitflags::bitflags! {
    pub struct RenderingFeatures: u8 {
        const RAY_TRACING = 1 << 0;
    }
}

/// A physical device that is compatible with the instance.
#[derive(Debug, Clone)]
pub struct CompatiblePhysicalDevice {
    physical_device: Arc<PhysicalDevice>,
    minimum_extensions: DeviceExtensions,
    queue_family_index: u32,
}

impl CompatiblePhysicalDevice {
    pub fn new(physical_device: Arc<PhysicalDevice>) -> Option<Self> {
        let mut minimum_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..Default::default()
        };

        if physical_device.api_version() < vulkano::Version::V1_3 {
            minimum_extensions.khr_dynamic_rendering = true;
            minimum_extensions.khr_synchronization2 = true;
        }

        if !physical_device
            .supported_extensions()
            .intersects(&minimum_extensions)
        {
            return None;
        }

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .filter_map(|(index, queue_family_properties)| {
                // Queue has to support graphics operations.
                if queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
                {
                    Some(index as u32)
                } else {
                    None
                }
            })
            .next()?;

        Some(CompatiblePhysicalDevice {
            physical_device,
            minimum_extensions,
            queue_family_index,
        })
    }

    /// The required extensions for the physical device.
    #[inline]
    pub fn required_extensions(&self) -> &DeviceExtensions {
        &self.minimum_extensions
    }

    /// Score the physical device based on its type and features.
    #[inline]
    pub fn score(&self) -> u8 {
        match self.physical_device.properties().device_type {
            vulkano::device::physical::PhysicalDeviceType::DiscreteGpu => 0,
            vulkano::device::physical::PhysicalDeviceType::VirtualGpu => 1,
            vulkano::device::physical::PhysicalDeviceType::IntegratedGpu => 2,
            vulkano::device::physical::PhysicalDeviceType::Cpu => 3,
            _ => 5,
        }
    }
}

pub struct RenderContextBuilder {
    instance: Arc<Instance>,
    window: Arc<winit::window::Window>,
}

impl RenderContextBuilder {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        window: Arc<winit::window::Window>,
    ) -> Self {
        let library = VulkanLibrary::new().unwrap();

        let extensions = Surface::required_extensions(event_loop).unwrap();

        let create_info = vulkano::instance::InstanceCreateInfo {
            engine_name: Some("Vislum".to_string()),
            engine_version: vulkano::Version::V1_0,
            max_api_version: Some(vulkano::Version::V1_3),
            enabled_extensions: extensions,
            ..Default::default()
        };

        let instance = Instance::new(library, create_info).unwrap();

        Self { instance, window }
    }

    /// Enumerate all the physical devices that are compatible with the instance.
    pub fn enumerate_compatible_physical_devices(&self) -> Vec<CompatiblePhysicalDevice> {
        let mut compatible_physical_devices = self
            .instance
            .enumerate_physical_devices()
            .unwrap()
            .filter_map(CompatiblePhysicalDevice::new)
            .collect::<Vec<_>>();

        compatible_physical_devices.sort_by_key(|device| device.score());
        compatible_physical_devices
    }

    pub fn build(self, compatible_physical_device: CompatiblePhysicalDevice) -> RenderContext {
        let CompatiblePhysicalDevice {
            physical_device,
            queue_family_index,
            minimum_extensions: enabled_extensions,
            ..
        } = compatible_physical_device;

        let device_create_info = DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions,
            enabled_features: DeviceFeatures {
                dynamic_rendering: true,
                synchronization2: true,
                extended_dynamic_state: true,
                // Descriptor indexing.
                shader_sampled_image_array_non_uniform_indexing: true,
                runtime_descriptor_array: true,
                descriptor_binding_variable_descriptor_count: true,
                descriptor_binding_partially_bound: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let (device, mut queues) = Device::new(physical_device, device_create_info).unwrap();
        let queue = queues.next().unwrap();

        RenderContext::new(device, queue)
    }
}

#[derive(Debug)]
pub struct RenderContext {
    device: Arc<Device>,
    queue: Arc<Queue>,
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    memory_allocator: Arc<dyn MemoryAllocator>,
    // textures: ResourceStorage<Texture>,
    // resource_manager: ResourceManager,
}

impl RenderContext {
    pub fn new(device: Arc<Device>, queue: Arc<Queue>) -> Self {
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            StandardDescriptorSetAllocatorCreateInfo {
                update_after_bind: false,
                ..Default::default()
            },
        ));

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        // let resource_manager = ResourceManager::new(
        //     device.clone(),
        //     descriptor_set_allocator.clone(),
        //     memory_allocator.clone(),
        // );

        Self {
            device,
            queue,
            descriptor_set_allocator,
            memory_allocator,
            // resource_manager,
        }
    }

    // pub fn resource_manager_mut(&mut self) -> &mut ResourceManager {
    //     // &mut self.resource_manager
    // }
}

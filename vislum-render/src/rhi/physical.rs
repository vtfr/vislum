use std::{collections::HashSet, ffi::CStr, sync::{Arc, OnceLock}};

use ash::vk;

use crate::rhi::{Extension, Instance};

#[derive(Debug)]
pub struct QueueFamily {
    /// The index of the queue family.
    index: u32,

    /// The flags that the queue family supports.
    flags: vk::QueueFlags,

    /// The number of queues in the family.
    queue_count: u32,
}

pub struct PhysicalDevice {
    /// The instance that this physical device belongs to.
    instance: Arc<Instance>,

    /// The raw Vulkan physical device handle.
    handle: vk::PhysicalDevice,
    
    /// The properties of the physical device.
    properties: vk::PhysicalDeviceProperties,

    /// The queue family properties of the physical device.
    queue_families: Vec<QueueFamily>,

    /// The memory properties of the physical device. Stored behind a once-lock to enable lazy querying.
    memory_properties: OnceLock<PhysicalDeviceMemoryProperties>,

    /// The device extensions of the physical device. Stored behind a once-lock to enable lazy querying.
    device_extensions: OnceLock<HashSet<Extension>>,

    /// The features of the physical device. Stored behind a once-lock to enable lazy querying.
    features: OnceLock<vk::PhysicalDeviceFeatures>,
}

static_assertions::assert_impl_all!(PhysicalDevice: Send, Sync);

impl std::fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("handle", &self.handle)
            .field("properties", &self.properties)
            .field("queue_families", &self.queue_families)
            .field("supported_extensions", &self.supported_extensions())
            .field("memory_properties", &self.memory_properties())
            .finish()
    }
}

impl PhysicalDevice {
    /// Creates a new PhysicalDevice from raw Vulkan handles and properties.
    /// 
    /// Returns `None` if the physical device is not supported.
    pub(crate) fn new(instance: Arc<Instance>, physical_device: vk::PhysicalDevice) -> Option<Self> {
        // Perform an initial query to get the physical device version.
        let mut vk_properties = vk::PhysicalDeviceProperties2::default();
        unsafe { instance.ash_handle().get_physical_device_properties2(physical_device, &mut vk_properties) };

        // We only support Vulkan 1.3 and above. Sorry!
        if vk_properties.properties.api_version < vk::API_VERSION_1_3 {
            return None;
        }

        let mut vk_features = vk::PhysicalDeviceFeatures2::default();
        let mut vulkan_1_3_features = vk::PhysicalDeviceVulkan13Features::default();
        vk_features = vk_features.push_next(&mut vulkan_1_3_features);

        unsafe { instance.ash_handle().get_physical_device_features2(physical_device, &mut vk_features) };

        if vulkan_1_3_features.dynamic_rendering == vk::FALSE || vulkan_1_3_features.synchronization2 == vk::FALSE {
            return None;
        }

        // Query the queue family properties.
        let queue_families = unsafe {
            instance
                .ash_handle()
                .get_physical_device_queue_family_properties(physical_device)
                .into_iter()
                .enumerate()
                .map(|(queue_family_index, properties)| QueueFamily {
                    index: queue_family_index as u32,
                    flags: properties.queue_flags,
                    queue_count: properties.queue_count,
                })
                .collect()
        };

        Some(Self {
            handle: physical_device,
            instance,
            properties: vk_properties.properties,
            queue_families,
            device_extensions: Default::default(),
            memory_properties: Default::default(),
            features: Default::default(),
        })
    }

    /// Returns the raw Vulkan physical device handle.
    pub fn vk_physical_device(&self) -> vk::PhysicalDevice {
        self.handle
    }

    /// Returns the memory properties of the physical device.
    pub fn memory_properties(&self) -> &PhysicalDeviceMemoryProperties {
        self.memory_properties.get_or_init(|| {
            let vk_memory_properties = unsafe { self.instance
                .ash_handle()
                .get_physical_device_memory_properties(self.handle) };

            let memory_types = vk_memory_properties.memory_types
                .into_iter()
                .take(vk_memory_properties.memory_type_count as usize)
                .collect();

            let memory_heaps = vk_memory_properties.memory_heaps
                .into_iter()
                .take(vk_memory_properties.memory_heap_count as usize)
                .collect();

            PhysicalDeviceMemoryProperties {
                memory_types,
                memory_heaps,
            }
        })
    }

    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        self.features.get_or_init(|| unsafe {
            self.instance
                .ash_handle()
                .get_physical_device_features(self.handle)
        })
    }

    /// Returns the device extensions of the physical device.
    pub fn supported_extensions(&self) -> &HashSet<Extension> {
        let extensions = self.device_extensions.get_or_init(|| unsafe {
            let extensions = self
                .instance
                .ash_handle()
                .enumerate_device_extension_properties(self.handle)
                .unwrap_or_default();

            extensions
                .into_iter()
                .map(|extension| Extension::from(extension.extension_name))
                .collect()
        });

        extensions
    }
}

#[derive(Debug)]
pub struct PhysicalDeviceMemoryProperties {
    /// The memory types of the physical device.
    pub memory_types: Vec<vk::MemoryType>,
    
    /// The memory heaps of the physical device.
    pub memory_heaps: Vec<vk::MemoryHeap>,
}

pub struct PhysicalDeviceFeatures {
    /// The core features of the physical device.
    pub features: vk::PhysicalDeviceFeatures,
    
    /// The dynamic rendering features of the physical device.
    pub dynamic_rendering: vk::PhysicalDeviceDynamicRenderingFeatures<'static>,

    /// The sync2 features of the physical device.
    pub sync2: vk::PhysicalDeviceSynchronization2Features<'static>,
}

impl PhysicalDeviceFeatures {
    /// Resets the `p_next` pointers of the features.
    pub fn reset_next(&mut self) {
        self.dynamic_rendering.p_next = std::ptr::null_mut(); 
        self.sync2.p_next = std::ptr::null_mut();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureSupport {
    /// The feature is supported in core.
    Core,

    /// The feature is supported in an extension.
    Extension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicalDeviceCapabilities {
    pub dynamic_rendering: FeatureSupport,
    pub synchronization2: FeatureSupport,
}

use std::sync::{Arc, PoisonError, RwLock};

use ash::vk;

use crate::{AshHandle, VkHandle, device::device::Device};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLocation {
    GpuOnly,
    CpuToGpu,
    GpuToCpu,
}

pub struct AllocationDescription<'a> {
    pub name: Option<&'a str>,
    pub requirements: MemoryRequirements,
    pub location: MemoryLocation,
}

pub struct MemoryAllocation {
    allocator: Arc<MemoryAllocator>,
    inner: Option<gpu_allocator::vulkan::Allocation>,
}

impl MemoryAllocation {
    pub fn memory(&self) -> vk::DeviceMemory {
        unsafe { self.inner.as_ref().unwrap().memory() }
    }

    pub fn offset(&self) -> u64 {
        self.inner.as_ref().unwrap().offset()
    }

    pub fn size(&self) -> u64 {
        self.inner.as_ref().unwrap().size()
    }

    pub fn mapped_ptr(&self) -> Option<std::ptr::NonNull<std::ffi::c_void>> {
        self.inner.as_ref().unwrap().mapped_ptr()
    }
}

impl Drop for MemoryAllocation {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            self.allocator.free(inner);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MemoryAllocationError {
    #[error("allocation error: {0}")]
    AllocationError(#[from] gpu_allocator::AllocationError),
}

pub type MemoryRequirements = vk::MemoryRequirements;

pub struct MemoryAllocator {
    allocator: RwLock<gpu_allocator::vulkan::Allocator>,
}

impl MemoryAllocator {
    /// Creates a new memory allocator.
    pub fn new(device: Arc<Device>) -> Arc<Self> {
        let vk_physical_device = device.physical_device().vk_handle();
        let ash_device = device.ash_handle();
        let ash_instance = device.physical_device().instance().ash_handle();

        let mut debug_settings = gpu_allocator::AllocatorDebugSettings::default();
        debug_settings.log_memory_information = true;
        debug_settings.log_leaks_on_shutdown = true;
        debug_settings.store_stack_traces = true;
        debug_settings.log_allocations = true;
        debug_settings.log_frees = true;
        debug_settings.log_stack_traces = true;

        let create_info = gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: ash_instance,
            device: ash_device,
            physical_device: vk_physical_device,
            debug_settings,
            buffer_device_address: false,
            allocation_sizes: gpu_allocator::AllocationSizes::default(),
        };

        let allocator = gpu_allocator::vulkan::Allocator::new(&create_info)
            .expect("Failed to create memory allocator");

        Arc::new(Self {
            allocator: RwLock::new(allocator),
        })
    }

    /// Allocates a new memory allocation.
    pub fn allocate(
        self: &Arc<Self>,
        allocation_description: AllocationDescription,
    ) -> Result<MemoryAllocation, MemoryAllocationError> {
        let location = match allocation_description.location {
            MemoryLocation::GpuOnly => gpu_allocator::MemoryLocation::GpuOnly,
            MemoryLocation::CpuToGpu => gpu_allocator::MemoryLocation::CpuToGpu,
            MemoryLocation::GpuToCpu => gpu_allocator::MemoryLocation::GpuToCpu,
        };

        let create_info = gpu_allocator::vulkan::AllocationCreateDesc {
            name: allocation_description.name.unwrap_or("<unknown>"),
            requirements: allocation_description.requirements,
            location,
            linear: true,
            allocation_scheme: gpu_allocator::vulkan::AllocationScheme::GpuAllocatorManaged,
        };

        let inner = self
            .allocator
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .allocate(&create_info)?;

        Ok(MemoryAllocation {
            allocator: self.clone(),
            inner: Some(inner),
        })
    }

    /// Frees an allocation.
    fn free(self: &Arc<Self>, allocation: gpu_allocator::vulkan::Allocation) {
        self.allocator
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .free(allocation)
            .expect("Failed to free memory allocation");
    }
}

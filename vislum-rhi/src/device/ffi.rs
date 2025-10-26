use ash::vk;
use vislum_rhi_macros::Wiring;

use crate::{impl_extensions, impl_features, version::Version};

impl_extensions! {
    pub struct DeviceExtensions {
        pub khr_swapchain = ash::khr::swapchain::NAME,
        pub khr_dynamic_rendering = ash::khr::dynamic_rendering::NAME,
        pub khr_synchronization2 = ash::khr::synchronization2::NAME,
        pub ext_extended_dynamic_state = ash::ext::extended_dynamic_state::NAME,
        pub khr_ext_descriptor_indexing = ash::ext::descriptor_indexing::NAME,
    }
}

impl_features! {
    pub struct DeviceFeatures {
        // Vulkan 1.2 features.
        pub descriptor_indexing: bool,
        pub shader_input_attachment_array_dynamic_indexing: bool,
        pub shader_uniform_texel_buffer_array_dynamic_indexing: bool,
        pub shader_storage_texel_buffer_array_dynamic_indexing: bool,
        pub shader_uniform_buffer_array_non_uniform_indexing: bool,
        pub shader_sampled_image_array_non_uniform_indexing: bool,
        pub shader_storage_buffer_array_non_uniform_indexing: bool,
        pub shader_storage_image_array_non_uniform_indexing: bool,
        pub shader_input_attachment_array_non_uniform_indexing: bool,
        pub shader_uniform_texel_buffer_array_non_uniform_indexing: bool,
        pub shader_storage_texel_buffer_array_non_uniform_indexing: bool,
        pub descriptor_binding_uniform_buffer_update_after_bind: bool,
        pub descriptor_binding_sampled_image_update_after_bind: bool,
        pub descriptor_binding_storage_image_update_after_bind: bool,
        pub descriptor_binding_storage_buffer_update_after_bind: bool,
        pub descriptor_binding_uniform_texel_buffer_update_after_bind: bool,
        pub descriptor_binding_storage_texel_buffer_update_after_bind: bool,
        pub descriptor_binding_update_unused_while_pending: bool,
        pub descriptor_binding_partially_bound: bool,
        pub descriptor_binding_variable_descriptor_count: bool,
        pub runtime_descriptor_array: bool,
        pub scalar_block_layout: bool,
        pub timeline_semaphore: bool,
        pub buffer_device_address: bool,

        // Vulkan 1.3 features.
        pub synchronization2: bool,
        pub dynamic_rendering: bool,

        // Extensions features.
        pub extended_dynamic_state: bool,
    }
}

impl DeviceFeatures {
    /// The minimum features required by the RHI for rendering.
    const MINIMUM: DeviceFeatures = DeviceFeatures {
        dynamic_rendering: true,
        synchronization2: true,
        extended_dynamic_state: true,
        ..DeviceFeatures::empty()
    };

    /// The minimum features required for bindless descriptor sets.
    const BINDLESS: DeviceFeatures = DeviceFeatures {
        descriptor_indexing: true,
        shader_input_attachment_array_dynamic_indexing: false,
        shader_sampled_image_array_non_uniform_indexing: true,
        runtime_descriptor_array: true,
        descriptor_binding_variable_descriptor_count: true,
        descriptor_binding_partially_bound: true,
        ..DeviceFeatures::empty()
    };
}

#[derive(Default, Wiring)]
pub(crate) struct DevicePhysicalFeaturesFFI {
    // #[wiring(base)]
    // pub features: vk::PhysicalDeviceFeatures,

    // #[wiring(
    //         version(Version::V1_1),
    //         provides(
    //         )
    //     )]
    // pub vulkan_1_1_features: Option<vk::PhysicalDeviceVulkan11Features<'static>>,
    #[wiring(
        version(Version::V1_2),
        provides(
            descriptor_indexing,
            shader_input_attachment_array_dynamic_indexing,
            shader_uniform_texel_buffer_array_dynamic_indexing,
            shader_storage_texel_buffer_array_dynamic_indexing,
            shader_uniform_buffer_array_non_uniform_indexing,
            shader_sampled_image_array_non_uniform_indexing,
            shader_storage_buffer_array_non_uniform_indexing,
            shader_storage_image_array_non_uniform_indexing,
            shader_input_attachment_array_non_uniform_indexing,
            shader_uniform_texel_buffer_array_non_uniform_indexing,
            shader_storage_texel_buffer_array_non_uniform_indexing,
            descriptor_binding_uniform_buffer_update_after_bind,
            descriptor_binding_sampled_image_update_after_bind,
            descriptor_binding_storage_image_update_after_bind,
            descriptor_binding_storage_buffer_update_after_bind,
            descriptor_binding_uniform_texel_buffer_update_after_bind,
            descriptor_binding_storage_texel_buffer_update_after_bind,
            descriptor_binding_update_unused_while_pending,
            descriptor_binding_partially_bound,
            descriptor_binding_variable_descriptor_count,
            runtime_descriptor_array,
            scalar_block_layout,
            timeline_semaphore,
            buffer_device_address,
        )
    )]
    pub vulkan_1_2_features: Option<vk::PhysicalDeviceVulkan12Features<'static>>,

    #[wiring(version(Version::V1_3), provides(synchronization2, dynamic_rendering,))]
    pub vulkan_1_3_features: Option<vk::PhysicalDeviceVulkan13Features<'static>>,

    #[wiring(
        promoted(Version::V1_2),
        extension(khr_ext_descriptor_indexing),
        provides(
            descriptor_indexing = true,
            shader_input_attachment_array_dynamic_indexing,
            shader_uniform_texel_buffer_array_dynamic_indexing,
            shader_storage_texel_buffer_array_dynamic_indexing,
            shader_uniform_buffer_array_non_uniform_indexing,
            shader_sampled_image_array_non_uniform_indexing,
            shader_storage_buffer_array_non_uniform_indexing,
            shader_storage_image_array_non_uniform_indexing,
            shader_input_attachment_array_non_uniform_indexing,
            shader_uniform_texel_buffer_array_non_uniform_indexing,
            shader_storage_texel_buffer_array_non_uniform_indexing,
            descriptor_binding_uniform_buffer_update_after_bind,
            descriptor_binding_sampled_image_update_after_bind,
            descriptor_binding_storage_image_update_after_bind,
            descriptor_binding_storage_buffer_update_after_bind,
            descriptor_binding_uniform_texel_buffer_update_after_bind,
            descriptor_binding_storage_texel_buffer_update_after_bind,
            descriptor_binding_update_unused_while_pending,
        )
    )]
    pub descriptor_indexing_features: Option<vk::PhysicalDeviceDescriptorIndexingFeatures<'static>>,

    #[wiring(
        promoted(Version::V1_3),
        extension(khr_synchronization2),
        provides(synchronization2,)
    )]
    pub synchronization2_features: Option<vk::PhysicalDeviceSynchronization2Features<'static>>,

    #[wiring(
        promoted(Version::V1_3),
        extension(khr_dynamic_rendering),
        provides(dynamic_rendering,)
    )]
    pub dynamic_rendering_features: Option<vk::PhysicalDeviceDynamicRenderingFeatures<'static>>,

    #[wiring(
        extension(ext_extended_dynamic_state),
        provides(extended_dynamic_state,)
    )]
    pub extended_dynamic_state_features:
        Option<vk::PhysicalDeviceExtendedDynamicStateFeaturesEXT<'static>>,
}

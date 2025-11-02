use ash::vk;

use crate::{Version, device::DeviceExtensions};

/// A generic trait for storing features.
pub trait FeatureStorage {
    /// Writes to the [`DeviceFeatures`] struct.
    fn write_to_device_features(&self, features: &mut DeviceFeatures);

    /// Reads from the [`DeviceFeatures`] struct.
    fn read_from_device_features(&mut self, features: &DeviceFeatures);
}

trait BoolLike {
    fn into_bool(self) -> bool;
    fn from_bool(value: bool) -> Self;
}

macro_rules! impl_feature_storage {
    (
        for $ident:path;

        $($field:ident),*
        $(,)?
    ) => {
        impl FeatureStorage for $ident {
            fn write_to_device_features(&self, features: &mut DeviceFeatures) {
                $(
                    features.$field = self.$field == vk::TRUE;
                )*
            }

            fn read_from_device_features(&mut self, features: &DeviceFeatures) {
                $(
                    self.$field = if features.$field { vk::TRUE } else { vk::FALSE };
                )*
            }
        }
    };
}

impl_feature_storage! {
    for vk::PhysicalDeviceVulkan11Features<'static>;
    storage_buffer16_bit_access,
    uniform_and_storage_buffer16_bit_access,
    storage_push_constant16,
    storage_input_output16,
    multiview,
    multiview_geometry_shader,
    multiview_tessellation_shader,
    variable_pointers_storage_buffer,
    variable_pointers,
    protected_memory,
    sampler_ycbcr_conversion,
    shader_draw_parameters,
}

impl_feature_storage! {
    for vk::PhysicalDeviceVulkan12Features<'static>;
    sampler_mirror_clamp_to_edge,
    draw_indirect_count,
    storage_buffer8_bit_access,
    uniform_and_storage_buffer8_bit_access,
    storage_push_constant8,
    shader_buffer_int64_atomics,
    shader_shared_int64_atomics,
    shader_float16,
    shader_int8,
    shader_uniform_buffer_array_non_uniform_indexing,
    shader_sampled_image_array_non_uniform_indexing,
    shader_storage_buffer_array_non_uniform_indexing,
    shader_storage_image_array_non_uniform_indexing,
    shader_input_attachment_array_non_uniform_indexing,
    shader_uniform_texel_buffer_array_non_uniform_indexing,
    shader_storage_texel_buffer_array_non_uniform_indexing,
    descriptor_binding_uniform_buffer_update_after_bind,
    sampler_filter_minmax,
    scalar_block_layout,
    imageless_framebuffer,
    uniform_buffer_standard_layout,
    shader_subgroup_extended_types,
    separate_depth_stencil_layouts,
    host_query_reset,
    timeline_semaphore,
    buffer_device_address,
    buffer_device_address_capture_replay,
    buffer_device_address_multi_device,
    vulkan_memory_model,
    vulkan_memory_model_device_scope,
    vulkan_memory_model_availability_visibility_chains,
    shader_output_layer,
}

impl_feature_storage! {
    for vk::PhysicalDeviceVulkan13Features<'static>;
    robust_image_access,
    inline_uniform_block,
    descriptor_binding_inline_uniform_block_update_after_bind,
    pipeline_creation_cache_control,
    private_data,
    shader_demote_to_helper_invocation,
    shader_terminate_invocation,
    subgroup_size_control,
    compute_full_subgroups,
    synchronization2,
    texture_compression_astc_hdr,
    shader_zero_initialize_workgroup_memory,
    dynamic_rendering,
    shader_integer_dot_product,
    maintenance4,
}

macro_rules! impl_device_features {
    (
        $vis:vis struct $ident:ident {
            $($field:ident),*
            $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        $vis struct $ident {
            $(pub $field: bool),*
        }

        impl $ident {
            pub const fn empty() -> Self {
                Self { $($field: false),* }
            }
        }
    }
}

impl_device_features! {
    pub struct DeviceFeatures {
        storage_buffer16_bit_access,
        uniform_and_storage_buffer16_bit_access,
        storage_push_constant16,
        storage_input_output16,
        multiview,
        multiview_geometry_shader,
        multiview_tessellation_shader,
        variable_pointers_storage_buffer,
        variable_pointers,
        protected_memory,
        sampler_ycbcr_conversion,
        shader_draw_parameters,
        sampler_mirror_clamp_to_edge,
        draw_indirect_count,
        storage_buffer8_bit_access,
        uniform_and_storage_buffer8_bit_access,
        storage_push_constant8,
        shader_buffer_int64_atomics,
        shader_shared_int64_atomics,
        shader_float16,
        shader_int8,
        shader_uniform_buffer_array_non_uniform_indexing,
        shader_sampled_image_array_non_uniform_indexing,
        shader_storage_buffer_array_non_uniform_indexing,
        shader_storage_image_array_non_uniform_indexing,
        shader_input_attachment_array_non_uniform_indexing,
        shader_uniform_texel_buffer_array_non_uniform_indexing,
        shader_storage_texel_buffer_array_non_uniform_indexing,
        descriptor_binding_uniform_buffer_update_after_bind,
        sampler_filter_minmax,
        scalar_block_layout,
        imageless_framebuffer,
        uniform_buffer_standard_layout,
        shader_subgroup_extended_types,
        separate_depth_stencil_layouts,
        host_query_reset,
        timeline_semaphore,
        buffer_device_address,
        buffer_device_address_capture_replay,
        buffer_device_address_multi_device,
        vulkan_memory_model,
        vulkan_memory_model_device_scope,
        vulkan_memory_model_availability_visibility_chains,
        shader_output_layer,
        robust_image_access,
        inline_uniform_block,
        descriptor_binding_inline_uniform_block_update_after_bind,
        pipeline_creation_cache_control,
        private_data,
        shader_demote_to_helper_invocation,
        shader_terminate_invocation,
        subgroup_size_control,
        compute_full_subgroups,
        synchronization2,
        texture_compression_astc_hdr,
        shader_zero_initialize_workgroup_memory,
        dynamic_rendering,
        shader_integer_dot_product,
        maintenance4,
    }
}

#[derive(Default)]
pub(crate) struct PhysicalDeviceFeaturesFfi {
    pub vk11: Option<vk::PhysicalDeviceVulkan11Features<'static>>,
    pub vk12: Option<vk::PhysicalDeviceVulkan12Features<'static>>,
    pub vk13: Option<vk::PhysicalDeviceVulkan13Features<'static>>,
    pub khr_dynamic_rendering: Option<vk::PhysicalDeviceDynamicRenderingFeaturesKHR<'static>>,
}

impl PhysicalDeviceFeaturesFfi {
    pub fn wire_to_properties<'a>(
        &'a mut self,
        api_version: Version,
        extensions: &DeviceExtensions,
        mut properties: vk::PhysicalDeviceFeatures2<'a>,
    ) -> vk::PhysicalDeviceFeatures2<'a> {
        // Vulkan 1.1 features
        if api_version >= Version::V1_1 {
            let vk11 = self.vk11.get_or_insert_default();
            properties = properties.push_next(vk11);
        }

        // Vulkan 1.2 features
        if api_version >= Version::V1_2 {
            let vk12 = self.vk12.get_or_insert_default();
            properties = properties.push_next(vk12);
        }

        // Vulkan 1.3 features
        if api_version >= Version::V1_3 {
            let vk13 = self.vk13.get_or_insert_default();
            properties = properties.push_next(vk13);
        }

        properties
    }

    pub fn wire_to_create_info<'a>(
        &'a mut self,
        api_version: Version,
        extensions: &DeviceExtensions,
        features: &DeviceFeatures,
        mut create_info: vk::DeviceCreateInfo<'a>,
    ) -> vk::DeviceCreateInfo<'a> {
        // Vulkan 1.1 features
        if api_version >= Version::V1_1 {
            let vk11 = self.vk11.get_or_insert_default();
            vk11.read_from_device_features(features);
            create_info = create_info.push_next(vk11);
        }
        if api_version >= Version::V1_2 {
            let vk12 = self.vk12.get_or_insert_default();
            vk12.read_from_device_features(features);
            create_info = create_info.push_next(vk12);
        }
        if api_version >= Version::V1_3 {
            let vk13 = self.vk13.get_or_insert_default();
            vk13.read_from_device_features(features);
            create_info = create_info.push_next(vk13);
        }

        create_info
    }

    pub fn into_device_features(self) -> DeviceFeatures {
        let mut features = DeviceFeatures::empty();
        if let Some(vk11) = self.vk11 {
            vk11.write_to_device_features(&mut features);
        }
        if let Some(vk12) = self.vk12 {
            vk12.write_to_device_features(&mut features);
        }
        if let Some(vk13) = self.vk13 {
            vk13.write_to_device_features(&mut features);
        }
        features
    }
}

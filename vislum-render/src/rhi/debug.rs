use ash::vk;

pub(crate) extern "system" fn debug_trampoline(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { &*p_callback_data };

    let message = match unsafe { callback_data.message_as_c_str() } {
        Some(message) => message.to_str().unwrap_or_default(),
        None => "",
    };

    // log::info!("Debug callback: {message}");
    vk::TRUE
}
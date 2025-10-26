use std::borrow::Cow;

use ash::{prelude::VkResult, vk};

pub mod buffer;
pub mod command;
pub mod descriptor;
pub mod device;
pub mod image;
pub mod instance;
pub mod macros;
pub mod memory;
pub mod pipeline;
pub mod queue;
pub mod surface;
pub mod swapchain;
pub mod sync;
pub mod version;

pub trait VkHandle {
    type Handle: ash::vk::Handle;

    /// Returns the inner handle of the object.
    fn vk_handle(&self) -> Self::Handle;

    #[inline]
    fn vk_object_type(&self) -> ash::vk::ObjectType {
        <Self::Handle as ash::vk::Handle>::TYPE
    }
}

pub trait VkRawHandle {
    fn vk_raw_handle(&self) -> u64;
}

impl<T> VkRawHandle for T
where
    T: VkHandle,
{
    #[inline]
    fn vk_raw_handle(&self) -> u64 {
        ash::vk::Handle::as_raw(self.vk_handle())
    }
}

pub trait AshHandle {
    type Handle;

    fn ash_handle(&self) -> &Self::Handle;
}

enum ErrorSource {
    Vulkan(ash::vk::Result),
    Other(Box<dyn std::error::Error>),
}

impl std::fmt::Display for ErrorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSource::Vulkan(result) => write!(f, "vulkan error: {}", result),
            ErrorSource::Other(error) => write!(f, "other error: {}", error),
        }
    }
}

/// An RHI error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("vulkan error: {context}: {result}")]
    Vulkan {
        context: Cow<'static, str>,
        result: ash::vk::Result,
    },
}

pub trait WithContext<T> {
    fn with_context(self, context: impl Into<Cow<'static, str>>) -> Result<T, Error>
    where
        Self: Sized,
    {
        self.with_context_fn(|| context.into())
    }

    fn with_context_fn<F, S>(self, context: F) -> Result<T, Error>
    where 
        F: FnOnce() -> S,
        S: Into<Cow<'static, str>>;
}

impl<T> WithContext<T> for Result<T, ash::vk::Result> {
    fn with_context_fn<F, S>(self, context: F) -> Result<T, Error>
    where 
        F: FnOnce() -> S,
        S: Into<Cow<'static, str>> {
        self.map_err(|result| Error::Vulkan {
            context: context().into(),
            result,
        })
    }
}
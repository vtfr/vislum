use core::fmt;
use std::{fmt::Debug, sync::Arc};

use crate::resources::id::{HandleInner, Resource, ResourceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureDescription {
    pub dimensions: TextureDimensions,
}

pub struct Texture {
    pub description: TextureDescription,
}

impl Resource for Texture {
    const TYPE: ResourceType = ResourceType::Texture;
}

#[derive(Clone)]
pub struct TextureHandle(Arc<HandleInner<Texture, TextureDimensions>>);

impl TextureHandle {
    pub fn dimensions(&self) -> TextureDimensions {
        *self.0.user_data()
    }
}

impl Debug for TextureHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextureHandle")
            .field("id", &self.0.id())
            .field("dimensions", &self.dimensions())
            .finish()
    }
}

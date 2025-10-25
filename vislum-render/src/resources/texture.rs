use core::fmt;
use std::{fmt::Debug, sync::Arc};

use vulkano::image::{Image, view::ImageView};

use crate::resources::id::{ErasedResourceId, HandleInner, Resource, ResourceId, ResourceType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TextureDescription {
    pub dimensions: TextureDimensions,
}

/// A texture resource.
pub struct Texture {
    pub(crate) description: TextureDescription,
    pub(crate) image: Arc<Image>,
    pub(crate) default_view: Arc<ImageView>,
}

impl Resource for Texture {
    const TYPE: ResourceType = ResourceType::Texture;
}

impl Texture {
    /// Returns the description of the texture.
    #[inline]
    pub fn description(&self) -> &TextureDescription {
        &self.description
    }

    /// Returns the underlying image.
    #[inline]
    pub fn image(&self) -> &Arc<Image> {
        &self.image
    }

    /// Returns the default view of the texture.
    #[inline]
    pub fn default_view(&self) -> &Arc<ImageView> {
        &self.default_view
    }
}

#[derive(Clone)]
pub struct TextureHandle(Arc<HandleInner<Texture, TextureDescription>>);

impl TextureHandle {
    pub(crate) fn new(
        id: ResourceId<Texture>,
        description: TextureDescription,
        resource_drop_tx: crossbeam_channel::Sender<ErasedResourceId>,
    ) -> Self {
        Self(Arc::new(HandleInner::new(
            id,
            description,
            resource_drop_tx,
        )))
    }

    pub fn dimensions(&self) -> TextureDimensions {
        self.0.user_data().dimensions
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

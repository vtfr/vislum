
use std::{marker::PhantomData, sync::Arc};

use slotmap::SlotMap;
use vulkano::{
    device::Device,
    image::{Image, ImageCreateInfo, ImageUsage, view::ImageView},
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
};

slotmap::new_key_type! {
    struct ResourceKey;
}

pub struct ResourceId<T> {
    key: ResourceKey,
    phantom: PhantomData<fn() -> T>,
}

impl<T> Copy for ResourceId<T> {}

impl<T> Clone for ResourceId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for ResourceId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> Eq for ResourceId<T> {}

impl<T> PartialOrd for ResourceId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T> Ord for ResourceId<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<T> std::hash::Hash for ResourceId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

impl<T> std::fmt::Debug for ResourceId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceId")
            .field("key", &self.key)
            .finish()
    }
}

pub(crate) struct ResourcePool<T> {
    resources: SlotMap<ResourceKey, T>,
}

impl<T> Default for ResourcePool<T> {
    fn default() -> Self {
        Self {
            resources: Default::default(),
        }
    }
}

impl<T> ResourcePool<T> {
    pub fn insert(&mut self, resource: T) -> ResourceId<T> {
        let key = self.resources.insert(resource);
        ResourceId {
            key,
            phantom: PhantomData,
        }
    }

    pub fn get(&self, id: ResourceId<T>) -> Option<&T> {
        self.resources.get(id.key)
    }

    pub fn get_mut(&mut self, id: ResourceId<T>) -> Option<&mut T> {
        self.resources.get_mut(id.key)
    }
}
use std::{cmp::Ordering, hash::{Hash, Hasher}, marker::PhantomData};

use slotmap::{DefaultKey, SlotMap};

/// The identifier for a resource of type `T`.
pub struct ResourceId<T> {
    key: DefaultKey,
    phantom: PhantomData<T>,
}

impl<T> PartialEq for ResourceId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<T> Eq for ResourceId<T> {}

impl<T> Hash for ResourceId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<T> PartialOrd for ResourceId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl<T> Default for ResourceStorage<T> {
    fn default() -> Self {
        Self {
            resources: SlotMap::new(),
        }
    }
}

/// A generic storage for render resources, such as meshes, textures, etc.
pub struct ResourceStorage<T> {
    resources: SlotMap<DefaultKey, T>,
}

impl<T> ResourceStorage<T> {
    #[inline]
    pub fn insert(&mut self, resource: T) -> ResourceId<T> {
        let key = self.resources.insert(resource);
        ResourceId { key, phantom: PhantomData }
    }

    #[inline]
    pub fn get(&self, resource: ResourceId<T>) -> Option<&T> {
        self.resources.get(resource.key)
    }

    #[inline]
    pub fn get_mut(&mut self, resource: ResourceId<T>) -> Option<&mut T> {
        self.resources.get_mut(resource.key)
    }

    #[inline]
    pub fn remove(&mut self, resource: ResourceId<T>) -> Option<T> {
        self.resources.remove(resource.key)
    }

    #[inline]
    pub fn clear(&mut self) {
        self.resources.clear();
    }
}
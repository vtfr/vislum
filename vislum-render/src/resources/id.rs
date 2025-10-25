use std::{hash::Hash, marker::PhantomData};

use slotmap::SlotMap;

slotmap::new_key_type! {
    /// The inner key used to store and retrieve resources.
    ///
    /// # Safety
    /// This is not safe to be used across different resource managers.
    struct ResourceKey;
}

pub(crate) struct ResourceId<T> {
    key: ResourceKey,
    phantom: PhantomData<fn() -> T>,
}

impl<T> ResourceId<T> {
    #[inline]
    fn key(&self) -> ResourceKey {
        self.key
    }

    #[inline]
    pub fn erase(&self) -> ErasedResourceId
    where 
        T: Resource,
    {
        ErasedResourceId { ty: T::TYPE, key: self.key }
    }
}

impl<T> std::fmt::Debug for ResourceId<T> 
where 
    T: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceId<{}>({:?})", std::any::type_name::<T>(), self.key)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Texture,
}

pub(crate) trait Resource {
    const TYPE: ResourceType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErasedResourceId {
    ty: ResourceType,
    key: ResourceKey,
}

impl ErasedResourceId {
    #[inline]
    pub fn ty(&self) -> ResourceType {
        self.ty
    }

    #[inline]
    fn key(&self) -> ResourceKey {
        self.key
    }

    #[inline]
    pub fn cast<T>(&self) -> Option<ResourceId<T>>
    where 
        T: Resource,
    {
        if self.ty == T::TYPE {
            Some(ResourceId { key: self.key, phantom: PhantomData })
        } else {
            None
        }
    }

    #[inline]
    pub fn cast_unchecked<T>(&self) -> ResourceId<T>
    where 
        T: Resource,
    {
        ResourceId { key: self.key, phantom: PhantomData }
    }
}

pub(crate) struct ResourceStorage<T> {
    resources: SlotMap<ResourceKey, T>,
}

impl<T> Default for ResourceStorage<T> {
    #[inline]
    fn default() -> Self {
        Self { resources: Default::default() }
    }
}

impl<T> ResourceStorage<T> {
    /// Insert a new resource into the storage.
    pub fn insert(&mut self, value: T) -> ResourceId<T> {
        let key = self.resources.insert(value);
        ResourceId { key, phantom: PhantomData }
    }

    /// Remove a resource from the storage.
    pub fn remove(&mut self, id: ResourceId<T>) -> Option<T> {
        self.resources.remove(id.key())
    }

    /// Get a resource from the storage.
    pub fn get(&self, id: ResourceId<T>) -> Option<&T> {
        self.resources.get(id.key())
    }

    /// Get a mutable reference to a resource from the storage.
    pub fn get_mut(&mut self, id: ResourceId<T>) -> Option<&mut T> {
        self.resources.get_mut(id.key())
    }

    /// Iterate over all resources in the storage.
    pub fn iter(&self) -> impl ExactSizeIterator<Item=ResourceId<T>> {
        self.resources.iter()
            .map(|(key, _)| ResourceId { key, phantom: PhantomData })
    }
}

/// The base for a resource handle of any type.
pub(crate) struct HandleInner<T: Resource, U: 'static> {
    id: ResourceId<T>,
    user_data: U,
    drop_notifier: crossbeam_channel::Sender<ErasedResourceId>,
}

impl<T, U> HandleInner<T, U> 
where 
    T: Resource 
{
    #[inline]
    pub fn new(id: ResourceId<T>, user_data: U, drop_notifier: crossbeam_channel::Sender<ErasedResourceId>) -> Self {
        Self { id, user_data, drop_notifier }
    }

    #[inline]
    pub fn id(&self) -> ResourceId<T> {
        self.id
    }

    #[inline]
    pub fn user_data(&self) -> &U {
        &self.user_data
    }
}

impl<T, U> PartialEq for HandleInner<T, U> 
where 
    T: Resource,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T, U> Eq for HandleInner<T, U> 
where 
    T: Resource,
{ }

impl<T, U> Drop for HandleInner<T, U> 
where 
    T: Resource,
{
    fn drop(&mut self) {
        let _ = self.drop_notifier.send(self.id.erase());
    }
}

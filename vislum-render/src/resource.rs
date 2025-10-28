use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use slotmap::SlotMap;

slotmap::new_key_type! {
    /// A unique identifier for a resource.
    pub(crate) struct ResourceKey;
}

/// A weak-reference to a resource.
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

impl<T> Debug for ResourceId<T>
where
    T: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ResourceId<{}>({:?})",
            std::any::type_name::<T>(),
            self.key
        )
    }
}

impl<T> ResourceId<T> {
    #[inline]
    pub(crate) const fn new(key: ResourceKey) -> Self {
        Self {
            key,
            phantom: PhantomData,
        }
    }
}

struct HandleInner<T> {
    id: ResourceId<T>,
    phantom: PhantomData<fn() -> T>,
    resource_drop_tx: crossbeam_channel::Sender<ResourceId<T>>,
}

impl<T> Drop for HandleInner<T> {
    fn drop(&mut self) {
        // Notify the resource storage that the resource has been dropped.
        let _ = self.resource_drop_tx.send(self.id);
    }
}

// A strong handle to a resource.
pub struct Handle<T>(Arc<HandleInner<T>>);

impl<T> Handle<T> {
    #[inline]
    pub fn id(&self) -> ResourceId<T> {
        self.0.id
    }
}

impl<T> Debug for Handle<T>
where
    T: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Handle<{}>({:?})",
            std::any::type_name::<T>(),
            self.0.id.key
        )
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl<T> Eq for Handle<T> {}

pub trait IntoResourceId<T> {
    fn into_resource_id(&self) -> ResourceId<T>;
}

impl<T> IntoResourceId<T> for Handle<T> {
    fn into_resource_id(&self) -> ResourceId<T> {
        self.id()
    }
}

impl<T> IntoResourceId<T> for &Handle<T> {
    fn into_resource_id(&self) -> ResourceId<T> {
        self.id()
    }
}

impl<T> IntoResourceId<T> for ResourceId<T> {
    fn into_resource_id(&self) -> ResourceId<T> {
        *self
    }
}

pub struct ResourceStorage<T> {
    resource_drop_tx: crossbeam_channel::Sender<ResourceId<T>>,
    resource_drop_rx: crossbeam_channel::Receiver<ResourceId<T>>,
    resources: SlotMap<ResourceKey, T>,
}

impl<T> Default for ResourceStorage<T> {
    fn default() -> Self {
        let (resource_drop_tx, resource_drop_rx) = crossbeam_channel::unbounded();
        Self {
            resource_drop_tx,
            resource_drop_rx,
            resources: Default::default(),
        }
    }
}

impl<T> ResourceStorage<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, resource: T) -> Handle<T> {
        let key = self.resources.insert(resource);
        Handle(Arc::new(HandleInner {
            id: ResourceId::new(key),
            phantom: PhantomData,
            resource_drop_tx: self.resource_drop_tx.clone(),
        }))
    }

    /// Inserts a new resource with the given function.
    ///
    /// The function is called with the [`ResourceId<T>`] of the resource being added to the storage..
    pub fn insert_with_it<F>(&mut self, f: F) -> Handle<T>
    where
        F: FnOnce(ResourceId<T>) -> T,
    {
        let key = self
            .resources
            .insert_with_key(|key| f(ResourceId::new(key)));
        let id = ResourceId::new(key);

        Handle(Arc::new(HandleInner {
            id,
            phantom: PhantomData,
            resource_drop_tx: self.resource_drop_tx.clone(),
        }))
    }

    pub fn get(&self, id: impl IntoResourceId<T>) -> Option<&T> {
        let id = id.into_resource_id();
        self.resources.get(id.key)
    }

    pub fn get_mut(&mut self, id: impl IntoResourceId<T>) -> Option<&mut T> {
        let id = id.into_resource_id();
        self.resources.get_mut(id.key)
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (ResourceId<T>, &T)> {
        self.resources
            .iter()
            .map(|(key, resource)| (ResourceId::new(key), resource))
    }

    /// Flushes the drop events for the storage, returning all the resources that have been dropped.
    pub fn flush_drop_events<'a>(&'a mut self) -> DropEvents<T> {
        let iter = self.resource_drop_rx.try_iter();

        DropEvents {
            iter,
            slotmap: &mut self.resources,
        }
    }

}

pub struct DropEvents<'a, T> {
    iter: crossbeam_channel::TryIter<'a, ResourceId<T>>,
    slotmap: &'a mut SlotMap<ResourceKey, T>,
}

impl<'a, T> Iterator for DropEvents<'a, T> {
    type Item = (ResourceId<T>, T);

    fn next(&mut self) -> Option<Self::Item> {
        // Iterate the dropped handles.
        while let Some(id) = self.iter.next() {
            // Try to remove the resource from the slotmap.
            //
            // If the resource is not found, this means we fucked up
            // somehow, as the only possible way to remove resources
            // is when the handle is dropped, and it reaches this
            // code.
            //
            // Nevertheless, better safe than sorry.
            if let Some(resource) = self.slotmap.remove(id.key) {
                return Some((id, resource));
            }
        }

        None
    }
}
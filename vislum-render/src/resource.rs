use std::{hash::{Hash, Hasher}, marker::PhantomData, sync::Arc};

use crossbeam::channel::{Receiver, Sender};
use slotmap::{DefaultKey, SlotMap};

// A non-owning reference to a given resource.
pub struct ResourceId<T> {
    key: DefaultKey,
    phantom: PhantomData<fn() -> T>,
}

impl<T> ResourceId<T> {
    /// Creates a new resource ID.
    pub(crate) fn new(id: DefaultKey) -> Self {
        Self { key: id, phantom: Default::default() }
    }
}

impl<T> std::fmt::Debug for ResourceId<T> {
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

impl<T> Hash for ResourceId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

/// An event sent when a handle to a resource is dropped.
/// 
/// Emitted when all [`Handle`]s to a resource are dropped.
pub struct HandleDropEvent<T> {
    id: ResourceId<T>, 
    phantom: PhantomData<T>,
}

impl<T> HandleDropEvent<T> {
    /// Gets the ID of the resource that was dropped.
    pub fn id(&self) -> ResourceId<T> {
        self.id
    }
}

struct HandleInner<T> {
    id: ResourceId<T>,
    drop_tx: Sender<HandleDropEvent<T>>,
}

impl<T> Drop for HandleInner<T> {
    fn drop(&mut self) {
        let _ = self.drop_tx.send(HandleDropEvent{
            id: self.id,
            phantom: Default::default(),
        });
    }
}

/// A owned handle to a rendering resource.
pub struct Handle<T> {
    inner: Arc<HandleInner<T>>,
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle<{}>({:?})", std::any::type_name::<T>(), self.inner.id.key)
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.id == other.inner.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.id.hash(state);
    }
}

pub trait IntoResourceId<T> {
    fn into_resource_id(&self) -> ResourceId<T>;
}

impl<T> IntoResourceId<T> for ResourceId<T> {
    fn into_resource_id(&self) -> ResourceId<T> {
        *self
    }
}

impl<T> IntoResourceId<T> for Handle<T> {
    fn into_resource_id(&self) -> ResourceId<T> {
        self.inner.id
    }
}

impl<T> IntoResourceId<T> for &'_ Handle<T> {
    fn into_resource_id(&self) -> ResourceId<T> {
        self.inner.id
    }
}

/// A storage for resources.
pub struct ResourceStorage<T> {
    drop_tx: Sender<HandleDropEvent<T>>,
    drop_rx: Receiver<HandleDropEvent<T>>,
    storage: SlotMap<DefaultKey, T>,
}

impl<T> ResourceStorage<T> {
    /// Creates a new resource storage.
    pub fn new() -> Self {
        let (drop_tx, drop_rx) = crossbeam::channel::unbounded();

        Self {
            storage: SlotMap::new(),
            drop_tx,
            drop_rx,
        }
    }

    /// Gets a reference to a resource.
    pub fn get(&self, id: impl IntoResourceId<T>) -> Option<&T> {
        let id = id.into_resource_id();
        self.storage.get(id.key)
    }

    /// Gets a mutable reference to a resource.
    pub fn get_mut(&mut self, id: impl IntoResourceId<T>) -> Option<&mut T> {
        let id = id.into_resource_id();
        self.storage.get_mut(id.key)
    }

    /// Creates a new resource.
    pub fn insert(&mut self, data: T) -> Handle<T> {
        let id = self.storage.insert(data);
        Handle {
            inner: Arc::new(HandleInner {
                id: ResourceId::new(id),
                drop_tx: self.drop_tx.clone(),
            }),
        }
    }

    /// Gets an iterator over the IDs of the resources that were dropped.
    pub fn drop_events(&mut self) -> DropEventsIter<'_, T> {
        DropEventsIter {
            receiver: &self.drop_rx,
        }
    }
}

impl<T> Default for ResourceStorage<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DropEventsIter<'a, T> {
    receiver: &'a Receiver<HandleDropEvent<T>>,
}

impl<'a, T> Iterator for DropEventsIter<'a, T> {
    type Item = ResourceId<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(event) = self.receiver.try_recv() {
            Some(event.id)
        } else {
            None
        }
    }
}

// A non-owning reference to a given resource.
pub struct ResourceId<T> {
    id: usize,
    phantom: PhantomData,
}

pub struct HandleDropEvent<T> {
    id: ResourceId<T>, 
    phantom: PhantomData<T>,
}

impl<T> HandleDropEvent<T> {
    pub fn id(&self) -> usize {
        self.id
    }
}

struct HandleInner<T> {
    id: ResourceId<T>,
    drop_tx: Sender<HandleDropEvent<T>>,
}

impl<T> Drop for HandleInner<T> {
    fn drop(self) {
        self.drop_tx.send(HandleDropEvent{
            id: self.id,
            phantom: Default::default(),
        })
    }
}

/// A owned handle to a rendering resource.
pub struct Handle<T> {
    inner: Arc<HandleInner<T>>,
}

pub struct ResourceStorage<T> {
    storage: SlotMap<KeyData, T>
}

impl<T> ResourceStorage<T> {
    /// Gets a reference to a resource.
    pub fn get(&self, id: impl Into<ResourceId>) {
        todo!()
    }

    /// Gets a mutable reference to a resource.
    pub fn get_mut(&mut self, id: impl Into<ResourceId>) {
        todo!()
    }

    /// Creates a new resource.
    pub fn insert(&mut self, data: T) -> Handle<T> {
        todo!()
    }
}

// impl shit for Handle<T> despite T being phantom here.

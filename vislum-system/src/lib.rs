extern crate self as vislum_system;

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut, UnsafeCell};
use std::collections::HashMap;

// Re-export the System macro.
pub use vislum_system_macros::Resource;

/// A marker trait for identifying resources.
pub trait Resource: 'static + Any {}

/// A reference to a resource.
pub struct Res<'a, T>(Ref<'a, T>);

impl<'a, T> std::ops::Deref for Res<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A mutable reference to a resource.
pub struct ResMut<'a, T>(RefMut<'a, T>);

impl<'a, T> std::ops::Deref for ResMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for ResMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct ErasedResourceCell(Box<RefCell<dyn Any>>);

impl ErasedResourceCell {
    /// Creates a new shared system cell.
    #[inline]
    pub fn new<T>(system: T) -> Self
    where
        T: 'static,
    {
        Self(Box::new(RefCell::new(system)))
    }

    /// Get a reference to a system.
    ///
    /// # Safety
    ///
    /// Assumes that the caller has checked that the system is of the correct type.
    #[inline]
    pub fn get_downcasted_ref<T>(&self) -> Res<'_, T>
    where
        T: 'static,
    {
        let system = match self.0.try_borrow() {
            Ok(system) => system,
            Err(_) => resource_already_borrowed(std::any::type_name::<T>()),
        };

        let system = Ref::map(system, |s| match s.downcast_ref::<T>() {
            Some(system) => system,
            None => incompatible_resource_downcast(std::any::type_name::<T>()),
        });

        Res(system)
    }

    /// Gets a mutable reference to a system.
    ///
    /// # Safety
    ///
    /// Assumes that the caller has checked that the system is of the correct type.
    #[inline]
    pub fn get_downcasted_mut<T>(&self) -> ResMut<'_, T>
    where
        T: 'static,
    {
        let system = match self.0.try_borrow_mut() {
            Ok(system) => system,
            Err(_) => resource_already_borrowed(std::any::type_name::<T>()),
        };

        let system = RefMut::map(system, |s| match s.downcast_mut::<T>() {
            Some(system) => system,
            None => incompatible_resource_downcast(std::any::type_name::<T>()),
        });

        ResMut(system)
    }
}

#[derive(Default)]
pub struct Resources {
    resources: UnsafeCell<HashMap<TypeId, ErasedResourceCell>>,
}

impl std::fmt::Debug for Resources {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Resources")
    }
}

impl Resources {
    /// Creates a new systems.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets a system by type, panicking if the system is not found.
    pub fn get<T>(&self) -> Res<'_, T>
    where
        T: Resource,
    {
        // SAFETY: We're not mutating the inner HashMap.
        let inner = unsafe { self.inner() };

        match inner.get(&std::any::TypeId::of::<T>()) {
            Some(res) => res.get_downcasted_ref::<T>(),
            None => resource_not_found(std::any::type_name::<T>()),
        }
    }

    /// Gets a mutable system by type, panicking if the system is not found.
    pub fn get_mut<T>(&self) -> ResMut<'_, T>
    where
        T: Resource,
    {
        // SAFETY: We're not mutating the inner HashMap.
        let inner = unsafe { self.inner() };

        match inner.get(&std::any::TypeId::of::<T>()) {
            Some(res) => res.get_downcasted_mut::<T>(),
            None => resource_not_found(std::any::type_name::<T>()),
        }
    }

    /// Gets a resource by type, inserting a default resource if it is not found.
    pub fn get_or_insert_default<T>(&self) -> Res<'_, T>
    where
        T: Resource + Default,
    {
        // SAFETY: We only insert a new resource if it is not found, so no borrows to the
        // previous resource can be invalidated (as the resource does not exist). Moreover, the
        // HashMap stores a pointer to the resource, so we are guaranteed that changes within the
        // HashMap storage will invalidate other borrows.
        let inner = unsafe { self.inner() };

        inner
            .entry(TypeId::of::<T>())
            .or_insert_with(|| ErasedResourceCell::new(T::default()))
            .get_downcasted_ref::<T>()
    }

    /// Gets a mutable resource by type, inserting a default resource if it is not found.
    pub fn get_mut_or_insert_default<T>(&self) -> ResMut<'_, T>
    where
        T: Resource + Default,
    {
        // SAFETY: We only insert a new resource if it is not found, so no borrows to the
        // previous resource can be invalidated (as the resource does not exist). Moreover, the
        // HashMap stores a pointer to the resource, so we are guaranteed that changes within the
        // HashMap storage will invalidate other borrows.
        let inner = unsafe { self.inner() };

        inner
            .entry(TypeId::of::<T>())
            .or_insert_with(|| ErasedResourceCell::new(T::default()))
            .get_downcasted_mut::<T>()
    }

    /// Inserts a system.
    pub fn insert<T>(&mut self, resource: T)
    where
        T: Resource,
    {
        // SAFETY: We have exclusive access to the resources, so no borrows are possible.
        let inner = unsafe { self.inner() };

        inner.insert(TypeId::of::<T>(), ErasedResourceCell::new(resource));
    }

    /// Inserts a default system.
    pub fn insert_default<T>(&mut self)
    where
        T: Resource + Default,
    {
        self.insert(T::default());
    }

    unsafe fn inner(&self) -> &mut HashMap<TypeId, ErasedResourceCell> {
        unsafe { &mut *self.resources.get() }
    }
}

#[cold]
fn resource_already_borrowed(type_name: &str) -> ! {
    panic!("Resource already borrowed: {}", type_name)
}

#[cold]
fn incompatible_resource_downcast(type_name: &str) -> ! {
    panic!("Resource is not of type {}", type_name)
}

#[cold]
fn resource_not_found(type_name: &str) -> ! {
    panic!("Resource not found: {}", type_name)
}

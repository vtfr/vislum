extern crate self as vislum_system;

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;

// Re-export the System macro.
pub use vislum_system_macros::System;

/// Marker trait for systems.
pub trait System {
    /// Returns a reference to the system
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the system
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Returns the type name of the system
    fn type_name(&self) -> &str;
}

/// A reference to a system.
pub struct SysRef<'a, T>(Ref<'a, T>);

impl<'a, T> std::ops::Deref for SysRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A mutable reference to a system.
pub struct SysMut<'a, T>(RefMut<'a, T>);

impl<'a, T> std::ops::Deref for SysMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> std::ops::DerefMut for SysMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

struct ErasedSystemCell(Box<RefCell<dyn System>>);

impl Debug for ErasedSystemCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ErasedSystemCell")
    }
}

impl ErasedSystemCell {
    /// Creates a new shared system cell.
    #[inline]
    pub fn new<T>(system: T) -> Self
    where
        T: System + 'static,
    {
        Self(Box::new(RefCell::new(system)))
    }

    /// Get a reference to a system.
    ///
    /// # Safety
    ///
    /// Assumes that the caller has checked that the system is of the correct type.
    #[inline]
    pub fn get_downcasted_ref<T>(&self) -> SysRef<T>
    where
        T: System + 'static,
    {
        let system = match self.0.try_borrow() {
            Ok(system) => system,
            Err(_) => system_already_borrowed(std::any::type_name::<T>()),
        };

        let system = Ref::map(system, |s| match s.as_any().downcast_ref::<T>() {
            Some(system) => system,
            None => incompatible_system_downcast(std::any::type_name::<T>()),
        });

        SysRef(system)
    }

    /// Gets a mutable reference to a system.
    ///
    /// # Safety
    ///
    /// Assumes that the caller has checked that the system is of the correct type.
    #[inline]
    pub fn get_downcasted_mut<T>(&self) -> SysMut<T>
    where
        T: System + 'static,
    {
        let system = match self.0.try_borrow_mut() {
            Ok(system) => system,
            Err(_) => system_already_borrowed(std::any::type_name::<T>()),
        };

        let system = RefMut::map(system, |s| match s.as_any_mut().downcast_mut::<T>() {
            Some(system) => system,
            None => incompatible_system_downcast(std::any::type_name::<T>()),
        });

        SysMut(system)
    }
}

#[derive(Default)]
pub struct Systems {
    systems: HashMap<TypeId, ErasedSystemCell>,
}

impl Systems {
    /// Creates a new systems.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets a system by type.
    pub fn get<T>(&self) -> Option<SysRef<T>>
    where
        T: System + 'static,
    {
        let system = self.systems.get(&std::any::TypeId::of::<T>())?;
        Some(system.get_downcasted_ref::<T>())
    }

    /// Gets a mutable system by type.
    pub fn get_mut<T>(&self) -> Option<SysMut<T>>
    where
        T: System + 'static,
    {
        let system = self.systems.get(&TypeId::of::<T>())?;
        Some(system.get_downcasted_mut::<T>())
    }

    /// Gets a system by type, panicking if the system is not found.
    pub fn must_get<T>(&self) -> SysRef<T>
    where
        T: System + 'static,
    {
        match self.get::<T>() {
            Some(system) => system,
            None => system_not_found(std::any::type_name::<T>()),
        }
    }

    /// Gets a mutable system by type, panicking if the system is not found.
    pub fn must_get_mut<T>(&self) -> SysMut<T>
    where
        T: System + 'static,
    {
        match self.get_mut::<T>() {
            Some(system) => system,
            None => system_not_found(std::any::type_name::<T>()),
        }
    }

    /// Inserts a system.
    pub fn insert<T>(&mut self, system: T)
    where
        T: System + 'static,
    {
        self.systems
            .insert(TypeId::of::<T>(), ErasedSystemCell::new(system));
    }

    /// Inserts a default system.
    pub fn insert_default<T>(&mut self)
    where
        T: System + Default + 'static,
    {
        self.insert(T::default());
    }
}

#[cold]
fn system_already_borrowed(type_name: &str) -> ! {
    panic!("System already borrowed: {}", type_name)
}

#[cold]
fn incompatible_system_downcast(type_name: &str) -> ! {
    panic!("System is not of type {}", type_name)
}

#[cold]
fn system_not_found(type_name: &str) -> ! {
    panic!("System not found: {}", type_name)
}

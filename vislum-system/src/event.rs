use std::{any::TypeId, cell::{Ref, RefCell, RefMut, UnsafeCell}, collections::HashMap};

use downcast_rs::Downcast;

use crate::Resource;

pub trait Event {}

pub(crate) trait DynEventQueue: Downcast {
    /// Clears the event queue.
    fn clear(&mut self);
}

downcast_rs::impl_downcast!(DynEventQueue);

pub struct EventQueue<T> {
    queue: Vec<T>,
}

impl<T> Default for EventQueue<T> {
    fn default() -> Self {
        Self { queue: Vec::new() }
    }
}

impl<T> DynEventQueue for EventQueue<T>
where
    T: 'static,
{
    fn clear(&mut self) {
        self.queue.clear();
    }
}

/// A system for managing events.
#[derive(Resource)]
pub struct EventBusSystem {
    queues: UnsafeCell<HashMap<TypeId, Box<RefCell<dyn DynEventQueue>>>>,
}

impl EventBusSystem {
    pub fn read<T: Event>(&self) -> EventReader<'_, T>
    where 
        T: 'static,
    {
        let events = unsafe { &mut *self.queues.get() };
        let queue = events.entry(TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(RefCell::new(EventQueue::<T>::default()))
            })
            .borrow();

        let queue = Ref::map(queue, |queue| {
            unsafe {
                queue.downcast_ref::<EventQueue<T>>().unwrap_unchecked()
            }
        });

        EventReader::new(queue)
    }

    pub fn write<T: Event>(&self) -> EventWriter<'_, T>
    where 
        T: 'static,
    {
        let events = unsafe { &mut *self.queues.get() };
        let queue = events.entry(TypeId::of::<T>())
            .or_insert_with(|| {
                Box::new(RefCell::new(EventQueue::<T>::default()))
            })
            .borrow_mut();

        let queue = RefMut::map(queue, |queue| {
            unsafe {
                queue.downcast_mut::<EventQueue<T>>().unwrap_unchecked()
            }
        });

        EventWriter::new(queue)
    }

    pub fn clear<T: Event>(&self)
    where 
        T: 'static,
    {
        let events = unsafe { &mut *self.queues.get() };
        if let Some(queue) = events.get(&TypeId::of::<T>()) {
            queue.borrow_mut().clear();
        }
    }

    pub fn clear_all(&self) {
        let events = unsafe { &mut *self.queues.get() };
        for queue in events.values_mut() {
            queue.borrow_mut().clear();
        }
    }
}

pub struct EventReader<'a, T> {
    events: Ref<'a, EventQueue<T>>,
    index: usize,
}

impl<'a, T> EventReader<'a, T> {
    pub(crate) fn new(events: Ref<'a, EventQueue<T>>) -> Self {
        Self { 
            events,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for EventReader<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.events.queue.len() {
            let event = &self.events.queue[self.index];
            self.index += 1;
            // SAFETY: We know the Ref<'a, EventQueue<T>> keeps the data alive for lifetime 'a
            // and we're returning a reference to data within that structure
            Some(unsafe { &*(event as *const T) })
        } else {
            None
        }
    }
}

pub struct EventWriter<'a, T> {
    events: RefMut<'a, EventQueue<T>>,
}

impl<'a, T> EventWriter<'a, T> {
    pub(crate) fn new(events: RefMut<'a, EventQueue<T>>) -> Self {
        Self { events }
    }

    pub fn add(&mut self, event: T) {
        self.events.queue.push(event);
    }

    pub fn extend(&mut self, events: impl IntoIterator<Item = T>) {
        self.events.queue.extend(events);
    }
}
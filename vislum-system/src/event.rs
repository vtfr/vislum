pub trait Event {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventId(pub(crate) u64);

pub struct EventBus<T> {
    /// The last event ID.
    last_event_id: EventId,

    /// The previous events.
    previous: Vec<(EventId, T)>,

    // The current events.
    current: Vec<(EventId, T)>,

    // The next events.
    next: Vec<(EventId, T)>,
}

impl<T> EventBus<T> {
    pub fn finish(&self) -> Self {
    }
}

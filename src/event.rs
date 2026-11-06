use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    num::NonZeroUsize,
    sync::Mutex,
};
use tokio::sync::broadcast;

/// A per-type broadcast bus. Tokio rounds the capacity up to a power of two;
/// allocation failure may terminate the process. Events without receivers are discarded.
/// Receivers expose Tokio's lag and closure errors without modification.
pub struct EventBus {
    capacity: usize,
    channels: Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

/// The requested capacity cannot be rounded up by Tokio.
#[derive(Debug)]
pub struct InvalidCapacity;
impl fmt::Display for InvalidCapacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("event capacity is too large")
    }
}
impl std::error::Error for InvalidCapacity {}

impl Default for EventBus {
    fn default() -> Self {
        Self {
            capacity: 256,
            channels: Mutex::new(HashMap::new()),
        }
    }
}
impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(capacity: NonZeroUsize) -> Result<Self, InvalidCapacity> {
        if capacity.get() > usize::MAX / 2 {
            return Err(InvalidCapacity);
        }
        Ok(Self {
            capacity: capacity.get(),
            channels: Mutex::new(HashMap::new()),
        })
    }
    fn sender<E: 'static + Send + Sync + Clone>(&self) -> broadcast::Sender<E> {
        let mut channels = self.channels.lock().unwrap_or_else(|e| e.into_inner());
        let sender = channels.entry(TypeId::of::<E>()).or_insert_with(|| {
            let (sender, receiver) = broadcast::channel::<E>(self.capacity);
            drop(receiver);
            Box::new(sender)
        });
        sender
            .downcast_ref::<broadcast::Sender<E>>()
            .expect("event type invariant violated")
            .clone()
    }
    pub fn subscribe<E: 'static + Send + Sync + Clone>(&self) -> broadcast::Receiver<E> {
        self.sender::<E>().subscribe()
    }
    pub fn publish<E: 'static + Send + Sync + Clone>(&self, event: E) -> usize {
        self.sender::<E>().send(event).unwrap_or(0)
    }
}

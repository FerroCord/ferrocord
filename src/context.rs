use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    sync::{Arc, RwLock},
};

/// A thread-safe registry containing at most one service of each concrete type.
#[derive(Default)]
pub struct Context {
    services: RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

/// A service of the requested type is already registered.
#[derive(Debug)]
pub struct DuplicateService;

impl fmt::Display for DuplicateService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("service already registered")
    }
}
impl std::error::Error for DuplicateService {}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: Any + Send + Sync>(&self, service: Arc<T>) -> Option<Arc<T>> {
        let old = self
            .services
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(TypeId::of::<T>(), service);
        old.map(|value| {
            value
                .downcast::<T>()
                .unwrap_or_else(|_| panic!("service type invariant violated"))
        })
    }

    pub fn try_insert<T: Any + Send + Sync>(
        &self,
        service: Arc<T>,
    ) -> Result<(), DuplicateService> {
        use std::collections::hash_map::Entry;
        let mut services = self.services.write().unwrap_or_else(|e| e.into_inner());
        match services.entry(TypeId::of::<T>()) {
            Entry::Vacant(entry) => {
                entry.insert(service);
                Ok(())
            }
            Entry::Occupied(_) => {
                drop(services);
                drop(service);
                Err(DuplicateService)
            }
        }
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let value = self
            .services
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&TypeId::of::<T>())
            .cloned();
        value.map(|value| {
            value
                .downcast::<T>()
                .unwrap_or_else(|_| panic!("service type invariant violated"))
        })
    }
}

//! Generic services, typed events, and plugin lifecycle management.

pub mod context;
pub mod event;
pub mod plugin;

pub use context::{Context, DuplicateService};
pub use event::{EventBus, InvalidCapacity};
pub use plugin::{LifecycleError, Plugin, PluginError, PluginManager};

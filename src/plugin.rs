use crate::Context;
use std::{error::Error, fmt};

pub type PluginError = Box<dyn Error + Send + Sync + 'static>;

#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn init(&self, context: &Context) -> Result<(), PluginError>;
    async fn shutdown(&self, context: &Context) -> Result<(), PluginError>;
}

#[derive(Debug)]
pub enum LifecycleError {
    InvalidState,
    DuplicateName(String),
    InitFailed {
        plugin: String,
        source: PluginError,
        rollback_errors: Vec<(String, PluginError)>,
    },
    ShutdownFailed {
        errors: Vec<(String, PluginError)>,
    },
}
impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidState => f.write_str("invalid plugin manager state"),
            Self::DuplicateName(name) => write!(f, "duplicate plugin name: {name}"),
            Self::InitFailed { plugin, source, .. } => {
                write!(f, "plugin {plugin} failed to initialize: {source}")
            }
            Self::ShutdownFailed { errors } => {
                write!(f, "{} plugin(s) failed to shut down", errors.len())
            }
        }
    }
}
impl Error for LifecycleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InitFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    New,
    Starting,
    Running,
    Stopping,
    Terminated,
}

/// Manages plugins in registration order. Use the same context for start and shutdown.
/// The caller must poll lifecycle futures to completion: cancellation leaves the manager
/// in an intermediate state, and asynchronous cleanup is not guaranteed.
#[derive(Default)]
pub struct PluginManager {
    state: State,
    plugins: Vec<(String, Box<dyn Plugin>)>,
}
impl PluginManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add(&mut self, plugin: Box<dyn Plugin>) -> Result<(), LifecycleError> {
        if self.state != State::New {
            return Err(LifecycleError::InvalidState);
        }
        let name = plugin.name().to_owned();
        if self.plugins.iter().any(|(existing, _)| existing == &name) {
            return Err(LifecycleError::DuplicateName(name));
        }
        self.plugins.push((name, plugin));
        Ok(())
    }
    pub async fn start(&mut self, context: &Context) -> Result<(), LifecycleError> {
        if self.state != State::New {
            return Err(LifecycleError::InvalidState);
        }
        self.state = State::Starting;
        for (index, (name, plugin)) in self.plugins.iter().enumerate() {
            if let Err(source) = plugin.init(context).await {
                let mut rollback_errors = Vec::new();
                for (name, plugin) in self.plugins[..index].iter().rev() {
                    if let Err(error) = plugin.shutdown(context).await {
                        rollback_errors.push((name.clone(), error));
                    }
                }
                self.state = State::Terminated;
                return Err(LifecycleError::InitFailed {
                    plugin: name.clone(),
                    source,
                    rollback_errors,
                });
            }
        }
        self.state = State::Running;
        Ok(())
    }
    pub async fn shutdown(&mut self, context: &Context) -> Result<(), LifecycleError> {
        if self.state != State::Running {
            return Err(LifecycleError::InvalidState);
        }
        self.state = State::Stopping;
        let mut errors = Vec::new();
        for (name, plugin) in self.plugins.iter().rev() {
            if let Err(error) = plugin.shutdown(context).await {
                errors.push((name.clone(), error));
            }
        }
        self.state = State::Terminated;
        if errors.is_empty() {
            Ok(())
        } else {
            Err(LifecycleError::ShutdownFailed { errors })
        }
    }
}

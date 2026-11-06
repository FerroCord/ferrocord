use ferrocord::{Context, EventBus, LifecycleError, Plugin, PluginError, PluginManager};
use std::{
    future::Future,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    task::{Context as TaskContext, Poll, Waker},
};
use tokio::sync::broadcast::error::RecvError;

#[test]
fn context_services_and_concurrency() {
    let ctx = Arc::new(Context::new());
    let first = Arc::new(1_u32);
    let text = Arc::new(String::from("service"));
    assert!(ctx.insert(first.clone()).is_none());
    ctx.insert(text.clone());
    assert!(Arc::ptr_eq(&ctx.get::<u32>().unwrap(), &first));
    assert!(Arc::ptr_eq(&ctx.get::<String>().unwrap(), &text));
    assert!(ctx.get::<u8>().is_none());
    assert!(ctx.try_insert(Arc::new(2_u32)).is_err());
    assert!(Arc::ptr_eq(&ctx.get::<u32>().unwrap(), &first));
    let replacement = Arc::new(3_u32);
    assert!(Arc::ptr_eq(
        &ctx.insert(replacement.clone()).unwrap(),
        &first
    ));
    assert!(Arc::ptr_eq(&ctx.get::<u32>().unwrap(), &replacement));
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                for i in 0..1000 {
                    ctx.insert(Arc::new(i));
                    assert!(ctx.get::<i32>().is_some());
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}

#[tokio::test]
async fn events_are_isolated_and_report_lag_and_close() {
    let bus = EventBus::new();
    assert_eq!(bus.publish(1_u32), 0);
    let mut a = bus.subscribe::<u32>();
    let mut b = bus.subscribe::<u32>();
    let mut c = bus.subscribe::<String>();
    assert_eq!(bus.publish(42_u32), 2);
    assert_eq!(bus.publish("hi".to_owned()), 1);
    assert_eq!(a.recv().await.unwrap(), 42);
    assert_eq!(b.recv().await.unwrap(), 42);
    assert_eq!(c.recv().await.unwrap(), "hi");
    drop(bus);
    assert_eq!(a.recv().await, Err(RecvError::Closed));
    assert!(EventBus::with_capacity(NonZeroUsize::new(usize::MAX).unwrap()).is_err());
    for (capacity, count) in [(2, 3), (3, 5)] {
        let bus = EventBus::with_capacity(NonZeroUsize::new(capacity).unwrap()).unwrap();
        let mut rx = bus.subscribe::<u32>();
        for i in 0..count {
            bus.publish(i);
        }
        assert_eq!(rx.recv().await, Err(RecvError::Lagged(1)));
        for i in 1..count {
            assert_eq!(rx.recv().await.unwrap(), i);
        }
    }
}

struct TestPlugin {
    name: &'static str,
    log: Arc<Mutex<Vec<String>>>,
    init_fail: bool,
    shutdown_fail: bool,
    pending_init: bool,
    pending_shutdown: bool,
}
impl TestPlugin {
    fn new(name: &'static str, log: &Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            name,
            log: log.clone(),
            init_fail: false,
            shutdown_fail: false,
            pending_init: false,
            pending_shutdown: false,
        }
    }
}
#[async_trait::async_trait]
impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        self.name
    }
    async fn init(&self, _: &Context) -> Result<(), PluginError> {
        self.log.lock().unwrap().push(format!("init {}", self.name));
        if self.pending_init {
            std::future::pending::<()>().await;
        }
        if self.init_fail {
            return Err("init error".into());
        }
        Ok(())
    }
    async fn shutdown(&self, _: &Context) -> Result<(), PluginError> {
        self.log
            .lock()
            .unwrap()
            .push(format!("shutdown {}", self.name));
        if self.pending_shutdown {
            std::future::pending::<()>().await;
        }
        if self.shutdown_fail {
            return Err("shutdown error".into());
        }
        Ok(())
    }
}
#[tokio::test]
async fn lifecycle_order_and_states() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let ctx = Context::new();
    let mut mgr = PluginManager::new();
    for name in ["a", "b", "c"] {
        mgr.add(Box::new(TestPlugin::new(name, &log))).unwrap();
    }
    assert!(
        matches!(mgr.add(Box::new(TestPlugin::new("a", &log))), Err(LifecycleError::DuplicateName(name)) if name == "a")
    );
    mgr.start(&ctx).await.unwrap();
    assert!(matches!(
        mgr.start(&ctx).await,
        Err(LifecycleError::InvalidState)
    ));
    assert!(matches!(
        mgr.add(Box::new(TestPlugin::new("d", &log))),
        Err(LifecycleError::InvalidState)
    ));
    mgr.shutdown(&ctx).await.unwrap();
    assert!(matches!(
        mgr.shutdown(&ctx).await,
        Err(LifecycleError::InvalidState)
    ));
    assert_eq!(
        *log.lock().unwrap(),
        [
            "init a",
            "init b",
            "init c",
            "shutdown c",
            "shutdown b",
            "shutdown a"
        ]
    );
}
#[tokio::test]
async fn lifecycle_collects_errors() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let ctx = Context::new();
    let mut mgr = PluginManager::new();
    let mut a = TestPlugin::new("a", &log);
    a.shutdown_fail = true;
    let mut b = TestPlugin::new("b", &log);
    b.init_fail = true;
    mgr.add(Box::new(a)).unwrap();
    mgr.add(Box::new(b)).unwrap();
    mgr.add(Box::new(TestPlugin::new("c", &log))).unwrap();
    match mgr.start(&ctx).await.unwrap_err() {
        LifecycleError::InitFailed {
            plugin,
            source,
            rollback_errors,
        } => {
            assert_eq!(plugin, "b");
            assert_eq!(source.to_string(), "init error");
            assert_eq!(rollback_errors.len(), 1);
            assert_eq!(rollback_errors[0].0, "a");
            assert_eq!(rollback_errors[0].1.to_string(), "shutdown error");
        }
        e => panic!("{e}"),
    }
    assert_eq!(*log.lock().unwrap(), ["init a", "init b", "shutdown a"]);
    assert!(matches!(
        mgr.start(&ctx).await,
        Err(LifecycleError::InvalidState)
    ));
    let mut mgr = PluginManager::new();
    for name in ["a", "b", "c"] {
        let mut p = TestPlugin::new(name, &log);
        p.shutdown_fail = name != "b";
        mgr.add(Box::new(p)).unwrap();
    }
    mgr.start(&ctx).await.unwrap();
    match mgr.shutdown(&ctx).await.unwrap_err() {
        LifecycleError::ShutdownFailed { errors } => assert_eq!(
            errors.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            ["c", "a"]
        ),
        e => panic!("{e}"),
    }
    assert!(log.lock().unwrap().ends_with(&[
        "shutdown c".into(),
        "shutdown b".into(),
        "shutdown a".into()
    ]));
}
#[tokio::test]
async fn cancellation_leaves_manager_unusable() {
    for during_start in [true, false] {
        let log = Arc::new(Mutex::new(Vec::new()));
        let ctx = Context::new();
        let mut mgr = PluginManager::new();
        let mut p = TestPlugin::new("a", &log);
        p.pending_init = during_start;
        p.pending_shutdown = !during_start;
        mgr.add(Box::new(p)).unwrap();
        if !during_start {
            mgr.start(&ctx).await.unwrap();
        }
        let mut future: std::pin::Pin<Box<dyn Future<Output = Result<(), LifecycleError>> + '_>> =
            if during_start {
                Box::pin(mgr.start(&ctx))
            } else {
                Box::pin(mgr.shutdown(&ctx))
            };
        let mut cx = TaskContext::from_waker(Waker::noop());
        assert!(matches!(future.as_mut().poll(&mut cx), Poll::Pending));
        drop(future);
        assert!(matches!(
            mgr.add(Box::new(TestPlugin::new("b", &log))),
            Err(LifecycleError::InvalidState)
        ));
        assert!(matches!(
            mgr.start(&ctx).await,
            Err(LifecycleError::InvalidState)
        ));
        assert!(matches!(
            mgr.shutdown(&ctx).await,
            Err(LifecycleError::InvalidState)
        ));
    }
}

mod example;

use async_trait::async_trait;
use dashmap::DashSet;
use std::default::Default;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{atomic, Arc, PoisonError};
use tokio::join;
use tokio::sync::mpsc::{channel, error::SendError, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Action<E> {
    pub event: Option<E>,
    pub options: ActionOptions,
    pub next: Option<(E, ActionOptions)>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Default)]
pub struct ActionOptions {
    pub is_parallel: bool,
    pub is_sequential: bool,
}

impl ActionOptions {
    pub fn parallel() -> Self {
        return Self { is_parallel: true, is_sequential: false };
    }
}

impl <E> Action<E> {
    pub fn new(t: E) -> Action<E> {
        return Action { event: Some(t), options: ActionOptions::default(), next: None };
    }

    pub fn with_options(t: E, options: ActionOptions) -> Action<E> {
        return Action { event: Some(t), options: options, next: None };
    }

    pub fn sequential(t: E) -> Action<E> {
        return Action { event: Some(t), options: ActionOptions { is_parallel: false, is_sequential: true }, next: None };
    }

    pub fn parallel(t: E) -> Action<E> {
        return Action { event: Some(t), options: ActionOptions { is_parallel: true, is_sequential: false }, next: None };
    }

    pub fn next(t: E, next: E, options: ActionOptions) -> Action<E> {
        return Action { event: Some(t), options: ActionOptions::default(), next: Some((next, options)) };
    }

    pub fn dp() -> Action<E> {
        return Action { event: None, options: ActionOptions::default(), next: None }
    }
}

impl <E> Action<E> {
    fn is_dp(&self) -> bool {
        return self.event.is_none();
    }
}


#[derive(Debug)]
pub enum ActionQueueError {
    SendError(String),
    LockPoisoned(String),
    Shutdown,
}

impl<T> From<SendError<Action<T>>> for ActionQueueError {
    fn from(err: SendError<Action<T>>) -> Self {
        ActionQueueError::SendError(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for ActionQueueError {
    fn from(err: PoisonError<T>) -> Self {
        ActionQueueError::LockPoisoned(err.to_string())
    }
}

pub struct ActionQueue<K, E> {
    main_tx: Arc<Sender<Action<E>>>,
    main_rx: Arc<Mutex<Receiver<Action<E>>>>,

    state_tx: Arc<Sender<Action<E>>>,
    state_rx: Arc<Mutex<Receiver<Action<E>>>>,

    is_shutdown: Arc<AtomicBool>,
    is_main_finished: Arc<AtomicBool>,
    is_state_finished: Arc<AtomicBool>,
    parallel_count: Arc<AtomicI32>,

    keys: Arc<DashSet<(u8, K)>>,
}

impl<K, E> ActionQueue<K, E>
where K: Hash + Display + Eq + Clone + Send + Sync + 'static,
      E: UniqueEvent<K> + Clone + Send + Display + 'static {

    pub fn new(buffer: usize) -> Self {
        let (tx, rx) = channel(buffer);
        let (u_tx, u_rx) = channel(buffer);

        ActionQueue {
            main_tx: Arc::new(tx),
            main_rx: Arc::new(Mutex::new(rx)),

            state_tx: Arc::new(u_tx),
            state_rx: Arc::new(Mutex::new(u_rx)),

            keys: Arc::new(DashSet::<(u8, K)>::new()),

            is_shutdown: Arc::new(AtomicBool::new(false)),
            is_main_finished: Arc::new(AtomicBool::new(false)),
            is_state_finished: Arc::new(AtomicBool::new(false)),

            parallel_count: Arc::new(AtomicI32::new(0)),
        }
    }

    fn unique_task(payload: &E) -> Option<(u8, K)> {
        let key = payload.unique_key();
        let Some(key) = key else {
            return None;
        };

        return Some((payload.event_id(), key));
    }

    pub async fn push(&self, action: E) {
        let _ = self.push_internal(Action::new(action))
            .await;
    }

    pub async fn push_sequential(&self, action: E) {
        let _ = self.push_internal(Action::sequential(action))
            .await;
    }

    pub async fn push_parallel(&self, action: E) {
        let _ = self.push_internal(Action::parallel(action))
            .await;
    }

    pub async fn push_with_options(&self, action: E, next_options: ActionOptions) {
        let _ = self.push_internal(Action::with_options(action, next_options))
            .await;
    }

    pub async fn push_action(&self, action: Action<E>) {
        let _ = self.push_internal(action)
            .await;
    }

    async fn push_internal(&self, action: Action<E>) -> Result<(), ActionQueueError> {
        if self.is_shutdown() {
            warn!("[QUEUE] Can't push action, shutting down");
            return Ok(());
        }

        let Some(event) = action.event.clone() else {
            self.main_tx.send(action)
                .await?;

            return Ok(());
        };

        let Some(key) = Self::unique_task(&event) else {
            self.main_tx.send(action)
                .await?;

            return Ok(());
        };

        let was_absent = self.keys.insert(key.clone());
        if !was_absent {
            // let main_rx_size = self.main_rx.lock().await.len();
            info!("[QUEUE] Task is already in the queue, main rx size!");
            return Ok(());
        } else {
            info!("[QUEUE] Lock unique task by id for - {}!", event);
        }

        if let Err(e) = self.main_tx.send(action).await {
            self.keys.remove(&key);
            return Err(ActionQueueError::SendError(e.to_string()));
        }

        return Ok(());
    }

    async fn handle_event<T : EventHandler<K, E>>(&self, action: Action<E>, ctx: Arc<Context<K, E>>, handler: Arc<T>) {
        let Some(event) = action.event else {
            error!("[QUEUE] ActionQueue missed dead action!");
            return;
        };

        let unique = Self::unique_task(&event);

        info!("[QUEUE] Start handling event {}!", event);
        let _ = handler.handle(event, ctx.clone())
            .await;

        if let Some(key) = unique {
            info!("[QUEUE] Remove task looker for {}!", key.1);
            self.keys.remove(&key);
        }

        if let Some((task, options)) = action.next {
            ctx.queue.push_with_options(task, options)
                .await;
        }
    }

    ///////////
    // SHUTDOWN
    ///////////

    async fn push_dps(&self) -> bool {
        return self.main_tx.send(Action::dp()).await.is_ok()
            && self.state_tx.send(Action::dp()).await.is_ok();
    }

    pub async fn async_shutdown(&self) {
        self.is_shutdown.store(true, atomic::Ordering::Release);
        self.push_dps().await;
    }

    pub fn is_shutdown_finished(&self) -> bool {
        let is_shutdown = self.is_shutdown();
        let is_main_finished = self.is_main_finished.load(atomic::Ordering::Acquire);
        let is_state_finished = self.is_state_finished.load(atomic::Ordering::Acquire);
        let has_parallel = self.has_parallel();

        debug!(
            "[QUEUE] is_shutdown_finished: {} | is_main_finished: {} | is_state_finished: {} | has_parallel: {}",
            is_shutdown, is_main_finished, is_state_finished, has_parallel
        );

        return is_shutdown && is_main_finished && is_state_finished && !has_parallel;
    }

    pub fn is_shutdown(&self) -> bool {
        return self.is_shutdown.load(atomic::Ordering::Acquire);
    }

    pub fn has_parallel(&self) -> bool {
        return self.parallel_count.load(atomic::Ordering::Acquire) > 0
    }

    fn finish_state(&self) {
        self.is_state_finished.store(true, atomic::Ordering::Release);
    }

    fn finish_main(&self) {
        self.is_main_finished.store(true, atomic::Ordering::Release);
    }

    fn inc_parallel(&self) {
        self.parallel_count.fetch_add(1, atomic::Ordering::AcqRel);
    }

    fn dec_parallel(&self) {
        self.parallel_count.fetch_sub(1, atomic::Ordering::AcqRel);
    }

    ///////////
    // RUN
    ///////////

    async fn pop(rx: Arc<Mutex<Receiver<Action<E>>>>) -> Option<Action<E>> {
        let mut rx_guard = rx.lock()
            .await;

        match rx_guard.recv().await {
            Some(action) => {
                Some(action)
            }

            None => None,
        }
    }

    pub async fn run<T>(self: Arc<Self>, handler: Arc<T>)
    where
        T: EventHandler<K, E> + Send + Sync + 'static,
    {
        let arc_s = self.clone();
        let handler_s = handler.clone();
        let state_task = tokio::spawn(async move {
            let ctx = Arc::new(Context::new(arc_s.clone()));
            loop {
                if arc_s.is_shutdown() {
                    arc_s.finish_state();
                    warn!("[QUEUE] Shutdown state queue received, shutting down...");
                    break;
                }

                if let Some(task) = Self::pop(arc_s.state_rx.clone()).await {
                    if task.is_dp() {
                        arc_s.finish_state();
                        warn!("[QUEUE] Unique queue received dead pill, shutting down...");
                        break;
                    }

                    let context_c = ctx.clone();
                    let handler_c = handler_s.clone();
                    let arc_c = arc_s.clone();

                    arc_c.handle_event(task, context_c, handler_c)
                        .await;
                }
            }
        });

        let arc_m = self.clone();
        let handler_m = handler.clone();
        let main_task = tokio::spawn(async move {
            let ctx = Arc::new(Context::new(arc_m.clone()));
            loop {
                debug!("[QUEUE] Waiting for event...");
                if arc_m.is_shutdown() {
                    arc_m.finish_main();
                    warn!("[QUEUE] Shutdown main queue received, shutting down...");
                    break;
                }

                if let Some(task) = Self::pop(arc_m.main_rx.clone()).await {
                    if task.is_dp() {
                        arc_m.finish_main();
                        warn!("[QUEUE] Main queue received dead pill, shutting down...");
                        break;
                    }

                    let context_c = ctx.clone();
                    let handler_c = handler_m.clone();
                    let arc_c = arc_m.clone();

                    if task.options.is_parallel {
                        arc_m.inc_parallel(); // TODO v2 decrement when failure??
                        tokio::spawn(async move {
                            if arc_c.is_shutdown() {
                                warn!("[QUEUE] Shutdown parallel task, shutting down...");
                            } else {
                                arc_c.handle_event(task, context_c, handler_c)
                                    .await;
                            }

                            arc_c.dec_parallel();
                        });
                    } else if task.options.is_sequential {
                        arc_c.handle_event(task, context_c, handler_c)
                            .await;
                    } else {
                        let _ = arc_m.state_tx.send(task)
                            .await;
                    }
                }
            }
        });

        let _ = join!(state_task, main_task);
    }
}

pub trait UniqueEvent<T> where T: Hash + Eq + Clone + Send {
    fn event_id(&self) -> u8;
    fn unique_key(&self) -> Option<T>;
}

pub struct Context<K, E> {
    pub queue: Arc<ActionQueue<K, E>>,
}

impl<K, E> Context<K, E> {
    pub fn new(queue: Arc<ActionQueue<K, E>>) -> Self {
        Self { queue }
    }
}

#[async_trait]
pub trait EventHandler<K, E>: Send + Sync {
    async fn handle(&self, event: E, ctx: Arc<Context<K, E>>) -> Result<(), ActionQueueError>;
}


#[cfg(test)]
mod tests {
    use crate::{ActionQueue, ActionQueueError, Context, EventHandler, UniqueEvent};
    use async_trait::async_trait;
    use derive_more::Display;
    use std::sync::Arc;
    use std::time::Duration;
    use tracing::info;

    #[derive(Debug, Display, Clone, Hash, Eq, PartialEq)]
    enum TestEvents {
        Launch,
        Plain,
        Unique,
        Sharded(u64),
    }

    impl UniqueEvent<u64> for TestEvents {
        fn event_id(&self) -> u8 {
            match self {
                TestEvents::Launch => 0,
                TestEvents::Plain => 1,
                TestEvents::Unique => 2,
                TestEvents::Sharded(..) => 3,
            }
        }

        fn unique_key(&self) -> Option<u64> {
            match self {
                TestEvents::Plain => None,
                TestEvents::Launch => Some(0),
                TestEvents::Unique => Some(0),
                TestEvents::Sharded(num) => Some(num.clone())
            }
        }
    }

    struct TestActionHandler;

    #[async_trait]
    impl EventHandler<u64, TestEvents> for TestActionHandler {
        async fn handle(&self, event: TestEvents, ctx: Arc<Context<u64, TestEvents>>) -> Result<(), ActionQueueError> {
            match event {
                TestEvents::Launch => {
                    info!("Execute Launch task");
                    ctx.queue.push(TestEvents::Plain).await;

                    ctx.queue.push(TestEvents::Unique).await;
                    ctx.queue.push(TestEvents::Unique).await;

                    ctx.queue.push_parallel(TestEvents::Sharded(0)).await;
                    ctx.queue.push_parallel(TestEvents::Sharded(0)).await;
                    ctx.queue.push_parallel(TestEvents::Sharded(1)).await;
                }
                TestEvents::Plain => {
                    info!("Execute Plain task");
                }
                TestEvents::Unique => {
                    info!("Execute Unique task");
                    tokio::time::sleep(Duration::from_secs(4))
                        .await;
                    info!("Finish execution Unique task");
                }
                TestEvents::Sharded(num) => {
                    info!("Execute Sharded task: {:?}", num);
                    for _ in 0..4 {
                        info!("Await for Sharded task: {:?}", num);
                        ctx.queue.push_parallel(TestEvents::Sharded(num)).await;
                        tokio::time::sleep(Duration::from_secs(4))
                            .await;
                    }
                }
            }

            return Ok(());
        }
    }

    #[tokio::test]
    async fn check_events() {
        let queue = Arc::new(ActionQueue::new(100));
        let handler = Arc::new(TestActionHandler { });

        queue.push(TestEvents::Launch)
            .await;

        queue.run(handler.clone())
            .await;
    }
}
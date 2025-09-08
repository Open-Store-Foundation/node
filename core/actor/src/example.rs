#[cfg(test)]
mod actor_example {
    use std::sync::Arc;
    use std::time::Duration;
    use async_trait::async_trait;
    use crate::{ActionQueue, ActionQueueError, Context, EventHandler, UniqueEvent};
    use derive_more::Display;
    use tokio::signal;
    use tokio::time::sleep;
    use tracing::{info, warn};
    use core_log::init_tracer;

    #[derive(Debug, Display, Clone, Hash, Eq, PartialEq)]
    pub enum ExampleAction {
        #[display("Launch")]
        Launch,

        #[display("Seq")]
        Seq,

        #[display("Parallel")]
        Common { id: u64 },

        #[display("Parallel")]
        Parallel,

        #[display("Shutdown")]
        Shutdown,
    }


    impl UniqueEvent<u64> for ExampleAction {

        fn event_id(&self) -> u8 {
            match self {
                ExampleAction::Launch => 0,
                ExampleAction::Seq => 1,
                ExampleAction::Parallel => 2,
                ExampleAction::Shutdown => 3,
                ExampleAction::Common { .. } => 4,
            }
        }

        fn unique_key(&self) -> Option<u64> {
            match self {
                ExampleAction::Launch => Some(0),
                ExampleAction::Seq => Some(0),
                ExampleAction::Common { .. } => None,
                ExampleAction::Parallel => Some(0),
                ExampleAction::Shutdown => Some(0),
            }
        }
    }

    pub type ExampleQueue = ActionQueue<u64, ExampleAction>;
    pub type ExampleContext = Context<u64, ExampleAction>;

    #[derive(Clone)]
    pub struct DaemonEventHandler {}


    #[async_trait]
    impl EventHandler<u64, ExampleAction> for DaemonEventHandler {
        async fn handle(&self, event: ExampleAction, ctx: Arc<ExampleContext>) -> Result<(), ActionQueueError> {

            match event {
                ExampleAction::Launch => {
                    info!("[LAUNCH] Start event");
                    ctx.queue.push_sequential(
                        ExampleAction::Seq
                    ).await;

                    ctx.queue.push_parallel(
                        ExampleAction::Parallel
                    ).await;
                    info!("[LAUNCH] Finish event");
                }
                ExampleAction::Seq => {
                    info!("[SEQ] Start event");
                    sleep(Duration::from_secs(20)).await;
                    info!("[SEQ] Finish event");
                }
                ExampleAction::Parallel => {
                    let mut inc = 0;

                    loop {
                        if ctx.queue.is_shutdown() {
                            info!("[PARALLEL] Shutting down");
                            return Ok(());
                        }

                        info!("[PARALLEL] Start iteration {inc}");
                        sleep(Duration::from_secs(10)).await;
                        info!("[PARALLEL] Finish iteration {inc}");

                        ctx.queue.push(
                            ExampleAction::Common { id: inc }
                        ).await;

                        inc += 1;
                    }
                }
                ExampleAction::Common { id } => {
                    info!("[COMMON] Start event [{}]", id);
                    sleep(Duration::from_secs(5)).await;
                    info!("[COMMON] Finish event [{}]", id);

                    if id % 2 == 0 {
                        ctx.queue.push_sequential(
                            ExampleAction::Seq
                        ).await;
                    }
                }
                ExampleAction::Shutdown => {
                    info!("[SHUTDOWN] Execute");
                    ctx.queue.async_shutdown()
                        .await;
                    info!("[SHUTDOWN] Finished");
                }
            }

            return Ok(())
        }
    }


    #[tokio::test]
    async fn start_event_handler() {
        let _guard = init_tracer();

        let handler = Arc::new(DaemonEventHandler {});
        let queue = Arc::new(ExampleQueue::new(10_000));

        let e_queue = queue.clone();
        tokio::spawn(async move {
            tokio::select! {
            () = signal::ctrl_c() => {
                warn!("Shutdown validator queue");
                e_queue.push_sequential(ExampleAction::Shutdown)
                    .await;
            },
        }
        });

        let queue_c = queue.clone();
        let handler_c = handler.clone();
        let task = tokio::spawn(async move {
            queue_c.push(ExampleAction::Launch)
                .await;

            let _ = queue_c.run(handler_c)
                .await;
        });

        let _ = tokio::join!(task);

        while !queue.is_shutdown_finished() {
            warn!("Waiting for parallel tasks shutdown signal");
            sleep(Duration::from_secs(5)).await;
        }
    }
}

use crate::daemon::handler::chain_sync::ChainSyncHandler;
use async_trait::async_trait;
use core_actor::{ActionQueue, ActionQueueError, Context, EventHandler, UniqueEvent};
use derive_more::Display;
use std::sync::Arc;

#[derive(Debug, Display, Clone, Hash, Eq, PartialEq)]
pub enum DaemonAction {
    #[display("Launch")]
    Launch,

    #[display("ChainSync")]
    ChainSync,

    #[display("DownloadsRecount")]
    DownloadsRecount,

    #[display("Shutdown")]
    Shutdown,
}

impl UniqueEvent<u64> for DaemonAction {

    fn event_id(&self) -> u8 {
        match self {
            DaemonAction::Launch => 0,
            DaemonAction::ChainSync => 1,
            DaemonAction::DownloadsRecount => 2,
            DaemonAction::Shutdown => 3,
        }
    }

    fn unique_key(&self) -> Option<(u64)> {
        match self {
            DaemonAction::Launch => Some(0),
            DaemonAction::ChainSync => Some(0),
            DaemonAction::DownloadsRecount => Some(0),
            DaemonAction::Shutdown => Some(0),
        }
    }
}

pub type DaemonQueue = ActionQueue<u64, DaemonAction>;
pub type DaemonContex = Context<u64, DaemonAction>;

#[derive(Clone)]
pub struct DaemonEventHandler {
    chain: Arc<ChainSyncHandler>,
}

impl DaemonEventHandler {
    pub fn new(chain: Arc<ChainSyncHandler>) -> Self {
        Self { chain }
    }
}

#[async_trait]
impl EventHandler<u64, DaemonAction> for DaemonEventHandler {
    async fn handle(&self, event: DaemonAction, ctx: Arc<DaemonContex>) -> Result<(), ActionQueueError> {
        match event {
            DaemonAction::Launch => {
                ctx.queue.push_parallel(DaemonAction::ChainSync)
                    .await;

                ctx.queue.push_parallel(DaemonAction::DownloadsRecount)
                    .await;
            }
            DaemonAction::ChainSync => {
                self.chain.handle(ctx.clone())
                    .await;
            }
            DaemonAction::DownloadsRecount => {
                
            }
            DaemonAction::Shutdown => {
                ctx.queue.async_shutdown()
                    .await;
            }
        }

        return Ok(())
    }
}

use crate::data::validation_repo::ValidationRepo;
use crate::handlers::try_assign::TryAssignHandler;
use crate::handlers::finalize::FinalizeHandler;
use crate::handlers::observe_voting::ObserveVotingHandler;
use crate::handlers::poll::{EvmEventPool, PollHandler};
use crate::handlers::propose::ProposeHandler;
use crate::handlers::register::RegisterHandler;
use crate::handlers::sync::SyncHandler;
use crate::handlers::unregister::UnregisterHandler;
use crate::handlers::validate_sync::ValidateSyncHandler;
use crate::handlers::vote::VoteHandler;
use alloy::primitives::private::derive_more::Display;
use alloy::primitives::Address;
use core_actor::{Action, ActionQueue, ActionQueueError, Context, EventHandler, UniqueEvent};
use service_sc::store::ScStoreService;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tokio::time::sleep;
use tracing::info;
use crate::handlers::check_proposal::CheckProposalHandler;
use crate::handlers::observe_overdue::ObserveOverdueHandler;

pub type ValidationQueue = ActionQueue<u64, ValidatorEvent>;
pub type ValidationAction = Action<ValidatorEvent>;
pub type ValidationContext = Context<u64, ValidatorEvent>;

#[derive(Debug, Display, Clone, Hash, Eq, PartialEq)]
pub enum ValidatorEvent {
    #[display("Launch")]
    Launch,
    #[display("Register")]
    Register,
    #[display("Sync")]
    Sync,

    #[display("Poll({block_number})")]
    Poll { block_number: u64 },
    #[display("ObserveOverdue")]
    ObserveOverdue,
    #[display("Enqueue")]
    TryAssign,

    #[display("ValidateSync")]
    ValidateSync,

    #[display("CheckProposal")]
    CheckProposal { block_id: Option<u64> },

    #[display("Propose({block_id},{from})")]
    Propose { block_id: u64, from: u64 },

    #[display("Vote({block_id})")]
    Vote { block_id: u64 },
    #[display("ObserveVoting({block_id})")]
    ObserveVoting { block_id: u64 },

    #[display("Finalize({block_id})")]
    Finalize { block_id: u64 },

    #[display("Unregister")]
    Unregister,
}

impl ValidatorEvent {
    pub fn vote(block_id: u64) -> Self {
        Self::Vote { block_id }
    }
    pub fn observe(block_id: u64) -> Self {
        Self::ObserveVoting { block_id }
    }
    pub fn check_proposal(block_id: Option<u64>) -> Self {
        Self::CheckProposal { block_id }
    }
    pub fn propose(block_id: u64, from: u64) -> Self {
        Self::Propose { block_id, from }
    }
    pub fn finalize(block_id: u64) -> Self {
        Self::Finalize { block_id }
    }
    pub fn poll(block_number: u64) -> Self {
        Self::Poll { block_number }
    }
}

impl UniqueEvent<u64> for ValidatorEvent {

    fn event_id(&self) -> u8 {
        match self {
            ValidatorEvent::Launch => 0,
            ValidatorEvent::Register => 5,
            ValidatorEvent::Sync => 10,
            ValidatorEvent::Poll { .. } => 20,
            ValidatorEvent::ObserveOverdue => 30,
            ValidatorEvent::TryAssign => 40,
            ValidatorEvent::ValidateSync { .. } => 50,
            ValidatorEvent::Vote { .. } => 60,
            ValidatorEvent::CheckProposal { .. } => 65,
            ValidatorEvent::Propose { .. } => 70,
            ValidatorEvent::ObserveVoting { .. } => 80,
            ValidatorEvent::Finalize { .. } => 90,
            ValidatorEvent::Unregister => 100,
        }
    }

    fn unique_key(&self) -> Option<(u64)> {
        match self {
            ValidatorEvent::Launch => Some(0),
            ValidatorEvent::ValidateSync { .. } => Some(0),
            ValidatorEvent::Register => Some(0),
            ValidatorEvent::Sync => Some(0),
            ValidatorEvent::Poll { .. } => Some(0),
            ValidatorEvent::ObserveOverdue => Some(0),
            
            ValidatorEvent::TryAssign => Some(0),

            ValidatorEvent::CheckProposal { block_id} => Some(block_id.unwrap_or(0)),
            ValidatorEvent::Vote { block_id} => Some(block_id.clone()),
            ValidatorEvent::Propose { block_id, .. } => Some(block_id.clone()),
            ValidatorEvent::ObserveVoting { block_id } => Some(block_id.clone()),
            ValidatorEvent::Finalize { block_id } => Some(block_id.clone()),
            ValidatorEvent::Unregister => Some(0),
        }
    }
}

#[derive(Clone)]
pub struct ValidatorQueue {
    register: Arc<RegisterHandler>,
    sync: Arc<SyncHandler>,
    poll: Arc<PollHandler>,

    try_assign: Arc<TryAssignHandler>,
    validate_sync: Arc<ValidateSyncHandler>,
    check_proposal: Arc<CheckProposalHandler>,
    vote: Arc<VoteHandler>,
    propose: Arc<ProposeHandler>,
    observe_voting: Arc<ObserveVotingHandler>,
    observe_overdue: Arc<ObserveOverdueHandler>,
    finalize: Arc<FinalizeHandler>,

    unregister: Arc<UnregisterHandler>,
}

impl ValidatorQueue {

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        register: Arc<RegisterHandler>,
        sync: Arc<SyncHandler>,
        poll: Arc<PollHandler>,

        enqueue: Arc<TryAssignHandler>,
        validate: Arc<ValidateSyncHandler>,
        check_proposal: Arc<CheckProposalHandler>,
        vote: Arc<VoteHandler>,
        propose: Arc<ProposeHandler>,
        observe_voting: Arc<ObserveVotingHandler>,
        observe_overdue: Arc<ObserveOverdueHandler>,
        finalize: Arc<FinalizeHandler>,

        unregister: Arc<UnregisterHandler>,
    ) -> Self {
        Self {
            register,
            sync,
            poll,

            try_assign: enqueue,
            validate_sync: validate,
            check_proposal,
            vote,
            propose,
            observe_voting,
            observe_overdue,
            finalize,

            unregister,
        }
    }
}

#[async_trait]
impl EventHandler<u64, ValidatorEvent> for ValidatorQueue {

    async fn handle(&self, event: ValidatorEvent, ctx: Arc<ValidationContext>) -> Result<(), ActionQueueError> {
        match event {
            ValidatorEvent::Launch => {
                ctx.queue.push_sequential(ValidatorEvent::ValidateSync)
                    .await;
            }
            ValidatorEvent::ValidateSync => {
                self.validate_sync.handle(ctx.clone())
                    .await;
            }
            ValidatorEvent::Register => {
                self.register.handle(ctx.clone())
                    .await;
            }
            ValidatorEvent::Sync => {
                self.sync.handle(ctx.clone())
                    .await;
            }
            ValidatorEvent::TryAssign => {
                self.try_assign.handle(ctx.clone())
                    .await;
            }
            ValidatorEvent::CheckProposal { block_id } => {
                self.check_proposal.handle(block_id, ctx.clone())
                    .await;
            }
            ValidatorEvent::Vote { block_id } => {
                self.vote.handle(block_id, ctx.clone())
                    .await;
            }
            ValidatorEvent::Propose { block_id, from } => {
                self.propose.handle(block_id, from, ctx.clone())
                    .await;
            }
            ValidatorEvent::Finalize { block_id } => {
                self.finalize.handle(block_id, ctx.clone())
                    .await;
            }
            ValidatorEvent::ObserveVoting { block_id } => {
                self.observe_voting.handle(block_id, ctx.clone())
                    .await;
            }
            ValidatorEvent::Poll { block_number } => {
                self.poll.spawn_polling(block_number, ctx.clone())
                    .await;
            }
            ValidatorEvent::ObserveOverdue => {
                self.observe_overdue.handle()
                    .await;
            }
            ValidatorEvent::Unregister => {
                self.unregister.handle(ctx.clone())
                    .await;
            }
        };

        return Ok(())
    }
}

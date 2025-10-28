use crate::env;
use alloy::consensus::Transaction;
use alloy::primitives::{Address, Bytes, Log, TxHash, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionReceipt;
use alloy::sol_types::sol_data::{Int, Uint};
use alloy::sol_types::{SolCall, SolEvent};
use codegen_contracts::contracts::OpenStore;
use codegen_contracts::ext::ToChecksum;
use derive_more::Display;
use net_client::node::provider::Web3Provider;
use net_client::node::result::{EthResult};
use net_client::node::watcher::TxWorkaround;
use std::sync::Arc;
use tracing::info;

#[derive(Debug)]
pub struct StoreRequestInfo {
    pub req_type: u8,
    pub target: Address,
    pub data: Bytes,
}

#[derive(Debug)]
pub struct StoreLastState {
    pub block_number: u64,

    pub next_assign_block: u64,
    pub next_final_block: u64,
    pub next_proposal_block: u64,

    pub my_block: u64,
    pub my_emergency_block: u64,

    pub next_request: u64,
    pub next_proposal_request: u64,
}

impl StoreLastState {
    pub fn can_unassign(&self) -> bool {
        return (self.next_assign_block - 1 == self.my_block || self.my_block == 0)
            && self.my_emergency_block == 0
    }

    pub fn can_assign_validator(&self) -> bool {
        return self.my_block == 0
    }

    pub fn is_next_proposal_my(&self) -> bool {
        return self.next_proposal_block == self.my_block;
    }

    pub fn has_proposal_requests(&self) -> bool {
        return self.next_request > self.next_proposal_request;
    }

    pub fn should_create_proposal(&self) -> bool {
        return self.is_next_proposal_my() && self.has_proposal_requests();
    }

    pub fn is_my_block(&self, block_id: u64) -> bool {
        return (block_id == self.my_emergency_block || block_id == self.my_block )
            && block_id <= self.next_final_block
    }

    pub fn is_my_next_finalization_block(&self, block_id: u64) -> bool {
        return self.is_my_block(block_id) && block_id == self.next_final_block;
    }
}

#[derive(Debug, Display, PartialEq, PartialOrd)]
pub enum ValidatorAssignStatus {
    Assignable = 0,
    ValidatorVersionOutdated = 1,
    NotRegistered = 2,
    NotEnoughVotes = 3,
    AlreadyAssigned = 4,
}

impl ValidatorAssignStatus {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => ValidatorAssignStatus::ValidatorVersionOutdated,
            2 => ValidatorAssignStatus::NotRegistered,
            3 => ValidatorAssignStatus::NotEnoughVotes,
            4 => ValidatorAssignStatus::AlreadyAssigned,
            _ => ValidatorAssignStatus::Assignable,
        }
    }
}

#[derive(Debug, Display, PartialEq, PartialOrd)]
pub enum BlockState {
    Unknown = 0,
    Assigned = 1,
    Proposed = 2,
    Discussing = 3,
    Voted = 4,
    Finalized = 5,
}

impl BlockState {
    pub fn from_u8(value: u8) -> Self {
        match value {
            1 => BlockState::Assigned,
            2 => BlockState::Proposed,
            3 => BlockState::Discussing,
            4 => BlockState::Voted,
            5 => BlockState::Finalized,
            _ => BlockState::Unknown,
        }
    }

    pub fn proposal(is_discussion: bool) -> BlockState {
        if is_discussion { BlockState::Discussing } else { BlockState::Proposed }
    }

    pub fn is_finalized(&self) -> bool {
        return self == &BlockState::Finalized;
    }

    pub fn is_voted(&self) -> bool {
        return self == &BlockState::Voted;
    }

    pub fn is_discussing(&self) -> bool {
        return self == &BlockState::Discussing;
    }

    pub fn is_assigned(&self) -> bool {
        return self == &BlockState::Assigned;
    }

    pub fn is_proposed(&self) -> bool {
        return self == &BlockState::Proposed;
    }

    pub fn at_least_proposed(&self) -> bool {
        return self >= &BlockState::Proposed;
    }

    pub fn at_least_assigned(&self) -> bool {
        return self >= &BlockState::Assigned;
    }

    pub fn at_least_voted(&self) -> bool {
        return self >= &BlockState::Voted;
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StoreBlockRef {
    pub id: u64,

    pub from_request_id: u64,
    pub to_request_id: u64, // to excluding
    pub result: U256, // bitset of results

    pub ref_id: Vec<u8>,
    pub protocol_id: i32,
    pub block_hash: B256,

    pub propoerty_mask: u8,
    pub created_by: Address,
}

pub struct ScStoreService {
    store: Address,
    version: u64,
    provider: Arc<Web3Provider>,
}

pub type AndroidObjRequestData = (Int<64>, Uint<64>, Uint<8>);

pub const BI_MASK_PROPOSAL: u8 = 0b00000000;
pub const BI_MASK_DISCUSSION: u8 = 0b00000001;

impl ScStoreService {
    pub fn new(store: Address, version: u64, client: &Arc<Web3Provider>) -> Self {
        Self { store, version, provider: client.clone() }
    }
}

impl ScStoreService {

    pub const BLOCK_FINALIZED_HASH: B256 = OpenStore::BlockFinalized::SIGNATURE_HASH;
    pub const NEW_REQUEST_HASH: B256 = OpenStore::NewRequest::SIGNATURE_HASH;
    pub const BLOCK_PROPOSED_HASH: B256 = OpenStore::BlockProposed::SIGNATURE_HASH;
    pub const ADDED_TO_TRACK_HASH: B256 = OpenStore::AddedToTrack::SIGNATURE_HASH;

    pub fn validation_topics() -> Vec<B256> {
        vec![
            ScStoreService::NEW_REQUEST_HASH,
            ScStoreService::BLOCK_PROPOSED_HASH,
        ]
    }

    pub fn decode_add_to_track(p0: &Log) -> alloy::sol_types::Result<Log<OpenStore::AddedToTrack>> {
        let result = OpenStore::AddedToTrack::decode_log(p0.as_ref());
        return result;
    }

    pub fn decode_block_finalize(p0: &Log) -> alloy::sol_types::Result<Log<OpenStore::BlockFinalized>> {
        let result = OpenStore::BlockFinalized::decode_log(p0.as_ref());
        return result;
    }

    pub fn decode_new_request(p0: &Log) -> alloy::sol_types::Result<Log<OpenStore::NewRequest>> {
        let result = OpenStore::NewRequest::decode_log(p0.as_ref());
        return result;
    }

    pub fn decode_proposed(p0: &Log) -> alloy::sol_types::Result<Log<OpenStore::BlockProposed>> {
        let result = OpenStore::BlockProposed::decode_log(p0.as_ref());
        return result;
    }

    pub async fn get_state(&self, validator: Address) -> EthResult<StoreLastState> {
        let contract = OpenStore::new(self.store, &self.provider);

        let data = contract.getLastState(validator)
            .call()
            .await?;


        let state = StoreLastState {
            block_number: data._0.to(),

            next_assign_block: data._1.to(),
            next_final_block: data._2.to(),
            my_emergency_block: data._3.to(),
            next_proposal_block: data._4.to(),
            my_block: data._5.to(),

            next_proposal_request: data._6.to(),
            next_request: data._7.to(),
        };

        Ok(state)
    }

    pub async fn get_request(&self, request_id: u64) -> EthResult<StoreRequestInfo> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.getRequest(U256::from(request_id))
            .call()
            .await?;

        Ok(
            StoreRequestInfo {
                req_type: result._0.to(),
                target: result._1,
                data: result._2,
            }
        )
    }

    pub async fn block_state(&self, block_id: u64, validator: Address) -> EthResult<BlockState> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.blockStateFor(U256::from(block_id), validator)
            .call()
            .await?;

        Ok(BlockState::from_u8(result))
    }

    pub async fn is_finalazible(&self, block_id: u64) -> EthResult<bool> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.isFinalazible(U256::from(block_id))
            .call()
            .await?;

        info!("Finalization winner {}, voterCounter {}, maxVotes {}, subMaxVotes {}, rest {}", result._0.checksum(), result._1, result._2, result._3, result._4);

        return Ok(result._0 != Address::ZERO);
    }

    pub async fn get_block_proposers(&self, block_id: u64) -> EthResult<Vec<Address>> {
        let contract = OpenStore::new(self.store, &self.provider);
        let proposers = contract.getBlockProposers(U256::from(block_id))
            .call()
            .await?;

        return Ok(proposers)
    }

    pub async fn get_block_info(&self, block_id: u64, validator: Address) -> EthResult<StoreBlockRef> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.proposalBlockInfo(U256::from(block_id), validator)
            .call()
            .await?;

        Ok(
            StoreBlockRef {
                id: result._0.to(),
                from_request_id: result._1.to(),
                to_request_id: result._2.to(),
                result: result._3,
                block_hash: result._4,
                ref_id: result._5.0.to_vec(),
                protocol_id: result._6 as i32,
                propoerty_mask: result._7,
                created_by: result._8,
            }
        )
    }

    pub async fn next_assign_block_id(&self) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.nextBlockIdToValidated
        ()
            .call()
            .await?;

        return Ok(result.to());
    }

    pub async fn next_block_id_to_finalize(&self) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.nextBlockIdToFinalize()
            .call()
            .await?;

        return Ok(result.to());
    }

    pub async fn next_block_id_for(&self, validator: Address) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.nextBlockIdFor(validator)
            .call()
            .await?;

        return Ok(result.to());
    }

    pub async fn emergency_block_id_for(&self, validator: Address) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.emergencyBlockIdFor(validator)
            .call()
            .await?;

        return Ok(result.to());
    }

    pub async fn least_request_id_to_finalize(&self) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        // let result = contract.leastRequestIdToFinalize() // TODO V2 replace after next contract redeploy
        //     .call()
        //     .await?;

        let result: U256 = contract.nextBlockIdToFinalize()
            .call()
            .await?;

        let block = contract.getBlockRef(result - U256::from(1))
            .call()
            .await?;

        let result = block.toRequestId.to::<u64>();
        if result == 0 {
            return Ok(1);
        }

        return Ok(result);
    }

    pub async fn next_request_id(&self) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.nextRequestIdToValidate()
            .call()
            .await?;
        
        return Ok(result.to());
    }

    pub async fn total_balance(&self, validator: Address) -> EthResult<U256> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.validatorTotalBalance(validator)
            .call()
            .await?;

        return Ok(result);
    }

    pub async fn balance(&self, validator: Address) -> EthResult<U256> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.balance(validator)
            .call()
            .await?;

        return Ok(result);
    }

    pub async fn recommended_stake_amount(&self) -> EthResult<U256> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.getMinStakeAmount()
            .call()
            .await?;

        return Ok(result);
    }

    pub async fn can_assign_validator(&self, validator: Address) -> EthResult<bool> {
        let contract = OpenStore::new(self.store, &self.provider);
        let result = contract.canAssignValidator(validator)
            .call()
            .await?;

        return Ok(result);
    }

    pub async fn finalize(&self, block_id: u64) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.finalizeBlock(U256::from(block_id))
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn top_up(&self, value: U256) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.topUp()
            .value(value)
            .into_transaction_request();
        
        return contract.provider().send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn validator_assign_status(&self, validator: Address) -> EthResult<ValidatorAssignStatus> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.validatorAssignStatus(validator, self.version)
            .call()
            .await?;

        return Ok(ValidatorAssignStatus::from_u8(result))
    }

    pub async fn min_available_version(&self) -> EthResult<u64> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.minValidatorVersion()
            .call()
            .await?;

        return Ok(result)
    }

    pub async fn is_registered(&self, validator: Address) -> EthResult<bool> {
        let contract = OpenStore::new(self.store, &self.provider);

        let result = contract.isValidatorRegistered(validator)
            .call()
            .await?;

        return Ok(result)
    }

    pub async fn register_validator(&self) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.registerValidator(self.version)
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn unregister_validator(&self) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.unregisterValidator()
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn assign_validator(&self, block_id: u64) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.assignBlockId(U256::from(block_id))
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn unassign_validator(&self, block_id: u64) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.unassignBlockId(U256::from(block_id))
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn get_block_data(&self, tx_hash: TxHash) -> EthResult<Option<Vec<u8>>> {
        let data = self.provider.get_transaction_by_hash(tx_hash)
            .await?;

        match data {
            Some(data) => {
                let tx = data.inner.input();
                let call = OpenStore::saveBlockDataCall::abi_decode(tx.0.as_ref());
                match call {
                    Ok(call) => Ok(Some(call.blockData.0.to_vec())),
                    Err(_) => Ok(None)
                }
            }
            None => {
                return Ok(None)
            }
        }
    }

    pub async fn save_block_data(&self, block_data: &Vec<u8>) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.saveBlockData(Bytes::from(block_data.clone()))
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn propose_block(&self, block: &StoreBlockRef) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let sc_block = OpenStore::BlockRef {
            id: U256::from(block.id),
            fromRequestId: U256::from(block.from_request_id),
            toRequestId: U256::from(block.to_request_id),
            result: block.result,
            objectHash: block.block_hash,
            objectId: Bytes::from(block.ref_id.clone()),
            protocolId: block.protocol_id as u16,
            blockMask: block.propoerty_mask,
            createdBy: block.created_by,
        };

        let request = contract.proposeBlock(sc_block)
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn vote(&self, block_id: u64, validator: Address, unavailable_mask: u128) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.vote(
            U256::from(block_id),
            validator,
            unavailable_mask
        )
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }

    pub async fn finalize_block(&self, block_id: u64) -> EthResult<TransactionReceipt> {
        let contract = OpenStore::new(self.store, &self.provider);

        let request = contract.finalizeBlock(U256::from(block_id))
            .into_transaction_request();

        return self.provider.send_and_wait(request, env::poll_timeout_ms())
            .await
    }
}

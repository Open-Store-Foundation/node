use alloy::hex::FromHex;
use crate::utils::hasher::HasherSha256;
use crate::utils::merkle::MerkleTree;
use alloy::primitives::{Address, B256, U256};
use codegen_block::block::{ValidationBlock, ValidationResult};
use prost::Message;
use tracing::error;
use codegen_contracts::ext::write_2bit_status;
use service_sc::store::{StoreBlockRef, BI_MASK_DISCUSSION, BI_MASK_PROPOSAL};
use crate::data::storage::ProtocolId;
use crate::handlers::common::create_proposal::ProposeBlockContext;

pub struct BlockRepo {
    version: u64,
    validator: Address,
}

impl BlockRepo {
    pub fn new(version: u64, validator: Address) -> Self {
        Self { version, validator }
    }
}

impl BlockRepo {

    pub fn create_block(
        &self,
        block_id: u64,
        data: Vec<ValidationResult>
    ) -> ValidationBlock {
        let block = ValidationBlock {
            id: block_id,
            requests: data,
        };

        return block;
    }
    
    pub fn contract_block(
        &self,
        tx_hash: Vec<u8>,
        provider_id: ProtocolId,
        ctx: &ProposeBlockContext,
    ) -> Option<StoreBlockRef> {
        let mut result= U256::ZERO;
        let block = &ctx.block;
        
        for i in 0..block.requests.len() {
            let next = i * 2;
            let request = &block.requests[i];
            write_2bit_status(&mut result, next, request.status);
        }
        
        let Some(from_incl) = block.from_request_id() else {
            error!("Block {} has no from request", block.id);
            return None;
        };
        
        let Some(to_excl) = block.to_request_id() else {
            error!("Block {} has no to request", block.id);
            return None;       
        };
        
        let hash = ctx.block_hash.clone();
        let Ok(hash) = B256::try_from(hash.as_slice()) else {
            error!("Block {} has no hash", block.id);
            return None;       
        };

        let block = StoreBlockRef {
            id: block.id,
            
            block_hash: hash,
            ref_id: tx_hash,
            protocol_id: provider_id as i32,

            from_request_id: from_incl,
            to_request_id: to_excl,
            result,

            propoerty_mask: if ctx.is_discussion { BI_MASK_DISCUSSION } else { BI_MASK_PROPOSAL },
            created_by: self.validator.clone(),
        };

        return Some(block);
    }
}

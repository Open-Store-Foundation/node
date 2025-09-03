use crate::data::block_repo::BlockRepo;
use crate::data::storage::ProtocolId;
use crate::data::validation_repo::ValidationRepo;
use crate::ext::validation_block::proto_sha256;
use alloy::primitives::Address;
use codegen_block::block::ValidationBlock;
use openssl::sha::sha256;
use prost::Message;
use service_sc::store::{BlockState, ScStoreService, StoreBlockRef};
use std::sync::Arc;
use hex::ToHex;
use tracing::{error, info};
use client_tg::{tg_alert, tg_msg};
use core_std::hexer;

pub struct ProposeBlockContext {
    pub(crate) block_hash: Vec<u8>,
    pub(crate) block_data: Vec<u8>,
    pub(crate) block: ValidationBlock,
    pub(crate) is_discussion: bool,
}

impl ProposeBlockContext {
    
    pub fn discussion(block: ValidationBlock) -> Self {
        return ProposeBlockContext::new(block, true);
    }
    
    pub fn proposal(block: ValidationBlock) -> Self {
        return ProposeBlockContext::new(block, false);
    }
    
    fn new(block: ValidationBlock, is_discussion: bool) -> Self {
        let block_data = block.encode_to_vec();
        let block_hash = sha256(block_data.as_slice()).to_vec();
        
        ProposeBlockContext {
            block_hash,
            block_data,
            block,
            is_discussion,
        }
    }
}

pub enum ProposeBlockStage {
    UploadBlockData,
    ProposeBlock(StoreBlockRef),
}

pub struct CreateProposalCase {
    persist: Arc<ValidationRepo>,
    service: Arc<ScStoreService>,
    block_repo: Arc<BlockRepo>,
}

impl CreateProposalCase {

    pub fn new(
        persist: Arc<ValidationRepo>,
        service: Arc<ScStoreService>,
        block_repo: Arc<BlockRepo>
    ) -> Self {
        Self { persist, service, block_repo }
    }
    
    pub async fn poll(
        &self,
        stage: Option<ProposeBlockStage>,
        ctx: &ProposeBlockContext,
    ) -> Option<ProposeBlockStage> {
        let block_id = ctx.block.id;
        let is_discussion = ctx.is_discussion;

        let mut stage = stage.unwrap_or_else(|| ProposeBlockStage::UploadBlockData);

        loop {
            stage = match stage {
                ProposeBlockStage::UploadBlockData => {
                    info!("[PROPOSE_HANDLER] Uploading block {} to storage...", block_id);
                    let receipt = self.service.save_block_data(&ctx.block_data)
                        .await;

                    match receipt {
                        Ok(receipt) => {
                            let tx_id = hexer::encode_upper_pref(receipt.transaction_hash);
                            let ref_id = receipt.transaction_hash.to_vec();
                            info!("[PROPOSE_HANDLER] Storage block {} - {} uploaded successfully.", block_id, tx_id);
                            let store_info = self.block_repo.contract_block(
                                ref_id, ProtocolId::BSC, &ctx
                            );
                            
                            match store_info {
                                Some(store_info) => ProposeBlockStage::ProposeBlock(store_info),
                                None => return Some(ProposeBlockStage::UploadBlockData)        
                            }
                        }
                        Err(e) => {
                            error!("[PROPOSE_HANDLER] Storage block {} upload failed: {}.", block_id, e);
                            tg_alert!(format!("[PROPOSE_HANDLER] Storage block {} upload failed: {}.", block_id, e));
                            return Some(ProposeBlockStage::UploadBlockData)
                        }
                    }
                }
                ProposeBlockStage::ProposeBlock(block) => {
                    info!("[PROPOSE_HANDLER] Proposing block {} to contract...", block_id);
                    let receipt = self.service.propose_block(&block)
                        .await;

                    if let Err(e) = receipt {
                        error!("[PROPOSE_HANDLER] Block {} propose failed: {}. Retrying...", block_id, e);
                        tg_alert!(format!("[PROPOSE_HANDLER] Block {} propose failed: {}. Retrying...", block_id, e));
                        return Some(ProposeBlockStage::ProposeBlock(block));
                    }

                    info!("[PROPOSE_HANDLER] Block {} proposed successfully.", block_id);

                    let state = BlockState::proposal(is_discussion);
                    let _ = self.persist.save_block(block_id, state, &ctx.block_data)
                        .await; // doesn't matter, we can upload it from arweave

                    info!("[PROPOSE_HANDLER] Proposal for block_id {} (discuss {}) created and persisted.", block_id, is_discussion);
                    tg_msg!(format!("[PROPOSE_HANDLER] Proposal for block_id {} (discuss {}) created and persisted.", block_id, is_discussion));
                    return None;
                }
            }
        }
    }
}

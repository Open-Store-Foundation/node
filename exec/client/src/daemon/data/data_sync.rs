use crate::data::models::{AssetlinkSync, NewArtifact, NewAsset, NewBuildRequest, Publishing, ValidationProof};
use crate::data::repo::artifact_repo::ArtifactRepo;
use crate::data::repo::assetlink_repo::AssetlinkRepo;
use crate::data::repo::batch_repo::{BatchRepo, TransactionBatch, TransactionStatus};
use crate::data::repo::error_repo::ErrorRepo;
use crate::data::repo::object_repo::ObjectRepo;
use crate::data::repo::validation_repo::ValidationRepo;
use db_psql::client::PgClient;
use service_graph::client::AppAsset;
use std::sync::Arc;
use tracing::error;
use crate::data::repo::publishing_repo::PublishingRepo;
use crate::result::ClientResult;

pub enum LogResultData {
    NewRequest(
        Option<NewBuildRequest>,
        Option<NewArtifact>,
        Option<NewAsset>,
        Option<Publishing>,
    ),
    FinishSync(
        Option<AssetlinkSync>,
        Option<ValidationProof>,
    ),
}

pub struct DataSyncHandler {
    client: PgClient,
    object_repo: Arc<ObjectRepo>,
    batch_repo: Arc<BatchRepo>,
    assetlink_repo: Arc<AssetlinkRepo>,
    art_repo: Arc<ArtifactRepo>,
    validation_repo: Arc<ValidationRepo>,
    publishing_repo: Arc<PublishingRepo>,
    error_repo: Arc<ErrorRepo>,
}

impl DataSyncHandler {

    pub fn new(
        client: PgClient,
        object_repo: Arc<ObjectRepo>,
        batch_repo: Arc<BatchRepo>,
        assetlink_repo: Arc<AssetlinkRepo>,
        art_repo: Arc<ArtifactRepo>,
        validation_repo: Arc<ValidationRepo>,
        publishing_repo: Arc<PublishingRepo>,
        error_repo: Arc<ErrorRepo>,
    ) -> Self {
        Self {
            client,
            object_repo,
            batch_repo,
            assetlink_repo,
            art_repo,
            validation_repo,
            publishing_repo,
            error_repo,
        }
    }

    pub async fn last_sync_batch(&self) -> ClientResult<Option<TransactionBatch>> {
        return self.batch_repo.get_last_batch().await;
    }

    pub async fn sync(
        &self,
        new_data: &Vec<LogResultData>,
        apps: &Option<Vec<AppAsset>>,
        from_block: u64,
        last_block_number: u64,
    ) {
        if let Ok(transaction) = self.client.start().await {
            for data in new_data.iter() {
                match data {
                    LogResultData::NewRequest(request, artifact, asset, publish) => {
                        if let Some(request) = request {
                            if let Err(e) = self.validation_repo.insert_or_update(&request).await {
                                error!("[NEW_REQ_HANDLER] Can't insert build for {}: {}", request.object_address, e);
                            };
                        }

                        if let Some(artifact) = artifact {
                            if let Err(e) = self.art_repo.insert_artifact(&artifact).await {
                                error!("[NEW_REQ_HANDLER] Can't insert artifact for {} with ref {}: {}", artifact.object_address, artifact.object_ref, e);
                            };
                        }

                        if let Some(obj) = asset {
                            if let Err(e) = self.object_repo.insert_or_update(&obj).await {
                                error!("[NEW_REQ_HANDLER] Can't insert asset with {}: {}", obj.address, e);
                            };
                        }

                        if let Some(publish) = publish {
                            if let Err(e) = self.publishing_repo.insert_or_update(&publish).await {
                                error!("[NEW_REQ_HANDLER] Can't insert publish for {} with track {} and version {}: {}", publish.object_address, publish.track_id, publish.version_code, e);
                            };
                        }
                    }
                    LogResultData::FinishSync(sync, proof) => {
                        if let Some(sync) = sync {
                            let result = self.assetlink_repo.insert_assetlink_status(&sync).await;

                            if let Err(e) = result {
                                error!("[SYNC_FINISH_HANDLER] Failed to insert assetlink status for asset {} with version {}: {}", sync.object_address, sync.owner_version, e);
                            } else {
                                break;
                            }
                        }

                        if let Some(proof) = proof {
                           let _ = self.assetlink_repo.insert_validation_proof(proof).await;
                        }
                    }
                }
            }

            if let Some(apps) = apps {
                for app in apps {
                    if let Err(e) = self.object_repo.update_app_graph(app).await {
                        error!("[DAEMON_SYNC] Graph batch update failed: {}", e);
                    }
                }
            }

            let _ = self.batch_repo
                .save_batch(TransactionBatch {
                    from_block_number: from_block as i64,
                    to_block_number: last_block_number as i64,
                    status: TransactionStatus::Confirmed,
                })
                .await;

            if let Err(e) = transaction.commit().await {
                error!("[NEW_REQ_HANDLER] Can't commit transaction {}, from {} | to {}", e, from_block, last_block_number);
            };
        }
    }
}

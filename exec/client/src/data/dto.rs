use crate::data::models::{Artifact, BuildRequest, ValidationProof};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DtoPublishing {
    pub id: Option<i64>,
    pub asset_address: String,
    pub track_id: i32,
    pub status: i32,
    pub artifact: Artifact,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidPublishingResponse {
    pub proof: Option<ValidationProof>,
    pub published: Vec<DtoPublishing>,
    pub reviewing: Vec<BuildRequest>,
}
